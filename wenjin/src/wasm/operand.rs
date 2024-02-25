use super::opcode;
use super::{ValueType, BlockType};


#[derive(Clone, Copy, Debug)]
pub struct Operand {
    pub source: u32,
    pub data: OperandData,
}

#[derive(Clone, Copy, Debug)]
pub enum OperandData {
    Unreachable,
    Nop,

    Block (BlockType),
    Loop  (BlockType),
    If    (BlockType),

    Else,
    End,

    Br      (u32),  // @todo: LabelIdx
    BrIf    (u32),  // @todo: LabelIdx
    BrTable,

    Return,
    Call          (u32),      // @todo: FuncIdx
    CallIndirect  (u32, u32), // @todo: TableIdx, TypeIdx

    Drop,
    Select,


    I32Const (i32),
    I64Const (i64),
    F32Const (f32),
    F64Const (f64),

    LocalGet  (u32), // @todo: LocalIdx
    LocalSet  (u32), // @todo: LocalIdx
    LocalTee  (u32), // @todo: LocalIdx
    GlobalGet (u32), // @todo: GlobalIdx
    GlobalSet (u32), // @todo: GlobalIdx

    Load  { load:  Load,  align: u32, offset: u32 },
    Store { store: Store, align: u32, offset: u32 },


    TestOp (TestOp),
    RelOp  (RelOp),
    Op1    (Op1),
    Op2    (Op2),

    Convert     (Convert),
    Reinterpret (Reinterpret),


    MemorySize,
    MemoryGrow,
    MemoryCopy,
    MemoryFill,
}

impl OperandData {
    #[inline]
    pub fn is_block(&self) -> bool {
        use OperandData::*;
        match self {
            Block (_) | Loop (_) | If (_) => true,

            _ => false,
        }
    }

    #[inline(always)]
    pub fn is_end(&self) -> bool {
        if let OperandData::End = self { true } else { false }
    }

    #[inline(always)]
    pub fn is_br_table(&self) -> bool {
        if let OperandData::BrTable = self { true } else { false }
    }
}



#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Load {
    I32,
    I32_8S,  I32_8U,
    I32_16S, I32_16U,

    I64,
    I64_8S,  I64_8U,
    I64_16S, I64_16U,
    I64_32S, I64_32U,

    F32,
    F64,
}

impl Load {
    pub fn from_opcode(opcode: u8) -> Load {
        use opcode::*;
        match opcode {
            I32_LOAD     => Load::I32,
            I32_LOAD8_S  => Load::I32_8S,
            I32_LOAD8_U  => Load::I32_8U,
            I32_LOAD16_S => Load::I32_16S,
            I32_LOAD16_U => Load::I32_16U,
            I64_LOAD     => Load::I64,
            I64_LOAD8_S  => Load::I64_8S,
            I64_LOAD8_U  => Load::I64_8U,
            I64_LOAD16_S => Load::I64_16S,
            I64_LOAD16_U => Load::I64_16U,
            I64_LOAD32_S => Load::I64_32S,
            I64_LOAD32_U => Load::I64_32U,
            F32_LOAD     => Load::F32,
            F64_LOAD     => Load::F64,

            _ => unreachable!()
        }
    }

    pub fn ty(self) -> ValueType {
        use Load::*;
        match self {
            I32 | I32_8S | I32_8U | I32_16S | I32_16U => ValueType::I32,

            I64 | I64_8S | I64_8U | I64_16S | I64_16U | I64_32S | I64_32U => ValueType::I64,

            F32 => ValueType::F32,

            F64 => ValueType::F64,
        }
    }
}


#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Store {
    I32, I32_8, I32_16,

    I64, I64_8, I64_16, I64_32,

    F32,
    F64,
}

impl Store {
    pub fn from_opcode(opcode: u8) -> Store {
        use opcode::*;
        match opcode {
            I32_STORE   => Store::I32,
            I32_STORE8  => Store::I32_8,
            I32_STORE16 => Store::I32_16,
            I64_STORE   => Store::I64,
            I64_STORE8  => Store::I64_8,
            I64_STORE16 => Store::I64_16,
            I64_STORE32 => Store::I64_32,
            F32_STORE   => Store::F32,
            F64_STORE   => Store::F64,

            _ => unreachable!()
        }
    }

