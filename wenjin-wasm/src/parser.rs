use sti::traits::UnwrapDebug;
use sti::reader::Reader;
use sti::arena::Arena;
use sti::manual_vec::ManualVec;

use crate::leb128;
use crate::{ValueType, RefType, FuncType, BlockType, Limits, TableType, MemoryType, GlobalType};
use crate::{Import, ImportKind, Imports, Global, Export, ExportKind, Element, ElementKind, Code, Data, DataKind};
use crate::{SubSection, Section, SectionKind, CustomSection};
use crate::ConstExpr;
use crate::{ModuleLimits, Module};
use crate::operator::{Operator, OperatorVisitor, MkOperator};


#[derive(Clone, Copy, Debug)]
pub struct ParseError {
    pub offset: usize,
    pub kind: ParseErrorKind,
}

#[derive(Clone, Copy, Debug)]
pub enum ParseErrorKind {
}

type Result<T> = core::result::Result<T, ParseError>;


#[derive(Clone)]
pub struct Parser<'a> {
    pub reader: Reader<'a, u8>,
}

impl<'a> Parser<'a> {
    #[inline]
    pub fn new(wasm: &'a [u8]) -> Self {
        Self { reader: Reader::new(wasm) }
    }

    #[inline]
    pub fn from_sub_section(wasm: &'a [u8], sub: SubSection) -> Self {
        let end = sub.offset + sub.len;
        let mut reader = Reader::new(&wasm[..end]);
        reader.set_offset(sub.offset);
        Self { reader }
    }

    #[inline]
    pub fn next(&mut self) -> Result<u8> {
        return self.reader.next().ok_or_else(||
            todo!());
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.reader.is_empty()
    }

    pub fn expect_done(&self) -> Result<()> {
        if self.reader.len() != 0 {
            todo!()
        }
        return Ok(());
    }

    #[inline]
    pub fn parse_i32(&mut self) -> Result<i32> {
        leb128::decode_i32(&mut self.reader).map_err(|_|
            todo!())
    }

    #[inline]
    pub fn parse_i64(&mut self) -> Result<i64> {
        leb128::decode_i64(&mut self.reader).map_err(|_|
            todo!())
    }

    #[inline]
    pub fn parse_f32(&mut self) -> Result<f32> {
        let bytes = self.reader.next_array::<4>().ok_or_else(|| todo!())?;
        return Ok(f32::from_le_bytes(bytes));
    }

    #[inline]
    pub fn parse_f64(&mut self) -> Result<f64> {
        let bytes = self.reader.next_array::<8>().ok_or_else(|| todo!())?;
        return Ok(f64::from_le_bytes(bytes));
    }

    #[inline]
    pub fn parse_u32(&mut self) -> Result<u32> {
        leb128::decode_u32(&mut self.reader).map_err(|_|
            todo!())
    }

    #[inline]
    pub fn parse_length(&mut self) -> Result<usize> {
        return Ok(self.parse_u32()? as usize);
    }

