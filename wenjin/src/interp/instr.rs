use crate::wasm;
use wasm::operand::{Load, Store};

use super::InterpFuncId;



sti::define_key!(pub(crate), u32, Label);
sti::define_key!(pub(crate), u32, JumpTable);
sti::define_key!(pub(crate), u8,  Local, opt: OptLocal);


/*
#[derive(Clone, Copy, Debug)]
pub struct Instr {
    pub data: InstrData,
}
*/

#[derive(Clone, Copy, Debug)]
pub(crate) enum InstrData {
    #[allow(dead_code)] // @temp
    Unimplemented { pop: u8, push: u8 },

    Unreachable,
    Nop,

    Label     { id: Label },
    Jump      { target: Label },
    JumpFalse { src: OptLocal, target: Label },
    JumpTrue  { src: OptLocal, target: Label },
    JumpTable { src: OptLocal, table: JumpTable },

    Return       { num_rets: u8 },
    CallIndirect { num_args: u8, num_rets: u8, index: u32 },
    CallBytecode { num_args: u8, num_rets: u8, func: InterpFuncId },
    CallTable    { num_args: u8, num_rets: u8, src: OptLocal, tab_idx: u32 },

    Drop,
    Select { dst: OptLocal, src1: OptLocal, src2: OptLocal, cond: OptLocal },

    I32Const { dst: OptLocal, value: i32 },
    I64Const { dst: OptLocal, value: i64 },
    F32Const { dst: OptLocal, value: f32 },
    F64Const { dst: OptLocal, value: f64 },

    LocalGet  { local: Local, version: u32 },
    LocalSet  { local: Local, src: OptLocal },
    LocalCopy { local: Local, src: OptLocal },

    GlobalGet { dst: OptLocal, global: u32 },
    GlobalSet { global: u32, src: OptLocal },

    Load  { dst: OptLocal, addr: OptLocal, load: Load, offset: u32 },
    Store { addr: OptLocal, src: OptLocal, store: Store, offset: u32 },

    Op1 { dst: OptLocal, src: OptLocal, op: Op1 },
    Op2 { dst: OptLocal, src1: OptLocal, src2: OptLocal, op: Op2 },

    MemorySize { dst: OptLocal },
    MemoryGrow { dst: OptLocal, delta: OptLocal },
    MemoryCopy { dst_addr: OptLocal, src_addr: OptLocal, len: OptLocal },
    MemoryFill { dst_addr: OptLocal, val: OptLocal, len: OptLocal },
}

impl InstrData {
    #[inline]
    pub fn dst(&mut self) -> Option<&mut OptLocal> {
        use InstrData::*;
        match self {
            Select { dst, src1: _, src2: _, cond: _ } |
            I32Const { dst, .. } | I64Const { dst, .. } | F32Const { dst, .. } | F64Const { dst, .. } |
            GlobalGet { dst, .. } |
            Load  { dst, .. } |
            Op1 { dst, .. } | Op2 { dst, .. } |
            MemorySize { dst } | MemoryGrow { dst, .. } 
            => Some(dst),

            Unimplemented {..} |
            Unreachable | Nop |
            Label {..} | Jump {..} | JumpFalse {..} | JumpTrue  {..} | JumpTable {..} |
            Return {..} | CallIndirect {..} | CallBytecode {..} | CallTable {..} |
            Drop |
            LocalGet  {..} | LocalSet  {..} | LocalCopy {..} |
            GlobalSet {..} |
            Store {..} |
            MemoryCopy {..} | MemoryFill {..}
            => None,
        }
    }
}


