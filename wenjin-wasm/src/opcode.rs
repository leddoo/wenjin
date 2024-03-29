use crate::ValueType;


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParseResult {
    Opcode(Opcode),
    Prefix(Prefix),
    Error,
}

impl Opcode {
    #[inline(always)]
    pub fn parse(v: u8) -> ParseResult {
        PARSE[v as usize]
    }

    #[inline(always)]
    pub fn parse_prefixed(prefix: Prefix, v: u32) -> Option<Opcode> {
        parse_prefixed_core(prefix, v)
    }

    #[inline(always)]
    pub fn class(self) -> OpcodeClass {
        CLASS[self as usize]
    }
}


// generated:


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Opcode {
    Unreachable,
    Nop,
    Block,
    Loop,
    If,
    Else,
    End,
    Br,
    BrIf,
    BrTable,
    Return,
    Call,
    CallIndirect,
    Drop,
    Select,
    TypedSelect,
    LocalGet,
    LocalSet,
    LocalTee,
    GlobalGet,
    GlobalSet,
    TableGet,
    TableSet,
    I32Load,
    I64Load,
    F32Load,
    F64Load,
    I32Load8S,
    I32Load8U,
    I32Load16S,
    I32Load16U,
    I64Load8S,
    I64Load8U,
    I64Load16S,
    I64Load16U,
    I64Load32S,
    I64Load32U,
    I32Store,
    I64Store,
    F32Store,
    F64Store,
    I32Store8,
    I32Store16,
    I64Store8,
    I64Store16,
    I64Store32,
    MemorySize,
    MemoryGrow,
    I32Const,
    I64Const,
    F32Const,
    F64Const,
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,
    I32WrapI64,
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64ExtendI32S,
    I64ExtendI32U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,
    F32ConvertI32S,
    F32ConvertI32U,
    F32ConvertI64S,
    F32ConvertI64U,
    F32DemoteF64,
    F64ConvertI32S,
    F64ConvertI32U,
    F64ConvertI64S,
    F64ConvertI64U,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,
    I32Extend8S,
    I32Extend16S,
    I64Extend8S,
    I64Extend16S,
    I64Extend32S,
    RefNull,
    RefIsNull,
    RefFunc,
    MemoryCopy,
    MemoryFill,
}
const NUM_OPCODES: usize = 185;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Prefix {
    Xfc,
}
const PARSE: &[ParseResult; 256] = &[
    ParseResult::Opcode(Opcode::Unreachable),
    ParseResult::Opcode(Opcode::Nop),
    ParseResult::Opcode(Opcode::Block),
    ParseResult::Opcode(Opcode::Loop),
    ParseResult::Opcode(Opcode::If),
    ParseResult::Opcode(Opcode::Else),
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Opcode(Opcode::End),
    ParseResult::Opcode(Opcode::Br),
    ParseResult::Opcode(Opcode::BrIf),
    ParseResult::Opcode(Opcode::BrTable),
    ParseResult::Opcode(Opcode::Return),
    ParseResult::Opcode(Opcode::Call),
    ParseResult::Opcode(Opcode::CallIndirect),
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Opcode(Opcode::Drop),
    ParseResult::Opcode(Opcode::Select),
    ParseResult::Opcode(Opcode::TypedSelect),
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Opcode(Opcode::LocalGet),
    ParseResult::Opcode(Opcode::LocalSet),
    ParseResult::Opcode(Opcode::LocalTee),
    ParseResult::Opcode(Opcode::GlobalGet),
    ParseResult::Opcode(Opcode::GlobalSet),
    ParseResult::Opcode(Opcode::TableGet),
    ParseResult::Opcode(Opcode::TableSet),
    ParseResult::Error,
    ParseResult::Opcode(Opcode::I32Load),
    ParseResult::Opcode(Opcode::I64Load),
    ParseResult::Opcode(Opcode::F32Load),
    ParseResult::Opcode(Opcode::F64Load),
    ParseResult::Opcode(Opcode::I32Load8S),
    ParseResult::Opcode(Opcode::I32Load8U),
    ParseResult::Opcode(Opcode::I32Load16S),
    ParseResult::Opcode(Opcode::I32Load16U),
    ParseResult::Opcode(Opcode::I64Load8S),
    ParseResult::Opcode(Opcode::I64Load8U),
    ParseResult::Opcode(Opcode::I64Load16S),
    ParseResult::Opcode(Opcode::I64Load16U),
    ParseResult::Opcode(Opcode::I64Load32S),
    ParseResult::Opcode(Opcode::I64Load32U),
    ParseResult::Opcode(Opcode::I32Store),
    ParseResult::Opcode(Opcode::I64Store),
    ParseResult::Opcode(Opcode::F32Store),
    ParseResult::Opcode(Opcode::F64Store),
    ParseResult::Opcode(Opcode::I32Store8),
    ParseResult::Opcode(Opcode::I32Store16),
    ParseResult::Opcode(Opcode::I64Store8),
    ParseResult::Opcode(Opcode::I64Store16),
    ParseResult::Opcode(Opcode::I64Store32),
    ParseResult::Opcode(Opcode::MemorySize),
    ParseResult::Opcode(Opcode::MemoryGrow),
    ParseResult::Opcode(Opcode::I32Const),
    ParseResult::Opcode(Opcode::I64Const),
    ParseResult::Opcode(Opcode::F32Const),
    ParseResult::Opcode(Opcode::F64Const),
    ParseResult::Opcode(Opcode::I32Eqz),
    ParseResult::Opcode(Opcode::I32Eq),
    ParseResult::Opcode(Opcode::I32Ne),
    ParseResult::Opcode(Opcode::I32LtS),
    ParseResult::Opcode(Opcode::I32LtU),
    ParseResult::Opcode(Opcode::I32GtS),
    ParseResult::Opcode(Opcode::I32GtU),
    ParseResult::Opcode(Opcode::I32LeS),
    ParseResult::Opcode(Opcode::I32LeU),
    ParseResult::Opcode(Opcode::I32GeS),
    ParseResult::Opcode(Opcode::I32GeU),
    ParseResult::Opcode(Opcode::I64Eqz),
    ParseResult::Opcode(Opcode::I64Eq),
    ParseResult::Opcode(Opcode::I64Ne),
    ParseResult::Opcode(Opcode::I64LtS),
    ParseResult::Opcode(Opcode::I64LtU),
    ParseResult::Opcode(Opcode::I64GtS),
    ParseResult::Opcode(Opcode::I64GtU),
    ParseResult::Opcode(Opcode::I64LeS),
    ParseResult::Opcode(Opcode::I64LeU),
    ParseResult::Opcode(Opcode::I64GeS),
    ParseResult::Opcode(Opcode::I64GeU),
    ParseResult::Opcode(Opcode::F32Eq),
    ParseResult::Opcode(Opcode::F32Ne),
    ParseResult::Opcode(Opcode::F32Lt),
    ParseResult::Opcode(Opcode::F32Gt),
    ParseResult::Opcode(Opcode::F32Le),
    ParseResult::Opcode(Opcode::F32Ge),
    ParseResult::Opcode(Opcode::F64Eq),
    ParseResult::Opcode(Opcode::F64Ne),
    ParseResult::Opcode(Opcode::F64Lt),
    ParseResult::Opcode(Opcode::F64Gt),
    ParseResult::Opcode(Opcode::F64Le),
    ParseResult::Opcode(Opcode::F64Ge),
    ParseResult::Opcode(Opcode::I32Clz),
    ParseResult::Opcode(Opcode::I32Ctz),
    ParseResult::Opcode(Opcode::I32Popcnt),
    ParseResult::Opcode(Opcode::I32Add),
    ParseResult::Opcode(Opcode::I32Sub),
    ParseResult::Opcode(Opcode::I32Mul),
    ParseResult::Opcode(Opcode::I32DivS),
    ParseResult::Opcode(Opcode::I32DivU),
    ParseResult::Opcode(Opcode::I32RemS),
    ParseResult::Opcode(Opcode::I32RemU),
    ParseResult::Opcode(Opcode::I32And),
    ParseResult::Opcode(Opcode::I32Or),
    ParseResult::Opcode(Opcode::I32Xor),
    ParseResult::Opcode(Opcode::I32Shl),
    ParseResult::Opcode(Opcode::I32ShrS),
    ParseResult::Opcode(Opcode::I32ShrU),
    ParseResult::Opcode(Opcode::I32Rotl),
    ParseResult::Opcode(Opcode::I32Rotr),
    ParseResult::Opcode(Opcode::I64Clz),
    ParseResult::Opcode(Opcode::I64Ctz),
    ParseResult::Opcode(Opcode::I64Popcnt),
    ParseResult::Opcode(Opcode::I64Add),
    ParseResult::Opcode(Opcode::I64Sub),
    ParseResult::Opcode(Opcode::I64Mul),
    ParseResult::Opcode(Opcode::I64DivS),
    ParseResult::Opcode(Opcode::I64DivU),
    ParseResult::Opcode(Opcode::I64RemS),
    ParseResult::Opcode(Opcode::I64RemU),
    ParseResult::Opcode(Opcode::I64And),
    ParseResult::Opcode(Opcode::I64Or),
    ParseResult::Opcode(Opcode::I64Xor),
    ParseResult::Opcode(Opcode::I64Shl),
    ParseResult::Opcode(Opcode::I64ShrS),
    ParseResult::Opcode(Opcode::I64ShrU),
    ParseResult::Opcode(Opcode::I64Rotl),
    ParseResult::Opcode(Opcode::I64Rotr),
    ParseResult::Opcode(Opcode::F32Abs),
    ParseResult::Opcode(Opcode::F32Neg),
    ParseResult::Opcode(Opcode::F32Ceil),
    ParseResult::Opcode(Opcode::F32Floor),
    ParseResult::Opcode(Opcode::F32Trunc),
    ParseResult::Opcode(Opcode::F32Nearest),
    ParseResult::Opcode(Opcode::F32Sqrt),
    ParseResult::Opcode(Opcode::F32Add),
    ParseResult::Opcode(Opcode::F32Sub),
    ParseResult::Opcode(Opcode::F32Mul),
    ParseResult::Opcode(Opcode::F32Div),
    ParseResult::Opcode(Opcode::F32Min),
    ParseResult::Opcode(Opcode::F32Max),
    ParseResult::Opcode(Opcode::F32Copysign),
    ParseResult::Opcode(Opcode::F64Abs),
    ParseResult::Opcode(Opcode::F64Neg),
    ParseResult::Opcode(Opcode::F64Ceil),
    ParseResult::Opcode(Opcode::F64Floor),
    ParseResult::Opcode(Opcode::F64Trunc),
    ParseResult::Opcode(Opcode::F64Nearest),
    ParseResult::Opcode(Opcode::F64Sqrt),
    ParseResult::Opcode(Opcode::F64Add),
    ParseResult::Opcode(Opcode::F64Sub),
    ParseResult::Opcode(Opcode::F64Mul),
    ParseResult::Opcode(Opcode::F64Div),
    ParseResult::Opcode(Opcode::F64Min),
    ParseResult::Opcode(Opcode::F64Max),
    ParseResult::Opcode(Opcode::F64Copysign),
    ParseResult::Opcode(Opcode::I32WrapI64),
    ParseResult::Opcode(Opcode::I32TruncF32S),
    ParseResult::Opcode(Opcode::I32TruncF32U),
    ParseResult::Opcode(Opcode::I32TruncF64S),
    ParseResult::Opcode(Opcode::I32TruncF64U),
    ParseResult::Opcode(Opcode::I64ExtendI32S),
    ParseResult::Opcode(Opcode::I64ExtendI32U),
    ParseResult::Opcode(Opcode::I64TruncF32S),
    ParseResult::Opcode(Opcode::I64TruncF32U),
    ParseResult::Opcode(Opcode::I64TruncF64S),
    ParseResult::Opcode(Opcode::I64TruncF64U),
    ParseResult::Opcode(Opcode::F32ConvertI32S),
    ParseResult::Opcode(Opcode::F32ConvertI32U),
    ParseResult::Opcode(Opcode::F32ConvertI64S),
    ParseResult::Opcode(Opcode::F32ConvertI64U),
    ParseResult::Opcode(Opcode::F32DemoteF64),
    ParseResult::Opcode(Opcode::F64ConvertI32S),
    ParseResult::Opcode(Opcode::F64ConvertI32U),
    ParseResult::Opcode(Opcode::F64ConvertI64S),
    ParseResult::Opcode(Opcode::F64ConvertI64U),
    ParseResult::Opcode(Opcode::F64PromoteF32),
    ParseResult::Opcode(Opcode::I32ReinterpretF32),
    ParseResult::Opcode(Opcode::I64ReinterpretF64),
    ParseResult::Opcode(Opcode::F32ReinterpretI32),
    ParseResult::Opcode(Opcode::F64ReinterpretI64),
    ParseResult::Opcode(Opcode::I32Extend8S),
    ParseResult::Opcode(Opcode::I32Extend16S),
    ParseResult::Opcode(Opcode::I64Extend8S),
    ParseResult::Opcode(Opcode::I64Extend16S),
    ParseResult::Opcode(Opcode::I64Extend32S),
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Opcode(Opcode::RefNull),
    ParseResult::Opcode(Opcode::RefIsNull),
    ParseResult::Opcode(Opcode::RefFunc),
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Prefix(Prefix::Xfc),
    ParseResult::Error,
    ParseResult::Error,
    ParseResult::Error,
];
#[inline]
fn parse_xfc(v: u32) -> Option<Opcode> {
    Some(match v {
        10 => Opcode::MemoryCopy,
        11 => Opcode::MemoryFill,
        _ => return None
    })
}
#[inline]
fn parse_prefixed_core(prefix: Prefix, v: u32) -> Option<Opcode> {
    match prefix {
        Prefix::Xfc => parse_xfc(v),
    }
}
#[derive(Clone, Copy, Debug)]
pub enum OpcodeClass {
    Basic { pop: &'static [ValueType], push: &'static [ValueType] },
    Mem { max_align: u8, pop: &'static [ValueType], push: &'static [ValueType] },
    Unreachable,
    Block,
    Loop,
    If,
    Else,
    End,
    Br,
    BrIf,
    BrTable,
    Return,
    Call,
    CallIndirect,
    Drop,
    Select,
    TypedSelect,
    LocalGet,
    LocalSet,
    LocalTee,
    GlobalGet,
    GlobalSet,
    TableGet,
    TableSet,
    MemorySize,
    MemoryGrow,
    I32Const,
    I64Const,
    F32Const,
    F64Const,
    RefNull,
    RefIsNull,
    RefFunc,
    MemoryCopy,
    MemoryFill,
}
const CLASS: &[OpcodeClass; NUM_OPCODES] = &[
    OpcodeClass::Unreachable,
    OpcodeClass::Basic { pop: &[], push: &[] },
    OpcodeClass::Block,
    OpcodeClass::Loop,
    OpcodeClass::If,
    OpcodeClass::Else,
    OpcodeClass::End,
    OpcodeClass::Br,
    OpcodeClass::BrIf,
    OpcodeClass::BrTable,
    OpcodeClass::Return,
    OpcodeClass::Call,
    OpcodeClass::CallIndirect,
    OpcodeClass::Drop,
    OpcodeClass::Select,
    OpcodeClass::TypedSelect,
    OpcodeClass::LocalGet,
    OpcodeClass::LocalSet,
    OpcodeClass::LocalTee,
    OpcodeClass::GlobalGet,
    OpcodeClass::GlobalSet,
    OpcodeClass::TableGet,
    OpcodeClass::TableSet,
    OpcodeClass::Mem { max_align: 4, pop: &[ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Mem { max_align: 8, pop: &[ValueType::I32], push: &[ValueType::I64] },
    OpcodeClass::Mem { max_align: 4, pop: &[ValueType::I32], push: &[ValueType::F32] },
    OpcodeClass::Mem { max_align: 8, pop: &[ValueType::I32], push: &[ValueType::F64] },
    OpcodeClass::Mem { max_align: 1, pop: &[ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Mem { max_align: 1, pop: &[ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Mem { max_align: 2, pop: &[ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Mem { max_align: 2, pop: &[ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Mem { max_align: 1, pop: &[ValueType::I32], push: &[ValueType::I64] },
    OpcodeClass::Mem { max_align: 1, pop: &[ValueType::I32], push: &[ValueType::I64] },
    OpcodeClass::Mem { max_align: 2, pop: &[ValueType::I32], push: &[ValueType::I64] },
    OpcodeClass::Mem { max_align: 2, pop: &[ValueType::I32], push: &[ValueType::I64] },
    OpcodeClass::Mem { max_align: 4, pop: &[ValueType::I32], push: &[ValueType::I64] },
    OpcodeClass::Mem { max_align: 4, pop: &[ValueType::I32], push: &[ValueType::I64] },
    OpcodeClass::Mem { max_align: 4, pop: &[ValueType::I32,ValueType::I32], push: &[] },
    OpcodeClass::Mem { max_align: 8, pop: &[ValueType::I32,ValueType::I64], push: &[] },
    OpcodeClass::Mem { max_align: 4, pop: &[ValueType::I32,ValueType::F32], push: &[] },
    OpcodeClass::Mem { max_align: 8, pop: &[ValueType::I32,ValueType::F64], push: &[] },
    OpcodeClass::Mem { max_align: 1, pop: &[ValueType::I32,ValueType::I32], push: &[] },
    OpcodeClass::Mem { max_align: 2, pop: &[ValueType::I32,ValueType::I32], push: &[] },
    OpcodeClass::Mem { max_align: 1, pop: &[ValueType::I32,ValueType::I64], push: &[] },
    OpcodeClass::Mem { max_align: 2, pop: &[ValueType::I32,ValueType::I64], push: &[] },
    OpcodeClass::Mem { max_align: 4, pop: &[ValueType::I32,ValueType::I64], push: &[] },
    OpcodeClass::MemorySize,
    OpcodeClass::MemoryGrow,
    OpcodeClass::I32Const,
    OpcodeClass::I64Const,
    OpcodeClass::F32Const,
    OpcodeClass::F64Const,
    OpcodeClass::Basic { pop: &[ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F32,ValueType::F32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F32,ValueType::F32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F32,ValueType::F32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F32,ValueType::F32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F32,ValueType::F32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F32,ValueType::F32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F64,ValueType::F64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F64,ValueType::F64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F64,ValueType::F64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F64,ValueType::F64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F64,ValueType::F64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F64,ValueType::F64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32,ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64,ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F32,ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F32,ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F32,ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F32,ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F32,ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F32,ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F32,ValueType::F32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F64,ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F64,ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F64,ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F64,ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F64,ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F64,ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F64,ValueType::F64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::I64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F64], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I32], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::F32], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::F32], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::F64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::F64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::I32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::I64], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::I64], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::F64], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::I32], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::I32], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::I64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::I64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F32], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::F32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::F64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I32], push: &[ValueType::F32] },
    OpcodeClass::Basic { pop: &[ValueType::I64], push: &[ValueType::F64] },
    OpcodeClass::Basic { pop: &[ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I32], push: &[ValueType::I32] },
    OpcodeClass::Basic { pop: &[ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::Basic { pop: &[ValueType::I64], push: &[ValueType::I64] },
    OpcodeClass::RefNull,
    OpcodeClass::RefIsNull,
    OpcodeClass::RefFunc,
    OpcodeClass::MemoryCopy,
    OpcodeClass::MemoryFill,
];