    pub fn parse_string(&mut self) -> Result<&'a str> {
        let len = self.parse_length()?;
        let bytes = self.reader.next_n(len).ok_or_else(|| todo!())?;
        let string = core::str::from_utf8(bytes).map_err(|_| todo!())?;
        return Ok(string);
    }

    pub fn parse_value_type(&mut self) -> Result<ValueType> {
        let at = self.next()?;
        let ty = ValueType::from_u8(at).ok_or_else(|| todo!())?;
        return Ok(ty);
    }

    pub fn parse_ref_type(&mut self) -> Result<RefType> {
        let at = self.next()?;
        let ty = RefType::from_u8(at).ok_or_else(|| todo!())?;
        return Ok(ty);
    }

    pub fn parse_func_type<'out>(&mut self, alloc: &'out Arena) -> Result<FuncType<'out>> {
        self.reader.expect(0x60).map_err(|_| todo!())?;

        let num_params = self.parse_length()?;
        let mut params = ManualVec::with_cap_in(alloc, num_params).ok_or_else(|| todo!())?;
        for _ in 0..num_params {
            params.push(self.parse_value_type()?).unwrap_debug();
        }

        let num_rets = self.parse_length()?;
        let mut rets = ManualVec::with_cap_in(alloc, num_rets).ok_or_else(|| todo!())?;
        for _ in 0..num_rets {
            rets.push(self.parse_value_type()?).unwrap_debug();
        }

        return Ok(FuncType { params: params.leak(), rets: rets.leak() });
    }

    pub fn parse_block_type(&mut self) -> Result<BlockType> {
        let ty = leb128::decode_i64(&mut self.reader).map_err(|_|
            todo!())?;

        // @todo: explain this.
        if ty < 0 {
            let high_bits = !0x7f;
            if ty & high_bits != high_bits {
                todo!()
            }
            let ty = (ty & !high_bits) as u64 as u8;

            if ty == 0x40 {
                return Ok(BlockType::Unit);
            }

            return Ok(BlockType::Value(ValueType::from_u8(ty).ok_or_else(|| todo!())?));
        }
        else {
            let ty = ty.try_into().map_err(|_| todo!())?;
            return Ok(BlockType::Func(ty));
        }
    }

    pub fn parse_limits(&mut self) -> Result<Limits> {
        return Ok(match self.next()? {
            0x00 => Limits { min: self.parse_u32()?, max: None },
            0x01 => Limits { min: self.parse_u32()?, max: Some(self.parse_u32()?) },

            _ => todo!()
        });
    }

    pub fn parse_table_type(&mut self) -> Result<TableType> {
        let ty = self.parse_ref_type()?;
        let limits = self.parse_limits()?;
        return Ok(TableType { ty, limits });
    }

    pub fn parse_memory_type(&mut self) -> Result<MemoryType> {
        return Ok(MemoryType { limits: self.parse_limits()? });
    }

    pub fn parse_global_type(&mut self) -> Result<GlobalType> {
        let ty = self.parse_value_type()?;
        let mutt = match self.next()? {
            0 => false,
            1 => true,

            _ => todo!(),
        };
        return Ok(GlobalType { ty, mutt });
    }


    pub fn parse_module_header(&mut self) -> Result<()> {
        self.reader.expect_n(b"\0asm").map_err(|_| todo!())?;
        self.reader.expect_n(&[1, 0, 0, 0]).map_err(|_| todo!())?;
        return Ok(())
    }

    pub fn parse_sub_section(&mut self) -> Result<SubSection> {
        let len = self.parse_length()?;
        let offset = self.reader.offset();
        self.reader.next_n(len).ok_or_else(|| todo!())?;
        return Ok(SubSection { offset, len });
    }

    pub fn parse_section(&mut self) -> Result<Section> {
        let kind = self.next()?;
        let kind = SectionKind::from_u8(kind).ok_or_else(|| todo!())?;
        let sub = self.parse_sub_section()?;
        return Ok(Section { kind, sub });
    }

    pub fn sub_parser(&self, sub: SubSection) -> Self {
        Self::from_sub_section(self.reader.original_slice(), sub)
    }

    pub fn parse_custom_section(&mut self) -> Result<CustomSection<'a>> {
        let name = self.parse_string()?;
        let data = self.reader.as_slice();
        self.reader.consume(data.len());
        return Ok(CustomSection { name, data });
    }

    pub fn parse_import(&mut self) -> Result<Import<'a>> {
        let module = self.parse_string()?;
        let name = self.parse_string()?;

        let kind = match self.next()? {
            0x00 => ImportKind::Func(self.parse_u32()?),
            0x01 => ImportKind::Table(self.parse_table_type()?),
            0x02 => ImportKind::Memory(self.parse_memory_type()?),
            0x03 => ImportKind::Global(self.parse_global_type()?),

            _ => todo!()
        };

        return Ok(Import { module, name, kind });
    }

    pub fn parse_global(&mut self) -> Result<Global> {
        let ty = self.parse_global_type()?;
        let init = self.parse_const_expr()?;
        return Ok(Global { ty, init });
    }

    pub fn parse_export(&mut self) -> Result<Export<'a>> {
        let name = self.parse_string()?;
        let kind = match self.next()? {
            0x00 => ExportKind::Func(self.parse_u32()?),
            0x01 => ExportKind::Table(self.parse_u32()?),
            0x02 => ExportKind::Memory(self.parse_u32()?),
            0x03 => ExportKind::Global(self.parse_u32()?),

            _ => todo!()
        };
        return Ok(Export { name, kind });
    }

    pub fn parse_element<'out>(&mut self, alloc: &'out Arena) -> Result<Element<'out>> {
        return Ok(match self.parse_u32()? {
            0 => {
                let ConstExpr::I32(offset) = self.parse_const_expr()? else {
                    todo!()
                };

                let num_values = self.parse_length()?;
                let mut values = ManualVec::with_cap_in(alloc, num_values).ok_or_else(|| todo!())?;
                for _ in 0..num_values {
                    values.push(self.parse_u32()?).unwrap_debug();
                }

                Element {
                    ty: ValueType::FuncRef,
                    kind: ElementKind::Active { table: 0, offset: offset as u32 },
                    values: values.leak(),
                }
            }

            _ => todo!(),
        });
    }

    pub fn parse_code<'out>(&mut self, max_locals: u32, alloc: &'out Arena) -> Result<Code<'out>> {
        let sub = self.parse_sub_section()?;

        let mut p = self.sub_parser(sub);

        let num_local_groups = p.parse_u32()?;

        let mut locals = ManualVec::new_in(alloc);
        for _ in 0..num_local_groups {
            let n = p.parse_length()?;
            let ty = p.parse_value_type()?;

            if locals.len() + n > max_locals as usize {
                todo!()
            }

            locals.reserve_extra(n).map_err(|_| todo!())?;
            for _ in 0..n {
                locals.push(ty).unwrap_debug();
            }
        }

        let expr = SubSection {
            offset: p.reader.offset(),
            len:    p.reader.len(),
        };

        return Ok(Code { locals: locals.leak(), expr });
    }

    pub fn parse_data(&mut self) -> Result<Data<'a>> {
        return Ok(match self.parse_u32()? {
            0 => {
                let offset = self.parse_const_expr()?;
                let ConstExpr::I32(offset) = offset else {
                    todo!()
                };

                let len = self.parse_length()?;
                let values = self.reader.next_n(len).ok_or_else(|| todo!())?;

                let kind = DataKind::Active { mem: 0, offset: offset as u32 };

                Data { kind, values }
            }

            _ => todo!()
        });
    }


    pub fn parse_const_expr(&mut self) -> Result<ConstExpr> {
        let result = match self.parse_operator()? {
            Operator::I32Const { value } => ConstExpr::I32(value),
            Operator::I64Const { value } => ConstExpr::I64(value),
            Operator::F32Const { value } => ConstExpr::F32(value),
            Operator::F64Const { value } => ConstExpr::F64(value),

            _ => todo!(),
        };

        let Operator::End = self.parse_operator()? else {
            todo!()
        };

        return Ok(result);
    }

    pub fn parse_operator(&mut self) -> Result<Operator> {
        self.parse_operator_with(MkOperator)
    }

    pub fn parse_operator_with<V: OperatorVisitor>(&mut self, mut v: V) -> Result<V::Output> {
        use crate::opcode::*;

        let at = self.next()?;
        return Ok(match at {
            UNREACHABLE     => v.visit_unreachable(),
            NOP             => v.visit_nop(),
            BLOCK           => v.visit_block(self.parse_block_type()?),
            LOOP            => v.visit_loop(self.parse_block_type()?),
            IF              => v.visit_if(self.parse_block_type()?),
            ELSE            => v.visit_else(),
            END             => v.visit_end(),
            BR              => v.visit_br(self.parse_u32()?),
            BR_IF           => v.visit_br_if(self.parse_u32()?),
            BR_TABLE => {
                let num_labels = self.parse_u32()?;
                for _ in 0..num_labels {
                    self.parse_u32()?;
                }
                let default = self.parse_u32()?;

                v.visit_br_table(default)
            }
            RETURN          => v.visit_return(),
            CALL            => v.visit_call(self.parse_u32()?),
            CALL_INDIRECT   => v.visit_call_indirect(self.parse_u32()?, self.parse_u32()?),
            DROP            => v.visit_drop(),
            SELECT          => v.visit_select(),
            TYPED_SELECT    => todo!(),
            LOCAL_GET       => v.visit_local_get(self.parse_u32()?),
            LOCAL_SET       => v.visit_local_set(self.parse_u32()?),
            LOCAL_TEE       => v.visit_local_tee(self.parse_u32()?),
            GLOBAL_GET      => v.visit_global_get(self.parse_u32()?),
            GLOBAL_SET      => v.visit_global_set(self.parse_u32()?),
            TABLE_GET       => todo!(),
            TABLE_SET       => todo!(),
            I32_LOAD        => v.visit_i32_load(self.parse_u32()?, self.parse_u32()?),
            I64_LOAD        => v.visit_i64_load(self.parse_u32()?, self.parse_u32()?),
            F32_LOAD        => v.visit_f32_load(self.parse_u32()?, self.parse_u32()?),
            F64_LOAD        => v.visit_f64_load(self.parse_u32()?, self.parse_u32()?),
            I32_LOAD8_S     => v.visit_i32_load8_s(self.parse_u32()?, self.parse_u32()?),
            I32_LOAD8_U     => v.visit_i32_load8_u(self.parse_u32()?, self.parse_u32()?),
            I32_LOAD16_S    => v.visit_i32_load16_s(self.parse_u32()?, self.parse_u32()?),
            I32_LOAD16_U    => v.visit_i32_load16_u(self.parse_u32()?, self.parse_u32()?),
            I64_LOAD8_S     => v.visit_i64_load8_s(self.parse_u32()?, self.parse_u32()?),
            I64_LOAD8_U     => v.visit_i64_load8_u(self.parse_u32()?, self.parse_u32()?),
            I64_LOAD16_S    => v.visit_i64_load16_s(self.parse_u32()?, self.parse_u32()?),
            I64_LOAD16_U    => v.visit_i64_load16_u(self.parse_u32()?, self.parse_u32()?),
            I64_LOAD32_S    => v.visit_i64_load32_s(self.parse_u32()?, self.parse_u32()?),
            I64_LOAD32_U    => v.visit_i64_load32_u(self.parse_u32()?, self.parse_u32()?),
            I32_STORE       => v.visit_i32_store(self.parse_u32()?, self.parse_u32()?),
            I64_STORE       => v.visit_i64_store(self.parse_u32()?, self.parse_u32()?),
            F32_STORE       => v.visit_f32_store(self.parse_u32()?, self.parse_u32()?),
            F64_STORE       => v.visit_f64_store(self.parse_u32()?, self.parse_u32()?),
            I32_STORE8      => v.visit_i32_store8(self.parse_u32()?, self.parse_u32()?),
            I32_STORE16     => v.visit_i32_store16(self.parse_u32()?, self.parse_u32()?),
            I64_STORE8      => v.visit_i64_store8(self.parse_u32()?, self.parse_u32()?),
            I64_STORE16     => v.visit_i64_store16(self.parse_u32()?, self.parse_u32()?),
            I64_STORE32     => v.visit_i64_store32(self.parse_u32()?, self.parse_u32()?),
            MEMORY_SIZE     => v.visit_memory_size(self.parse_u32()?),
            MEMORY_GROW     => v.visit_memory_grow(self.parse_u32()?),
            I32_CONST       => v.visit_i32_const(self.parse_i32()?),
            I64_CONST       => v.visit_i64_const(self.parse_i64()?),
            F32_CONST       => v.visit_f32_const(self.parse_f32()?),
            F64_CONST       => v.visit_f64_const(self.parse_f64()?),
            I32_EQZ         => v.visit_i32_eqz(),
            I32_EQ          => v.visit_i32_eq(),
            I32_NE          => v.visit_i32_ne(),
            I32_LT_S        => v.visit_i32_lt_s(),
            I32_LT_U        => v.visit_i32_lt_u(),
            I32_GT_S        => v.visit_i32_gt_s(),
            I32_GT_U        => v.visit_i32_gt_u(),
            I32_LE_S        => v.visit_i32_le_s(),
            I32_LE_U        => v.visit_i32_le_u(),
            I32_GE_S        => v.visit_i32_ge_s(),
            I32_GE_U        => v.visit_i32_ge_u(),
            I64_EQZ         => v.visit_i64_eqz(),
            I64_EQ          => v.visit_i64_eq(),
            I64_NE          => v.visit_i64_ne(),
            I64_LT_S        => v.visit_i64_lt_s(),
            I64_LT_U        => v.visit_i64_lt_u(),
            I64_GT_S        => v.visit_i64_gt_s(),
            I64_GT_U        => v.visit_i64_gt_u(),
            I64_LE_S        => v.visit_i64_le_s(),
            I64_LE_U        => v.visit_i64_le_u(),
            I64_GE_S        => v.visit_i64_ge_s(),
            I64_GE_U        => v.visit_i64_ge_u(),
            F32_EQ          => v.visit_f32_eq(),
            F32_NE          => v.visit_f32_ne(),
            F32_LT          => v.visit_f32_lt(),
            F32_GT          => v.visit_f32_gt(),
            F32_LE          => v.visit_f32_le(),
            F32_GE          => v.visit_f32_ge(),
            F64_EQ          => v.visit_f64_eq(),
            F64_NE          => v.visit_f64_ne(),
            F64_LT          => v.visit_f64_lt(),
            F64_GT          => v.visit_f64_gt(),
            F64_LE          => v.visit_f64_le(),
            F64_GE          => v.visit_f64_ge(),
            I32_CLZ         => v.visit_i32_clz(),
            I32_CTZ         => v.visit_i32_ctz(),
            I32_POPCNT      => v.visit_i32_popcnt(),
            I32_ADD         => v.visit_i32_add(),
            I32_SUB         => v.visit_i32_sub(),
            I32_MUL         => v.visit_i32_mul(),
            I32_DIV_S       => v.visit_i32_div_s(),
            I32_DIV_U       => v.visit_i32_div_u(),
            I32_REM_S       => v.visit_i32_rem_s(),
            I32_REM_U       => v.visit_i32_rem_u(),
            I32_AND         => v.visit_i32_and(),
            I32_OR          => v.visit_i32_or(),
            I32_XOR         => v.visit_i32_xor(),
            I32_SHL         => v.visit_i32_shl(),
            I32_SHR_S       => v.visit_i32_shr_s(),
            I32_SHR_U       => v.visit_i32_shr_u(),
            I32_ROTL        => v.visit_i32_rotl(),
            I32_ROTR        => v.visit_i32_rotr(),
            I64_CLZ         => v.visit_i64_clz(),
            I64_CTZ         => v.visit_i64_ctz(),
            I64_POPCNT      => v.visit_i64_popcnt(),
            I64_ADD         => v.visit_i64_add(),
            I64_SUB         => v.visit_i64_sub(),
            I64_MUL         => v.visit_i64_mul(),
            I64_DIV_S       => v.visit_i64_div_s(),
            I64_DIV_U       => v.visit_i64_div_u(),
            I64_REM_S       => v.visit_i64_rem_s(),
            I64_REM_U       => v.visit_i64_rem_u(),
            I64_AND         => v.visit_i64_and(),
            I64_OR          => v.visit_i64_or(),
            I64_XOR         => v.visit_i64_xor(),
            I64_SHL         => v.visit_i64_shl(),
            I64_SHR_S       => v.visit_i64_shr_s(),
            I64_SHR_U       => v.visit_i64_shr_u(),
            I64_ROTL        => v.visit_i64_rotl(),
            I64_ROTR        => v.visit_i64_rotr(),
            F32_ABS         => v.visit_f32_abs(),
            F32_NEG         => v.visit_f32_neg(),
            F32_CEIL        => v.visit_f32_ceil(),
            F32_FLOOR       => v.visit_f32_floor(),
            F32_TRUNC       => v.visit_f32_trunc(),
            F32_NEAREST     => v.visit_f32_nearest(),
            F32_SQRT        => v.visit_f32_sqrt(),
            F32_ADD         => v.visit_f32_add(),
            F32_SUB         => v.visit_f32_sub(),
            F32_MUL         => v.visit_f32_mul(),
            F32_DIV         => v.visit_f32_div(),
            F32_MIN         => v.visit_f32_min(),
            F32_MAX         => v.visit_f32_max(),
            F32_COPYSIGN    => v.visit_f32_copysign(),
            F64_ABS         => v.visit_f64_abs(),
            F64_NEG         => v.visit_f64_neg(),
            F64_CEIL        => v.visit_f64_ceil(),
            F64_FLOOR       => v.visit_f64_floor(),
            F64_TRUNC       => v.visit_f64_trunc(),
            F64_NEAREST     => v.visit_f64_nearest(),
            F64_SQRT        => v.visit_f64_sqrt(),
            F64_ADD         => v.visit_f64_add(),
            F64_SUB         => v.visit_f64_sub(),
            F64_MUL         => v.visit_f64_mul(),
            F64_DIV         => v.visit_f64_div(),
            F64_MIN         => v.visit_f64_min(),
            F64_MAX         => v.visit_f64_max(),
            F64_COPYSIGN    => v.visit_f64_copysign(),
            I32_WRAP_I64    => v.visit_i32_wrap_i64(),
            I32_TRUNC_F32_S => v.visit_i32_trunc_f32_s(),
            I32_TRUNC_F32_U => v.visit_i32_trunc_f32_u(),
            I32_TRUNC_F64_S => v.visit_i32_trunc_f64_s(),
            I32_TRUNC_F64_U => v.visit_i32_trunc_f64_u(),
            I64_EXTEND_I32_S => v.visit_i64_extend_i32_s(),
            I64_EXTEND_I32_U => v.visit_i64_extend_i32_u(),
            I64_TRUNC_F32_S => v.visit_i64_trunc_f32_s(),
            I64_TRUNC_F32_U => v.visit_i64_trunc_f32_u(),
            I64_TRUNC_F64_S => v.visit_i64_trunc_f64_s(),
            I64_TRUNC_F64_U => v.visit_i64_trunc_f64_u(),
            F32_CONVERT_I32_S => v.visit_f32_convert_i32_s(),
            F32_CONVERT_I32_U => v.visit_f32_convert_i32_u(),
            F32_CONVERT_I64_S => v.visit_f32_convert_i64_s(),
            F32_CONVERT_I64_U => v.visit_f32_convert_i64_u(),
            F32_DEMOTE_F64 => v.visit_f32_demote_f64(),
            F64_CONVERT_I32_S => v.visit_f64_convert_i32_s(),
            F64_CONVERT_I32_U => v.visit_f64_convert_i32_u(),
            F64_CONVERT_I64_S => v.visit_f64_convert_i64_s(),
            F64_CONVERT_I64_U => v.visit_f64_convert_i64_u(),
            F64_PROMOTE_F32 => v.visit_f64_promote_f32(),
            I32_REINTERPRET_F32 => v.visit_i32_reinterpret_f32(),
            I64_REINTERPRET_F64 => v.visit_i64_reinterpret_f64(),
            F32_REINTERPRET_I32 => v.visit_f32_reinterpret_i32(),
            F64_REINTERPRET_I64 => v.visit_f64_reinterpret_i64(),
            I32_EXTEND8_S   => v.visit_i32_extend8_s(),
            I32_EXTEND16_S  => v.visit_i32_extend16_s(),
            I64_EXTEND8_S   => v.visit_i64_extend8_s(),
            I64_EXTEND16_S  => v.visit_i64_extend16_s(),
            I64_EXTEND32_S  => v.visit_i64_extend32_s(),
            REF_NULL        => todo!(),
            REF_IS_NULL     => todo!(),
            REF_FUNC        => todo!(),

            0xfc => {
                todo!()
                //pub const MEMORY_COPY:              u32 = 10;
                //pub const MEMORY_FILL:              u32 = 11;
            }

            _ => {
                todo!()
            }
        })
    }


    pub fn parse_module
        (wasm: &'a [u8],
         limits: ModuleLimits,
         alloc: &'a Arena)
        -> Result<Module<'a>>
    {
        let mut p = Parser::new(wasm);
        p.parse_module_header()?;

        let mut has_section = [false; SectionKind::COUNT];

        let mut module = Module::default();

        let mut customs = ManualVec::new_in(alloc);

        while !p.is_done() {
            let section = p.parse_section()?;
            let kind = section.kind;

            if kind != SectionKind::Custom && has_section[kind as usize] {
                todo!()
            }
            has_section[kind as usize] = true;

            let mut sp = p.sub_parser(section.sub);
            match kind {
                SectionKind::Custom => {
                    if customs.len() >= limits.max_customs as usize {
                        todo!();
                    }

                    customs.push_or_alloc(sp.parse_custom_section()?)
                        .map_err(|_| todo!())?;
                }

                SectionKind::Type => {
                    let num_types = sp.parse_u32()?;
                    if num_types > limits.max_types {
                        todo!();
                    }

                    let mut types =
                        ManualVec::with_cap_in(alloc, num_types as usize)
                        .ok_or_else(|| todo!())?;

                    for _ in 0..num_types {
                        types.push(sp.parse_func_type(alloc)?).unwrap_debug();
                    }

                    module.types = types.leak();
                }

                SectionKind::Import => {
                    let num_imports = sp.parse_u32()?;
                    if num_imports > limits.max_imports {
                        todo!();
                    }

                    let mut imports =
                        ManualVec::with_cap_in(alloc, num_imports as usize)
                        .ok_or_else(|| todo!())?;

                    let mut num_funcs = 0;
                    let mut num_tables = 0;
                    let mut num_memories = 0;
                    let mut num_globals = 0;

                    for _ in 0..num_imports {
                        let import = sp.parse_import()?;
                        match import.kind {
                            ImportKind::Func(_)   => num_funcs    += 1,
                            ImportKind::Table(_)  => num_tables   += 1,
                            ImportKind::Memory(_) => num_memories += 1,
                            ImportKind::Global(_) => num_globals  += 1,
                        }
                        imports.push(import).unwrap_debug();
                    }

                    let mut funcs =
                        ManualVec::with_cap_in(alloc, num_funcs)
                        .ok_or_else(|| todo!())?;

                    let mut tables =
                        ManualVec::with_cap_in(alloc, num_tables)
                        .ok_or_else(|| todo!())?;

                    let mut memories =
                        ManualVec::with_cap_in(alloc, num_memories)
                        .ok_or_else(|| todo!())?;

                    let mut globals =
                        ManualVec::with_cap_in(alloc, num_globals)
                        .ok_or_else(|| todo!())?;

                    for import in imports.iter().copied() {
                        match import.kind {
                            ImportKind::Func(it)   => funcs.push(it).unwrap_debug(),
                            ImportKind::Table(it)  => tables.push(it).unwrap_debug(),
                            ImportKind::Memory(it) => memories.push(it).unwrap_debug(),
                            ImportKind::Global(it) => globals.push(it).unwrap_debug(),
                        }
                    }

                    module.imports = Imports {
                        imports: imports.leak(),
                        funcs:    funcs.leak(),
                        tables:   tables.leak(),
                        memories: memories.leak(),
                        globals:  globals.leak(),
                    };
                }

                SectionKind::Function => {
                    if has_section[SectionKind::Code as usize] {
                        todo!()
                    }

                    let num_funcs = sp.parse_u32()?;
                    if num_funcs > limits.max_funcs {
                        todo!();
                    }

                    let mut funcs =
                        ManualVec::with_cap_in(alloc, num_funcs as usize)
                        .ok_or_else(|| todo!())?;

                    for _ in 0..num_funcs {
                        funcs.push(sp.parse_u32()?).unwrap_debug();
                    }

                    module.funcs = funcs.leak();
                }

                SectionKind::Table => {
                    let num_tables = sp.parse_u32()?;
                    if num_tables > limits.max_tables {
                        todo!();
                    }

                    let mut tables =
                        ManualVec::with_cap_in(alloc, num_tables as usize)
                        .ok_or_else(|| todo!())?;

                    for _ in 0..num_tables {
                        tables.push(sp.parse_table_type()?).unwrap_debug();
                    }

                    module.tables = tables.leak();
                }

                SectionKind::Memory => {
                    let num_memories = sp.parse_u32()?;
                    if num_memories > limits.max_memories {
                        todo!();
                    }

                    let mut memories =
                        ManualVec::with_cap_in(alloc, num_memories as usize)
                        .ok_or_else(|| todo!())?;

                    for _ in 0..num_memories {
                        memories.push(sp.parse_memory_type()?).unwrap_debug();
                    }

                    module.memories = memories.leak();
                }

                SectionKind::Global => {
                    let num_globals = sp.parse_u32()?;
                    if num_globals > limits.max_globals {
                        todo!();
                    }

                    let mut globals =
                        ManualVec::with_cap_in(alloc, num_globals as usize)
                        .ok_or_else(|| todo!())?;

                    for _ in 0..num_globals {
                        globals.push(sp.parse_global()?).unwrap_debug();
                    }

                    module.globals = globals.leak();
                }

                SectionKind::Export => {
                    let num_exports = sp.parse_u32()?;
                    if num_exports > limits.max_exports {
                        todo!();
                    }

                    let mut exports =
                        ManualVec::with_cap_in(alloc, num_exports as usize)
                        .ok_or_else(|| todo!())?;

                    for _ in 0..num_exports {
                        exports.push(sp.parse_export()?).unwrap_debug();
                    }

                    module.exports = exports.leak();
                }

                SectionKind::Start => {
                    module.start = Some(sp.parse_u32()?);
                }

                SectionKind::Element => {
                    let num_elements = sp.parse_u32()?;
                    if num_elements > limits.max_elements {
                        todo!();
                    }

                    let mut elements =
                        ManualVec::with_cap_in(alloc, num_elements as usize)
                        .ok_or_else(|| todo!())?;

                    for _ in 0..num_elements {
                        elements.push(sp.parse_element(alloc)?).unwrap_debug();
                    }

                    module.elements = elements.leak();
                }

                SectionKind::Code => {
                    let num_codes = sp.parse_u32()?;
                    if num_codes as usize != module.funcs.len() {
                        todo!()
                    }

                    let mut codes =
                        ManualVec::with_cap_in(alloc, num_codes as usize)
                        .ok_or_else(|| todo!())?;

                    for _ in 0..num_codes {
                        codes.push(sp.parse_code(limits.max_locals, alloc)?).unwrap_debug();
                    }

                    module.codes = codes.leak();
                }

                SectionKind::Data => {
                    let num_datas = sp.parse_u32()?;
                    if num_datas > limits.max_datas {
                        todo!();
                    }

                    let mut datas =
                        ManualVec::with_cap_in(alloc, num_datas as usize)
                        .ok_or_else(|| todo!())?;

                    for _ in 0..num_datas {
                        datas.push(sp.parse_data()?).unwrap_debug();
                    }

                    module.datas = datas.leak();
                }

                SectionKind::DataCount => {
                    sp.parse_u32()?;
                }
            }
            sp.expect_done()?;
        }

        module.customs = customs.leak();

        return Ok(module);
    }
}


