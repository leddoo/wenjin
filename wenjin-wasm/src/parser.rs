use sti::reader::Reader;

use crate::{BlockType, leb128};
use crate::operator::OperatorVisitor;


pub struct ParseError {
    pub offset: usize,
    pub kind: ParseErrorKind,
}

pub enum ParseErrorKind {
}

type Result<T> = core::result::Result<T, ParseError>;


pub struct Parser<'a> {
    reader: Reader<'a, u8>,
    input_begin: *const u8,
}

impl<'a> Parser<'a> {
    fn next(&mut self) -> Result<u8> {
        return self.reader.next().ok_or_else(||
            todo!());
    }

    fn parse_u32(&mut self) -> Result<u32> {
        leb128::decode_u32(&mut self.reader).map_err(|_|
            todo!())
    }

    fn parse_block_type(&mut self) -> Result<BlockType> {
        todo!()
    }

    fn parse_operator<V: OperatorVisitor>(&mut self, mut v: V) -> Result<V::Output> {
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
            BR_TABLE        => todo!(),
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
            MEMORY_GROW     => v.visit_memory_size(self.parse_u32()?),
            I32_CONST       => todo!(),
            I64_CONST       => todo!(),
            F32_CONST       => todo!(),
            F64_CONST       => todo!(),
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
}