#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Op1 {
    I32_EQZ,

    I64_EQZ,

    I32_CLZ,
    I32_CTZ,
    I32_POPCNT,
    I32_EXTEND8_S,
    I32_EXTEND16_S,

    I64_CLZ,
    I64_CTZ,
    I64_POPCNT,
    I64_EXTEND8_S,
    I64_EXTEND16_S,
    I64_EXTEND32_S,

    F32_ABS,
    F32_NEG,
    F32_CEIL,  F32_FLOOR,
    F32_TRUNC, F32_NEAREST,
    F32_SQRT,

    F64_ABS,
    F64_NEG,
    F64_CEIL,  F64_FLOOR,
    F64_TRUNC, F64_NEAREST,
    F64_SQRT,

    I32_WRAP_I64,

    I64_EXTEND_I32_S, I64_EXTEND_I32_U,

    I32_TRUNC_F32_S,   I32_TRUNC_F32_U,
    F32_CONVERT_I32_S, F32_CONVERT_I32_U,

    I32_TRUNC_F64_S,   I32_TRUNC_F64_U,
    F64_CONVERT_I32_S, F64_CONVERT_I32_U,

    I64_TRUNC_F32_S,   I64_TRUNC_F32_U,
    F32_CONVERT_I64_S, F32_CONVERT_I64_U,

    I64_TRUNC_F64_S,   I64_TRUNC_F64_U,
    F64_CONVERT_I64_S, F64_CONVERT_I64_U,

    F32_DEMOTE_F64,
    F64_PROMOTE_F32,
}

impl Op1 {
    pub fn from_wasm_test_op(op: wasm::operand::TestOp) -> Op1 {
        use wasm::operand::TestOp::*;
        match op {
            I32_EQZ => Op1::I32_EQZ,
            I64_EQZ => Op1::I64_EQZ,
        }
    }

    pub fn from_wasm_op1(op: wasm::operand::Op1) -> Op1 {
        use wasm::operand::Op1::*;
        match op {
            I32_CLZ         => Op1::I32_CLZ,
            I32_CTZ         => Op1::I32_CTZ,
            I32_POPCNT      => Op1::I32_POPCNT,
            I32_EXTEND8_S   => Op1::I32_EXTEND8_S,
            I32_EXTEND16_S  => Op1::I32_EXTEND16_S,
            I64_CLZ         => Op1::I64_CLZ,
            I64_CTZ         => Op1::I64_CTZ,
            I64_POPCNT      => Op1::I64_POPCNT,
            I64_EXTEND8_S   => Op1::I64_EXTEND8_S,
            I64_EXTEND16_S  => Op1::I64_EXTEND16_S,
            I64_EXTEND32_S  => Op1::I64_EXTEND32_S,
            F32_ABS         => Op1::F32_ABS,
            F32_NEG         => Op1::F32_NEG,
            F32_CEIL        => Op1::F32_CEIL,
            F32_FLOOR       => Op1::F32_FLOOR,
            F32_TRUNC       => Op1::F32_TRUNC,
            F32_NEAREST     => Op1::F32_NEAREST,
            F32_SQRT        => Op1::F32_SQRT,
            F64_ABS         => Op1::F64_ABS,
            F64_NEG         => Op1::F64_NEG,
            F64_CEIL        => Op1::F64_CEIL,
            F64_FLOOR       => Op1::F64_FLOOR,
            F64_TRUNC       => Op1::F64_TRUNC,
            F64_NEAREST     => Op1::F64_NEAREST,
            F64_SQRT        => Op1::F64_SQRT,
        }
    }

