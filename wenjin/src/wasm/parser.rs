use sti::alloc::Alloc;
use sti::vec::Vec;
use sti::reader::Reader;
use crate::leb128;

use super::*;
use super::operand::*;



#[derive(Clone, Copy, Debug)]
pub struct ExprParseError {
    pub pos:  usize,
    pub expr: usize,
    pub op:   usize,
    pub data: ExprParseErrorData,
}

#[derive(Clone, Copy, Debug)]
pub enum ExprParseErrorData {
    EOI,
}

#[derive(Clone)]
pub struct ExprParser<'i> {
    pub reader:      Reader<'i, u8>,
    pub code_begin:  *const u8,
    pub expr_begin:  usize,
    pub op_begin:    usize,
}

impl<'i> ExprParser<'i> {
    pub fn new(reader: Reader<'i, u8>, code_begin: *const u8) -> Self {
        let current_offset = reader.as_ptr() as usize - code_begin as usize;
        ExprParser {
            reader,
            code_begin,
            expr_begin: current_offset,
            op_begin:   current_offset,
        }
    }

    #[inline(always)]
    pub fn begin_expr(&mut self) {
        self.expr_begin = self.reader.as_ptr() as usize - self.code_begin as usize;
    }

    #[inline(always)]
    pub fn begin_op(&mut self) {
        self.op_begin = self.reader.as_ptr() as usize - self.code_begin as usize;
    }

    pub fn err(&self, data: ExprParseErrorData) -> ExprParseError {
        ExprParseError {
            pos:  self.reader.as_ptr() as usize - self.code_begin as usize,
            expr: self.expr_begin,
            op:   self.op_begin,
            data,
        }
    }

    pub fn err_eoi(&self) -> ExprParseError {
        self.err(ExprParseErrorData::EOI)
    }


    #[inline]
    pub fn parse_u32(&mut self) -> Result<u32, ExprParseError> {
        if let Some(value) = self.reader.next_if(|byte| *byte < 0x80) {
            Ok(value as u32)
        }
        else {
            leb128::decode_u32(&mut self.reader).map_err(|_|
                unimplemented!())
        }
    }

    #[inline]
    pub fn next(&mut self) -> Result<u8, ExprParseError> {
        self.reader.next().ok_or_else(|| self.err_eoi())
    }


    pub fn parse_block_type(&mut self) -> Result<BlockType, ExprParseError> {
        let ty = leb128::decode_i64(&mut self.reader).ok().ok_or_else(||
            unimplemented!())?;

        if ty < 0 {
            let high_bits = !0x7f;
            if ty & high_bits != high_bits {
                unimplemented!()
            }
            let ty = (ty & !high_bits) as u64 as u8;

            if ty == 0x40 {
                return Ok(BlockType::Unit);
            }

            return Ok(BlockType::Value(ValueType::from_u8(ty).ok_or_else(|| unimplemented!())?));
        }
        else {
            let ty = ty.try_into().ok().ok_or_else(|| unimplemented!())?;
            return Ok(BlockType::Func(ty));
        }
    }

    pub fn parse_br_table<F: FnMut(u32)>(&mut self, mut f: F) -> Result<u32, ExprParseError> {
        let num_labels = self.parse_u32()?;
        for _ in 0..num_labels {
            f(self.parse_u32()?);
        }
        self.parse_u32()
    }