    pub fn ty(self) -> ValueType {
        use Store::*;
        match self {
            I32 | I32_8 | I32_16 => ValueType::I32,

            I64 | I64_8 | I64_16 | I64_32 => ValueType::I64,

            F32 => ValueType::F32,

            F64 => ValueType::F64,
        }
    }
}



#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum TestOp {
    I32_EQZ,

    I64_EQZ,
}

impl TestOp {
    pub fn from_opcode(opcode: u8) -> TestOp {
        use opcode::*;
        match opcode {
            I32_EQZ => TestOp::I32_EQZ,
            I64_EQZ => TestOp::I64_EQZ,

            _ => unreachable!()
        }
    }

    pub fn ty(self) -> ValueType {
        use TestOp::*;
        match self {
            I32_EQZ => ValueType::I32,

            I64_EQZ => ValueType::I64,
        }
    }
}


#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum RelOp {
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
}

impl RelOp {
    pub fn from_opcode(opcode: u8) -> RelOp {
        use opcode::*;
        match opcode {
            I32_EQ   => RelOp::I32_EQ,
            I32_NE   => RelOp::I32_NE,
            I32_LE_S => RelOp::I32_LE_S,
            I32_LE_U => RelOp::I32_LE_U,
            I32_LT_S => RelOp::I32_LT_S,
            I32_LT_U => RelOp::I32_LT_U,
            I32_GE_S => RelOp::I32_GE_S,
            I32_GE_U => RelOp::I32_GE_U,
            I32_GT_S => RelOp::I32_GT_S,
            I32_GT_U => RelOp::I32_GT_U,
            I64_EQ   => RelOp::I64_EQ,
            I64_NE   => RelOp::I64_NE,
            I64_LE_S => RelOp::I64_LE_S,
            I64_LE_U => RelOp::I64_LE_U,
            I64_LT_S => RelOp::I64_LT_S,
            I64_LT_U => RelOp::I64_LT_U,
            I64_GE_S => RelOp::I64_GE_S,
            I64_GE_U => RelOp::I64_GE_U,
            I64_GT_S => RelOp::I64_GT_S,
            I64_GT_U => RelOp::I64_GT_U,
            F32_EQ   => RelOp::F32_EQ,
            F32_NE   => RelOp::F32_NE,
            F32_LE   => RelOp::F32_LE,
            F32_LT   => RelOp::F32_LT,
            F32_GE   => RelOp::F32_GE,
            F32_GT   => RelOp::F32_GT,
            F64_EQ   => RelOp::F64_EQ,
            F64_NE   => RelOp::F64_NE,
            F64_LE   => RelOp::F64_LE,
            F64_LT   => RelOp::F64_LT,
            F64_GE   => RelOp::F64_GE,
            F64_GT   => RelOp::F64_GT,

            _ => unreachable!()
        }
    }

    pub fn ty(self) -> ValueType {
        use RelOp::*;
        match self {
            I32_EQ   | I32_NE |
            I32_LE_S | I32_LE_U | I32_LT_S | I32_LT_U |
            I32_GE_S | I32_GE_U | I32_GT_S | I32_GT_U
            => ValueType::I32,

            I64_EQ | I64_NE |
            I64_LE_S | I64_LE_U | I64_LT_S | I64_LT_U |
            I64_GE_S | I64_GE_U | I64_GT_S | I64_GT_U
            => ValueType::I64,

            F32_EQ | F32_NE |
            F32_LE | F32_LT |
            F32_GE | F32_GT
            => ValueType::F32,

            F64_EQ | F64_NE |
            F64_LE | F64_LT |
            F64_GE | F64_GT
            => ValueType::F64,
        }
    }
}


#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Op1 {
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
}