    pub fn from_wasm_convert(cvt: wasm::operand::Convert) -> Op1 {
        use wasm::operand::Convert::*;
        match cvt {
            I32_WRAP_I64        => Op1::I32_WRAP_I64,
            I64_EXTEND_I32_S    => Op1::I64_EXTEND_I32_S,
            I64_EXTEND_I32_U    => Op1::I64_EXTEND_I32_U,
            I32_TRUNC_F32_S     => Op1::I32_TRUNC_F32_S,
            I32_TRUNC_F32_U     => Op1::I32_TRUNC_F32_U,
            F32_CONVERT_I32_S   => Op1::F32_CONVERT_I32_S,
            F32_CONVERT_I32_U   => Op1::F32_CONVERT_I32_U,
            I32_TRUNC_F64_S     => Op1::I32_TRUNC_F64_S,
            I32_TRUNC_F64_U     => Op1::I32_TRUNC_F64_U,
            F64_CONVERT_I32_S   => Op1::F64_CONVERT_I32_S,
            F64_CONVERT_I32_U   => Op1::F64_CONVERT_I32_U,
            I64_TRUNC_F32_S     => Op1::I64_TRUNC_F32_S,
            I64_TRUNC_F32_U     => Op1::I64_TRUNC_F32_U,
            F32_CONVERT_I64_S   => Op1::F32_CONVERT_I64_S,
            F32_CONVERT_I64_U   => Op1::F32_CONVERT_I64_U,
            I64_TRUNC_F64_S     => Op1::I64_TRUNC_F64_S,
            I64_TRUNC_F64_U     => Op1::I64_TRUNC_F64_U,
            F64_CONVERT_I64_S   => Op1::F64_CONVERT_I64_S,
            F64_CONVERT_I64_U   => Op1::F64_CONVERT_I64_U,
            F32_DEMOTE_F64      => Op1::F32_DEMOTE_F64,
            F64_PROMOTE_F32     => Op1::F64_PROMOTE_F32,
        }
    }
}

impl From<wasm::operand::TestOp>  for Op1 { #[inline(always)] fn from(value: wasm::operand::TestOp)  -> Self { Self::from_wasm_test_op(value) } }
impl From<wasm::operand::Op1>     for Op1 { #[inline(always)] fn from(value: wasm::operand::Op1)     -> Self { Self::from_wasm_op1    (value) } }
impl From<wasm::operand::Convert> for Op1 { #[inline(always)] fn from(value: wasm::operand::Convert) -> Self { Self::from_wasm_convert(value) } }


#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Op2 {
    I32_EQ, I32_NE,
    I32_LE_S, I32_LE_U, I32_LT_S, I32_LT_U,
    I32_GE_S, I32_GE_U, I32_GT_S, I32_GT_U,

    I64_EQ, I64_NE,
    I64_LE_S, I64_LE_U, I64_LT_S, I64_LT_U,
    I64_GE_S, I64_GE_U, I64_GT_S, I64_GT_U,

    F32_EQ, F32_NE,
    F32_LE, F32_LT,
    F32_GE, F32_GT,

    F64_EQ, F64_NE,
    F64_LE, F64_LT,
    F64_GE, F64_GT,

    I32_ADD,
    I32_SUB,
    I32_MUL,
    I32_DIV_S, I32_DIV_U,
    I32_REM_S, I32_REM_U,
    I32_AND,
    I32_OR,
    I32_XOR,
    I32_SHL,
    I32_SHR_S, I32_SHR_U,
    I32_ROTL,  I32_ROTR,

    I64_ADD,
    I64_SUB,
    I64_MUL,
    I64_DIV_S, I64_DIV_U,
    I64_REM_S, I64_REM_U,
    I64_AND,
    I64_OR,
    I64_XOR,
    I64_SHL,
    I64_SHR_S, I64_SHR_U,
    I64_ROTL,  I64_ROTR,

    F32_ADD, F32_SUB,
    F32_MUL, F32_DIV,
    F32_MIN, F32_MAX,
    F32_COPYSIGN,