    pub fn parse_expr<F: FnMut(Operand, &mut ExprParser) -> Result<bool, ExprParseError>>(&mut self, mut f: F) -> Result<(), ExprParseError> {
        self.begin_expr();

        loop {
            self.begin_op();

            let source = self.op_begin as u32;

            let opcode = self.next()?;

            use opcode::*;
            let done = match opcode {
                UNREACHABLE => {
                    f(Operand { source, data: OperandData::Unreachable }, self)?
                }

                NOP => {
                    f(Operand { source, data: OperandData::Nop }, self)?
                }

                BLOCK => {
                    let ty = self.parse_block_type()?;
                    f(Operand { source, data: OperandData::Block(ty) }, self)?
                }

                LOOP => {
                    let ty = self.parse_block_type()?;
                    f(Operand { source, data: OperandData::Loop(ty) }, self)?
                }

                IF => {
                    let ty = self.parse_block_type()?;
                    f(Operand { source, data: OperandData::If(ty) }, self)?
                }

                ELSE => {
                    f(Operand { source, data: OperandData::Else }, self)?
                }

                END => {
                    f(Operand { source, data: OperandData::End }, self)?
                }


                BR => {
                    let label = self.parse_u32()?;
                    f(Operand { source, data: OperandData::Br(label) }, self)?
                }

                BR_IF => {
                    let label = self.parse_u32()?;
                    f(Operand { source, data: OperandData::BrIf(label) }, self)?
                }

                BR_TABLE => {
                    let old_pos = self.reader.as_ptr();
                    let result = f(Operand { source, data: OperandData::BrTable }, self)?;
                    debug_assert_ne!(self.reader.as_ptr(), old_pos);
                    result
                }


                RETURN => {
                    f(Operand { source, data: OperandData::Return }, self)?
                }

                CALL => {
                    let func = self.parse_u32()?;
                    f(Operand { source, data: OperandData::Call(func) }, self)?
                }

                CALL_INDIRECT => {
                    let ty    = self.parse_u32()?;
                    let table = self.parse_u32()?;
                    f(Operand { source, data: OperandData::CallIndirect(table, ty) }, self)?
                }


                DROP => {
                    f(Operand { source, data: OperandData::Drop }, self)?
                }

                SELECT => {
                    f(Operand { source, data: OperandData::Select }, self)?
                }


                I32_CONST => {
                    let value = leb128::decode_i32(&mut self.reader).ok().ok_or_else(|| unimplemented!())?;
                    f(Operand { source, data: OperandData::I32Const(value) }, self)?
                }

                I64_CONST => {
                    let value = leb128::decode_i64(&mut self.reader).ok().ok_or_else(|| unimplemented!())?;
                    f(Operand { source, data: OperandData::I64Const(value) }, self)?
                }

                F32_CONST => {
                    let value = self.reader.next_array::<4>().ok_or_else(|| unimplemented!())?;
                    let value = f32::from_le_bytes(value);
                    f(Operand { source, data: OperandData::F32Const(value) }, self)?
                }

                F64_CONST => {
                    let value = self.reader.next_array::<8>().ok_or_else(|| unimplemented!())?;
                    let value = f64::from_le_bytes(value);
                    f(Operand { source, data: OperandData::F64Const(value) }, self)?
                }


                LOCAL_GET => {
                    let local = self.parse_u32()?;
                    f(Operand { source, data: OperandData::LocalGet(local) }, self)?
                }

                LOCAL_SET => {
                    let local = self.parse_u32()?;
                    f(Operand { source, data: OperandData::LocalSet(local) }, self)?
                }

                LOCAL_TEE => {
                    let local = self.parse_u32()?;
                    f(Operand { source, data: OperandData::LocalTee(local) }, self)?
                }


                GLOBAL_GET => {
                    let local = self.parse_u32()?;
                    f(Operand { source, data: OperandData::GlobalGet(local) }, self)?
                }

                GLOBAL_SET => {
                    let local = self.parse_u32()?;
                    f(Operand { source, data: OperandData::GlobalSet(local) }, self)?
                }


                I32_LOAD | I32_LOAD8_S | I32_LOAD8_U | I32_LOAD16_S | I32_LOAD16_U |
                I64_LOAD | I64_LOAD8_S | I64_LOAD8_U | I64_LOAD16_S | I64_LOAD16_U | I64_LOAD32_S | I64_LOAD32_U |
                F32_LOAD | F64_LOAD
                => {
                    let align  = self.parse_u32()?;
                    let offset = self.parse_u32()?;
                    let load   = Load::from_opcode(opcode);

                    f(Operand { source, data: OperandData::Load { load, align, offset } }, self)?
                }


                I32_STORE | I32_STORE8 | I32_STORE16 |
                I64_STORE | I64_STORE8 | I64_STORE16 | I64_STORE32 |
                F32_STORE | F64_STORE
                => {
                    let align  = self.parse_u32()?;
                    let offset = self.parse_u32()?;
                    let store  = Store::from_opcode(opcode);

                    f(Operand { source, data: OperandData::Store { store, align, offset } }, self)?
                }


                I32_EQZ | I64_EQZ
                => {
                    let test_op = TestOp::from_opcode(opcode);
                    f(Operand { source, data: OperandData::TestOp(test_op) }, self)?
                }

                I32_EQ   | I32_NE   | I32_LT_S | I32_LT_U |
                I64_EQ   | I64_NE   | I64_LT_S | I64_LT_U |
                I32_GT_S | I32_GT_U | I32_LE_S | I32_LE_U | I32_GE_S | I32_GE_U |
                I64_GT_S | I64_GT_U | I64_LE_S | I64_LE_U | I64_GE_S | I64_GE_U |
                F32_EQ   | F32_NE   | F32_LT   | F32_GT   | F32_LE   | F32_GE   |
                F64_EQ   | F64_NE   | F64_LT   | F64_GT   | F64_LE   | F64_GE
                => {
                    let rel_op = RelOp::from_opcode(opcode);
                    f(Operand { source, data: OperandData::RelOp(rel_op) }, self)?
                }

                I32_CLZ | I32_CTZ | I32_POPCNT |
                I64_CLZ | I64_CTZ | I64_POPCNT |
                I32_EXTEND8_S | I32_EXTEND16_S |
                I64_EXTEND8_S | I64_EXTEND16_S | I64_EXTEND32_S |
                F32_ABS | F32_NEG | F32_CEIL | F32_FLOOR | F32_TRUNC | F32_NEAREST | F32_SQRT |
                F64_ABS | F64_NEG | F64_CEIL | F64_FLOOR | F64_TRUNC | F64_NEAREST | F64_SQRT
                => {
                    let op1 = Op1::from_opcode(opcode);
                    f(Operand { source, data: OperandData::Op1(op1) }, self)?
                }

                I32_ADD | I32_SUB | I32_MUL | I32_DIV_S | I32_DIV_U | I32_REM_S | I32_REM_U    |
                I32_AND | I32_OR  | I32_XOR | I32_SHL   | I32_SHR_S | I32_SHR_U | I32_ROTL     | I32_ROTR |
                I64_ADD | I64_SUB | I64_MUL | I64_DIV_S | I64_DIV_U | I64_REM_S | I64_REM_U    |
                I64_AND | I64_OR  | I64_XOR | I64_SHL   | I64_SHR_S | I64_SHR_U | I64_ROTL     | I64_ROTR |
                F32_ADD | F32_SUB | F32_MUL | F32_DIV   | F32_MIN   | F32_MAX   | F32_COPYSIGN |
                F64_ADD | F64_SUB | F64_MUL | F64_DIV   | F64_MIN   | F64_MAX   | F64_COPYSIGN
                => {
                    let op2 = Op2::from_opcode(opcode);
                    f(Operand { source, data: OperandData::Op2(op2) }, self)?
                }

                I32_WRAP_I64      |
                I32_TRUNC_F32_S   | I32_TRUNC_F32_U   |
                I32_TRUNC_F64_S   | I32_TRUNC_F64_U   |
                I64_TRUNC_F32_S   | I64_TRUNC_F32_U   |
                I64_TRUNC_F64_S   | I64_TRUNC_F64_U   |
                I64_EXTEND_I32_S  | I64_EXTEND_I32_U  |
                F32_CONVERT_I32_S | F32_CONVERT_I32_U |
                F32_CONVERT_I64_S | F32_CONVERT_I64_U |
                F64_CONVERT_I32_S | F64_CONVERT_I32_U |
                F64_CONVERT_I64_S | F64_CONVERT_I64_U |
                F32_DEMOTE_F64    | F64_PROMOTE_F32
                => {
                    let cvt = Convert::from_opcode(opcode);
                    f(Operand { source, data: OperandData::Convert(cvt) }, self)?
                }

                I32_REINTERPRET_F32 | I64_REINTERPRET_F64 | F32_REINTERPRET_I32 | F64_REINTERPRET_I64
                => {
                    let reterp = Reinterpret::from_opcode(opcode);
                    f(Operand { source, data: OperandData::Reinterpret(reterp) }, self)?
                }


                MEMORY_SIZE => {
                    self.reader.expect(0x00).ok().ok_or_else(|| unimplemented!())?;
                    f(Operand { source, data: OperandData::MemorySize }, self)?
                }

                MEMORY_GROW => {
                    self.reader.expect(0x00).ok().ok_or_else(|| unimplemented!())?;
                    f(Operand { source, data: OperandData::MemoryGrow }, self)?
                }


                0xfc => {
                    let op = self.parse_u32()?;

                    use xfc::*;
                    match op {
                        MEMORY_COPY => {
                            self.reader.expect(0x00).ok().ok_or_else(|| unimplemented!())?;
                            self.reader.expect(0x00).ok().ok_or_else(|| unimplemented!())?;

                            f(Operand { source, data: OperandData::MemoryCopy }, self)?
                        }

                        MEMORY_FILL => {
                            self.reader.expect(0x00).ok().ok_or_else(|| unimplemented!())?;

                            f(Operand { source, data: OperandData::MemoryFill }, self)?
                        }

                        _ => {
                            println!("unknown opcode 0xfc 0x{op:x}");
                            unimplemented!()
                        }
                    }
                }

                _ => {
                    println!("unknown opcode 0x{opcode:x}");
                    unimplemented!()
                }
            };
            if done {
                break;
            }
        }

        return Ok(());
    }
}