impl Op1 {
    pub fn from_opcode(opcode: u8) -> Op1 {
        use opcode::*;
        match opcode {
            I32_CLZ        => Op1::I32_CLZ,
            I32_CTZ        => Op1::I32_CTZ,
            I32_POPCNT     => Op1::I32_POPCNT,
            I32_EXTEND8_S  => Op1::I32_EXTEND8_S,
            I32_EXTEND16_S => Op1::I32_EXTEND16_S,
            I64_CLZ        => Op1::I64_CLZ,
            I64_CTZ        => Op1::I64_CTZ,
            I64_POPCNT     => Op1::I64_POPCNT,
            I64_EXTEND8_S  => Op1::I64_EXTEND8_S,
            I64_EXTEND16_S => Op1::I64_EXTEND16_S,
            I64_EXTEND32_S => Op1::I64_EXTEND32_S,
            F32_ABS        => Op1::F32_ABS,
            F32_NEG        => Op1::F32_NEG,
            F32_CEIL       => Op1::F32_CEIL,
            F32_FLOOR      => Op1::F32_FLOOR,
            F32_TRUNC      => Op1::F32_TRUNC,
            F32_NEAREST    => Op1::F32_NEAREST,
            F32_SQRT       => Op1::F32_SQRT,
            F64_ABS        => Op1::F64_ABS,
            F64_NEG        => Op1::F64_NEG,
            F64_CEIL       => Op1::F64_CEIL,
            F64_FLOOR      => Op1::F64_FLOOR,
            F64_TRUNC      => Op1::F64_TRUNC,
            F64_NEAREST    => Op1::F64_NEAREST,
            F64_SQRT       => Op1::F64_SQRT,

            _ => unreachable!()
        }
    }

    pub fn ty(self) -> ValueType {
        use Op1::*;
        match self {
            I32_CLZ | I32_CTZ | I32_POPCNT |
            I32_EXTEND8_S | I32_EXTEND16_S
            => ValueType::I32,

            I64_CLZ | I64_CTZ | I64_POPCNT |
            I64_EXTEND8_S | I64_EXTEND16_S | I64_EXTEND32_S
            => ValueType::I64,

            F32_ABS | F32_NEG | F32_CEIL | F32_FLOOR |
            F32_TRUNC | F32_NEAREST | F32_SQRT
            => ValueType::F32,

            F64_ABS | F64_NEG | F64_CEIL | F64_FLOOR |
            F64_TRUNC | F64_NEAREST | F64_SQRT 
            => ValueType::F64,
        }
    }
}


#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Op2 {
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
    pub fn from_opcode(opcode: u8) -> Op2 {
        use opcode::*;
        match opcode {
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

            _ => unreachable!()
        }
    }

    pub fn ty(self) -> ValueType {
        use Op2::*;
        match self {
            I32_ADD | I32_SUB | I32_MUL | I32_DIV_S | I32_DIV_U | I32_REM_S | I32_REM_U |
            I32_AND | I32_OR | I32_XOR | I32_SHL | I32_SHR_S | I32_SHR_U | I32_ROTL | I32_ROTR
            => ValueType::I32,

            I64_ADD | I64_SUB | I64_MUL | I64_DIV_S | I64_DIV_U | I64_REM_S | I64_REM_U |
            I64_AND | I64_OR | I64_XOR | I64_SHL | I64_SHR_S | I64_SHR_U | I64_ROTL | I64_ROTR
            => ValueType::I64,

            F32_ADD | F32_SUB | F32_MUL | F32_DIV | F32_MIN | F32_MAX | F32_COPYSIGN
            => ValueType::F32,

            F64_ADD | F64_SUB | F64_MUL | F64_DIV | F64_MIN | F64_MAX | F64_COPYSIGN
            => ValueType::F64,
        }
    }
}



#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Convert {
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

impl Convert {
    pub fn from_opcode(opcode: u8) -> Convert {
        use opcode::*;
        match opcode {
            I32_WRAP_I64      => Convert::I32_WRAP_I64,
            I64_EXTEND_I32_S  => Convert::I64_EXTEND_I32_S,
            I64_EXTEND_I32_U  => Convert::I64_EXTEND_I32_U,
            I32_TRUNC_F32_S   => Convert::I32_TRUNC_F32_S,
            I32_TRUNC_F32_U   => Convert::I32_TRUNC_F32_U,
            F32_CONVERT_I32_S => Convert::F32_CONVERT_I32_S,
            F32_CONVERT_I32_U => Convert::F32_CONVERT_I32_U,
            I32_TRUNC_F64_S   => Convert::I32_TRUNC_F64_S,
            I32_TRUNC_F64_U   => Convert::I32_TRUNC_F64_U,
            F64_CONVERT_I32_S => Convert::F64_CONVERT_I32_S,
            F64_CONVERT_I32_U => Convert::F64_CONVERT_I32_U,
            I64_TRUNC_F32_S   => Convert::I64_TRUNC_F32_S,
            I64_TRUNC_F32_U   => Convert::I64_TRUNC_F32_U,
            F32_CONVERT_I64_S => Convert::F32_CONVERT_I64_S,
            F32_CONVERT_I64_U => Convert::F32_CONVERT_I64_U,
            I64_TRUNC_F64_S   => Convert::I64_TRUNC_F64_S,
            I64_TRUNC_F64_U   => Convert::I64_TRUNC_F64_U,
            F64_CONVERT_I64_S => Convert::F64_CONVERT_I64_S,
            F64_CONVERT_I64_U => Convert::F64_CONVERT_I64_U,
            F32_DEMOTE_F64    => Convert::F32_DEMOTE_F64,
            F64_PROMOTE_F32   => Convert::F64_PROMOTE_F32,

            _ => unreachable!()
        }
    }