    F64_ADD, F64_SUB,
    F64_MUL, F64_DIV,
    F64_MIN, F64_MAX,
    F64_COPYSIGN,
}

impl Op2 {
    pub fn from_wasm_rel_op(op: wasm::operand::RelOp) -> Op2 {
        use wasm::operand::RelOp::*;
        match op {
            I32_EQ      => Op2::I32_EQ,
            I32_NE      => Op2::I32_NE,
            I32_LE_S    => Op2::I32_LE_S,
            I32_LE_U    => Op2::I32_LE_U,
            I32_LT_S    => Op2::I32_LT_S,
            I32_LT_U    => Op2::I32_LT_U,
            I32_GE_S    => Op2::I32_GE_S,
            I32_GE_U    => Op2::I32_GE_U,
            I32_GT_S    => Op2::I32_GT_S,
            I32_GT_U    => Op2::I32_GT_U,
            I64_EQ      => Op2::I64_EQ,
            I64_NE      => Op2::I64_NE,
            I64_LE_S    => Op2::I64_LE_S,
            I64_LE_U    => Op2::I64_LE_U,
            I64_LT_S    => Op2::I64_LT_S,
            I64_LT_U    => Op2::I64_LT_U,
            I64_GE_S    => Op2::I64_GE_S,
            I64_GE_U    => Op2::I64_GE_U,
            I64_GT_S    => Op2::I64_GT_S,
            I64_GT_U    => Op2::I64_GT_U,
            F32_EQ      => Op2::F32_EQ,
            F32_NE      => Op2::F32_NE,
            F32_LE      => Op2::F32_LE,
            F32_LT      => Op2::F32_LT,
            F32_GE      => Op2::F32_GE,
            F32_GT      => Op2::F32_GT,
            F64_EQ      => Op2::F64_EQ,
            F64_NE      => Op2::F64_NE,
            F64_LE      => Op2::F64_LE,
            F64_LT      => Op2::F64_LT,
            F64_GE      => Op2::F64_GE,
            F64_GT      => Op2::F64_GT,
        }
    }

    pub fn from_wasm_op2(op: wasm::operand::Op2) -> Op2 {
        use wasm::operand::Op2::*;
        match op {
            I32_ADD      => Op2::I32_ADD,
            I32_SUB      => Op2::I32_SUB,
            I32_MUL      => Op2::I32_MUL,
            I32_DIV_S    => Op2::I32_DIV_S,
            I32_DIV_U    => Op2::I32_DIV_U,
            I32_REM_S    => Op2::I32_REM_S,
            I32_REM_U    => Op2::I32_REM_U,
            I32_AND      => Op2::I32_AND,
            I32_OR       => Op2::I32_OR,
            I32_XOR      => Op2::I32_XOR,
            I32_SHL      => Op2::I32_SHL,
            I32_SHR_S    => Op2::I32_SHR_S,
            I32_SHR_U    => Op2::I32_SHR_U,
            I32_ROTL     => Op2::I32_ROTL,
            I32_ROTR     => Op2::I32_ROTR,
            I64_ADD      => Op2::I64_ADD,
            I64_SUB      => Op2::I64_SUB,
            I64_MUL      => Op2::I64_MUL,
            I64_DIV_S    => Op2::I64_DIV_S,
            I64_DIV_U    => Op2::I64_DIV_U,
            I64_REM_S    => Op2::I64_REM_S,
            I64_REM_U    => Op2::I64_REM_U,
            I64_AND      => Op2::I64_AND,
            I64_OR       => Op2::I64_OR,
            I64_XOR      => Op2::I64_XOR,
            I64_SHL      => Op2::I64_SHL,
            I64_SHR_S    => Op2::I64_SHR_S,
            I64_SHR_U    => Op2::I64_SHR_U,
            I64_ROTL     => Op2::I64_ROTL,
            I64_ROTR     => Op2::I64_ROTR,
            F32_ADD      => Op2::F32_ADD,
            F32_SUB      => Op2::F32_SUB,
            F32_MUL      => Op2::F32_MUL,
            F32_DIV      => Op2::F32_DIV,
            F32_MIN      => Op2::F32_MIN,
            F32_MAX      => Op2::F32_MAX,
            F32_COPYSIGN => Op2::F32_COPYSIGN,
            F64_ADD      => Op2::F64_ADD,
            F64_SUB      => Op2::F64_SUB,
            F64_MUL      => Op2::F64_MUL,
            F64_DIV      => Op2::F64_DIV,
            F64_MIN      => Op2::F64_MIN,
            F64_MAX      => Op2::F64_MAX,
            F64_COPYSIGN => Op2::F64_COPYSIGN,
        }
    }
}

impl From<wasm::operand::RelOp> for Op2 { #[inline(always)] fn from(value: wasm::operand::RelOp) -> Self { Self::from_wasm_rel_op(value) } }
impl From<wasm::operand::Op2>   for Op2 { #[inline(always)] fn from(value: wasm::operand::Op2)   -> Self { Self::from_wasm_op2   (value) } }