#[derive(Clone, Copy, Debug)]
pub struct Section<'a> {
    pub kind:   SectionKind,
    pub offset: usize,
    pub data:   &'a [u8],
}

#[derive(Clone, Copy, Debug)]
pub enum SectionKind {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
    DataCount,
}

impl SectionKind {
    pub const MAX: u8 = 12;

    pub fn from_u8(value: u8) -> Option<SectionKind> {
        use SectionKind::*;
        Some(match value {
             0 => Custom,
             1 => Type,
             2 => Import,
             3 => Function,
             4 => Table,
             5 => Memory,
             6 => Global,
             7 => Export,
             8 => Start,
             9 => Element,
            10 => Code,
            11 => Data,
            12 => DataCount,

            _ => return None,
        })
    }
}


pub fn parse_limits(reader: &mut Reader<u8>) -> Result<Limits, ()> {
    let kind = reader.next().ok_or(())?;
    return Ok(match kind {
        0x00 => {
            let min = leb128::decode_u32(reader).ok().ok_or(())?;
            Limits { min, max: None }
        }

        0x01 => {
            let min = leb128::decode_u32(reader).ok().ok_or(())?;
            let max = leb128::decode_u32(reader).ok().ok_or(())?;
            Limits { min, max: Some(max) }
        }

        _ => return Err(())
    });
}