    pub fn from_ty(self) -> ValueType {
        use Convert::*;
        match self {
            I64_EXTEND_I32_S  | I64_EXTEND_I32_U  |
            F32_CONVERT_I32_S | F32_CONVERT_I32_U |
            F64_CONVERT_I32_S | F64_CONVERT_I32_U
            => ValueType::I32,

            I32_WRAP_I64      |
            F32_CONVERT_I64_S | F32_CONVERT_I64_U |
            F64_CONVERT_I64_S | F64_CONVERT_I64_U
            => ValueType::I64,

            I32_TRUNC_F32_S | I32_TRUNC_F32_U |
            I64_TRUNC_F32_S | I64_TRUNC_F32_U |
            F64_PROMOTE_F32
            => ValueType::F32,

            I32_TRUNC_F64_S | I32_TRUNC_F64_U |
            I64_TRUNC_F64_S | I64_TRUNC_F64_U |
            F32_DEMOTE_F64
            => ValueType::F64,
        }
    }

    pub fn to_ty(self) -> ValueType {
        use Convert::*;
        match self {
            I32_WRAP_I64 |
            I32_TRUNC_F32_S | I32_TRUNC_F32_U |
            I32_TRUNC_F64_S | I32_TRUNC_F64_U
            => ValueType::I32,

            I64_EXTEND_I32_S | I64_EXTEND_I32_U |
            I64_TRUNC_F32_S  | I64_TRUNC_F32_U  |
            I64_TRUNC_F64_S  | I64_TRUNC_F64_U
            => ValueType::I64,

            F32_CONVERT_I32_S | F32_CONVERT_I32_U |
            F32_CONVERT_I64_S | F32_CONVERT_I64_U |
            F32_DEMOTE_F64
            => ValueType::F32,

            F64_CONVERT_I32_S | F64_CONVERT_I32_U |
            F64_CONVERT_I64_S | F64_CONVERT_I64_U |
            F64_PROMOTE_F32
            => ValueType::F64,
        }
    }
}


#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Reinterpret {
    I32_FROM_F32,
    F32_FROM_I32,
    I64_FROM_F64,
    F64_FROM_I64,
}

impl Reinterpret {
    pub fn from_opcode(opcode: u8) -> Reinterpret {
        use opcode::*;
        match opcode {
            I32_REINTERPRET_F32 => Reinterpret::I32_FROM_F32,
            F32_REINTERPRET_I32 => Reinterpret::F32_FROM_I32,
            I64_REINTERPRET_F64 => Reinterpret::I64_FROM_F64,
            F64_REINTERPRET_I64 => Reinterpret::F64_FROM_I64,

            _ => unreachable!()
        }
    }

    pub fn from_ty(self) -> ValueType {
        use Reinterpret::*;
        match self {
            F32_FROM_I32 => ValueType::I32,

            F64_FROM_I64 => ValueType::I64,

            I32_FROM_F32 => ValueType::F32,

            I64_FROM_F64 => ValueType::F64,
        }
    }

    pub fn to_ty(self) -> ValueType {
        use Reinterpret::*;
        match self {
            I32_FROM_F32 => ValueType::I32,

            I64_FROM_F64 => ValueType::I64,

            F32_FROM_I32 => ValueType::F32,

            F64_FROM_I64 => ValueType::F64,
        }
    }
}