pub fn parse_value_type(reader: &mut Reader<u8>) -> Result<ValueType, ()> {
    ValueType::from_u8(reader.next().ok_or(())?).ok_or(())
}

pub fn parse_name<'a, A: Alloc + 'a>(reader: &mut Reader<u8>, alloc: A) -> Result<&'a str, ()> {
    let len = leb128::decode_u32(reader).ok().ok_or(())? as usize;
    let name = reader.next_n(len).ok_or(())?;

    // @todo: Vec::from_slice.
    let mut name_buf = Vec::with_cap_in(alloc, len);
    for byte in name {
        name_buf.push(*byte);
    }
    let name = Vec::leak(name_buf);

    return core::str::from_utf8(name).ok().ok_or(());
}

pub fn parse_const_expr(reader: &mut Reader<u8>) -> Result<ConstExpr, ()> {
    use opcode::*;

    let op = reader.next().unwrap();
    let result = match op {
        I32_CONST => {
            let value = leb128::decode_i32(reader).unwrap();
            ConstExpr::I32(value)
        }

        I64_CONST => {
            let value = leb128::decode_i64(reader).unwrap();
            ConstExpr::I64(value)
        }

        F32_CONST => {
            let value = reader.next_array::<4>().unwrap();
            let value = f32::from_le_bytes(value);
            ConstExpr::F32(value)
        }

        F64_CONST => {
            let value = reader.next_array::<8>().unwrap();
            let value = f64::from_le_bytes(value);
            ConstExpr::F64(value)
        }

        _ => {
            unimplemented!()
        }
    };

    let op = reader.next().unwrap();
    assert_eq!(op, END);

    return Ok(result);
}



pub fn parse_module_ex<'i, F: FnMut(Section<'i>)>(module: &'i [u8], mut f: F) -> Result<(), ()> {
    let mut reader = Reader::new(module);

    reader.expect_n(b"\0asm").ok().ok_or_else(|| unimplemented!())?;
    reader.expect_n(&[1, 0, 0, 0]).ok().ok_or_else(|| unimplemented!())?;

    // @todo: the data-count section messes this up,
    // so we need an explicit state machine (const array of `Section`s in order & inc index until at current section).
    let mut max_section_kind = 0;

    while let Some(kind) = reader.next() {
        // validate section order.
        if kind > 0 {
            if kind <= max_section_kind {
                // duplicate or out-of-order section.
                unimplemented!();
            }
            max_section_kind = kind;
        }

        let kind = SectionKind::from_u8(kind).unwrap();

        let len  = leb128::decode_u32(&mut reader).ok().ok_or_else(|| unimplemented!())? as usize;
        let data = reader.next_n(len).ok_or_else(|| unimplemented!())?;

        let offset = reader.as_ptr() as usize - module.as_ptr() as usize;

        f(Section { kind, offset, data });
    }

    Ok(())
}


pub struct CustomSection<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

pub struct ParseModuleResult<'i, 'm> {
    pub module: Module<'m>,
    pub custom: &'m [CustomSection<'i>],
    pub code:   &'i [u8],
}

pub fn parse_module<'i, 'm, A: Alloc + Copy + 'm>(wasm: &'i [u8], alloc: A) -> Result<ParseModuleResult<'i, 'm>, ()> {
    let mut module = Module::new();
    let mut custom = Vec::new_in(alloc);
    let mut code   = &[][..];

    parse_module_ex(wasm, |section| {
        let mut reader = Reader::new(section.data);

        match section.kind {
            SectionKind::Custom => {
                let name_len = leb128::decode_u32(&mut reader).unwrap() as usize;
                let name = core::str::from_utf8(reader.next_n(name_len).unwrap()).unwrap();
                custom.push(CustomSection { name, data: reader.as_slice() });
            }

            SectionKind::Type => {
                let num_types = leb128::decode_u32(&mut reader).unwrap() as usize;

                let mut types = Vec::with_cap_in(alloc, num_types);

                for _ in 0..num_types {
                    reader.expect(0x60).unwrap();

                    let num_params = leb128::decode_u32(&mut reader).unwrap() as usize;
                    let mut params = Vec::with_cap_in(alloc, num_params);
                    for _ in 0..num_params {
                        params.push(parse_value_type(&mut reader).unwrap());
                    }

                    let num_rets = leb128::decode_u32(&mut reader).unwrap() as usize;
                    let mut rets = Vec::with_cap_in(alloc, num_rets);
                    for _ in 0..num_rets {
                        rets.push(parse_value_type(&mut reader).unwrap());
                    }

                    types.push(FuncType { params: Vec::leak(params), rets: Vec::leak(rets) });
                }

                module.types = Vec::leak(types);
            }

            SectionKind::Import => {
                let num_imports = leb128::decode_u32(&mut reader).unwrap() as usize;

                let mut imports  = Vec::with_cap_in(alloc, num_imports);
                let mut funcs    = Vec::with_cap_in(alloc, num_imports);
                // let mut memories = Vec::new_in(alloc);
                // let mut globals  = Vec::new_in(alloc);
                // let mut tables   = Vec::new_in(alloc);
                for _ in 0..num_imports {
                    let module = parse_name(&mut reader, alloc).unwrap();
                    let name   = parse_name(&mut reader, alloc).unwrap();

                    let kind = reader.next().unwrap();
                    let (kind, index) = match kind {
                        0x00 => {
                            let index = funcs.len() as u32;
                            funcs.push(leb128::decode_u32(&mut reader).unwrap());
                            (ImportKind::Func, index)
                        }

                        _ => {
                            unimplemented!()
                        }
                    };

                    imports.push(Import { module, name, kind, index });
                }

                module.imports = Imports {
                    imports:  Vec::leak(imports),
                    funcs:    Vec::leak(funcs),
                    memories: &[],
                    globals:  &[],
                    tables:   &[],
                };
            }

            SectionKind::Function => {
                let num_funcs = leb128::decode_u32(&mut reader).unwrap() as usize;

                let mut funcs = Vec::with_cap_in(alloc, num_funcs);
                for _ in 0..num_funcs {
                    let type_index = leb128::decode_u32(&mut reader).unwrap();
                    funcs.push(type_index);
                }

                module.func_types = Vec::leak(funcs);
            }

            SectionKind::Table => {
                let num_tables = leb128::decode_u32(&mut reader).unwrap() as usize;

                let mut tables = Vec::with_cap_in(alloc, num_tables);
                for _ in 0..num_tables {
                    let ty = parse_value_type(&mut reader).unwrap();
                    assert!(ty.is_ref());

                    let limits = parse_limits(&mut reader).unwrap();

                    tables.push(TableType { ty, limits });
                }

                module.tables = Vec::leak(tables);
            }

            SectionKind::Memory => {
                let num_memories = leb128::decode_u32(&mut reader).unwrap() as usize;

                let mut memories = Vec::with_cap_in(alloc, num_memories);
                for _ in 0..num_memories {
                    memories.push(MemoryType { limits: parse_limits(&mut reader).unwrap() });
                }

                module.memories = Vec::leak(memories);
            }

            SectionKind::Global => {
                let num_globals = leb128::decode_u32(&mut reader).unwrap() as usize;

                let mut globals = Vec::with_cap_in(alloc, num_globals);

                for _ in 0..num_globals {
                    let ty = parse_value_type(&mut reader).unwrap();
                    let mutt = reader.next().unwrap();
                    assert!(mutt == 0x00 || mutt == 0x01);
                    let mutt = mutt == 0x01;

                    let init = parse_const_expr(&mut reader).unwrap();

                    let ty = GlobalType { ty, mutt };
                    globals.push(Global { ty, init });
                }

                module.globals = Vec::leak(globals);
            }

            SectionKind::Export => {
                let num_exports = leb128::decode_u32(&mut reader).unwrap() as usize;

                let mut exports = Vec::with_cap_in(alloc, num_exports);
                for _ in 0..num_exports {
                    let name = parse_name(&mut reader, alloc).unwrap();

                    let kind = reader.next().unwrap();
                    let data = match kind {
                        0x00 => {
                            ExportData::Func(leb128::decode_u32(&mut reader).unwrap())
                        }

                        0x01 => {
                            ExportData::Table(leb128::decode_u32(&mut reader).unwrap())
                        }

                        0x02 => {
                            ExportData::Memory(leb128::decode_u32(&mut reader).unwrap())
                        }

                        0x03 => {
                            ExportData::Global(leb128::decode_u32(&mut reader).unwrap())
                        }

                        _ => {
                            unimplemented!()
                        }
                    };

                    exports.push(Export { name, data });
                }

                module.exports = Vec::leak(exports);
            }

            SectionKind::Start => {
                unimplemented!()
            }

            SectionKind::Element => {
                let num_elems = leb128::decode_u32(&mut reader).unwrap() as usize;

                let mut elems = Vec::with_cap_in(alloc, num_elems);
                for _ in 0..num_elems {
                    let encoding = leb128::decode_u32(&mut reader).unwrap() as usize;

                    let elem = match encoding {
                        0 => {
                            let offset = parse_const_expr(&mut reader).unwrap();
                            let ConstExpr::I32(offset) = offset else { unimplemented!() };

                            let num_values = leb128::decode_u32(&mut reader).unwrap() as usize;

                            let mut values = Vec::with_cap_in(alloc, num_values);
                            for _ in 0..num_values {
                                let value = leb128::decode_u32(&mut reader).unwrap();
                                values.push(value);
                            }

                            let ty = ValueType::FuncRef;
                            let kind = ElemKind::Active { tab_idx: 0, offset: offset as u32 };
                            Elem { ty, kind, values: Vec::leak(values) }
                        }

                        _ => {
                            unimplemented!()
                        }
                    };

                    elems.push(elem);
                }

                module.elems = Vec::leak(elems);
            }

            SectionKind::Code => {
                code = section.data;

            }

            SectionKind::Data => {
                let num_datas = leb128::decode_u32(&mut reader).unwrap() as usize;

                let mut datas = Vec::with_cap_in(alloc, num_datas);
                for _ in 0..num_datas {
                    let encoding = leb128::decode_u32(&mut reader).unwrap() as usize;

                    let data = match encoding {
                        0 => {
                            let offset = parse_const_expr(&mut reader).unwrap();
                            let ConstExpr::I32(offset) = offset else { unimplemented!() };

                            let len = leb128::decode_u32(&mut reader).unwrap() as usize;
                            let bytes = reader.next_n(len).unwrap();

                            let mut values = Vec::with_cap_in(alloc, len);
                            unsafe {
                                core::ptr::copy_nonoverlapping(bytes.as_ptr(), values.as_mut_ptr(), len);
                                values.set_len(len);
                            }

                            let kind = DataKind::Active { mem_idx: 0, offset: offset as u32 };
                            Data { kind, values: Vec::leak(values) }
                        }

                        _ => {
                            unimplemented!()
                        }
                    };

                    datas.push(data);
                }

                module.datas = Vec::leak(datas);
            }

            SectionKind::DataCount => {}
        }
    })?;

    let custom = Vec::leak(custom);
    Ok(ParseModuleResult { module, custom, code })
}

