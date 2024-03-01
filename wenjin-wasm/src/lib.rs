pub mod leb128;

pub mod opcode;
pub mod operator;
pub mod parser;


pub const PAGE_SIZE: usize = 64*1024;


pub type TypeIdx = u32;
pub type FuncIdx = u32;
pub type GlobalIdx = u32;
pub type MemoryIdx = u32;
pub type TableIdx = u32;


#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}

impl ValueType {
    pub fn from_u8(value: u8) -> Option<ValueType> {
        Some(match value {
            0x7f => ValueType::I32,
            0x7e => ValueType::I64,
            0x7d => ValueType::F32,
            0x7c => ValueType::F64,

            0x7b => ValueType::V128,

            0x70 => ValueType::FuncRef,
            0x6f => ValueType::ExternRef,

            _ => return None,
        })
    }

    pub fn is_ref(self) -> bool {
        use ValueType::*;
        match self {
            FuncRef | ExternRef => true,

            I32 | I64 | F32 | F64 | V128 => false,
        }
    }

    pub fn as_slice(self) -> &'static [ValueType] {
        use ValueType::*;
        match self {
            I32       => &[I32],
            I64       => &[I64],
            F32       => &[F32],
            F64       => &[F64],
            V128      => &[V128],
            FuncRef   => &[FuncRef],
            ExternRef => &[ExternRef],
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockType {
    Unit,
    Value(ValueType),
    Func(TypeIdx),
}

impl BlockType {
    pub fn begin_types<'m>(self, module: &Module<'m>) -> &'m [ValueType] {
        match self {
            BlockType::Unit => &[],
            BlockType::Value(_) => &[],
            BlockType::Func(ty) => &module.types[ty as usize].params,
        }
    }

    pub fn end_types<'m>(self, module: &Module<'m>) -> &'m [ValueType] {
        match self {
            BlockType::Unit => &[],
            BlockType::Value(ty) => ty.as_slice(),
            BlockType::Func(ty) => &module.types[ty as usize].rets,
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FuncType<'a> {
    pub params: &'a [ValueType],
    pub rets: &'a [ValueType],
}


#[derive(Clone, Copy, Debug)]
pub struct TableType {
    pub ty: ValueType,
    pub limits: Limits,
}


#[derive(Clone, Copy, Debug)]
pub struct MemoryType {
    pub limits: Limits,
}


#[derive(Clone, Copy, Debug)]
pub struct GlobalType {
    pub ty: ValueType,
    pub mutt: bool,
}


#[derive(Clone, Copy, Debug, Default)]
pub struct Imports<'a> {
    pub imports: &'a [Import<'a>],

    pub funcs:    &'a [TypeIdx],
    pub memories: &'a [MemoryType],
    pub globals:  &'a [GlobalType],
    pub tables:   &'a [TableType],
}

#[derive(Clone, Copy, Debug)]
pub struct Import<'a> {
    pub module: &'a str,
    pub name: &'a str,
    pub kind: ImportKind,
    pub index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ImportKind {
    Func,
    Memory,
    Global,
    Table,
}


#[derive(Clone, Copy, Debug)]
pub enum ConstExpr {
    I32 (i32),
    I64 (i64),
    F32 (f32),
    F64 (f64),
}

#[derive(Clone, Copy, Debug)]
pub struct Global {
    pub ty: GlobalType,
    pub init: ConstExpr,
}


#[derive(Clone, Copy, Debug)]
pub struct Export<'a> {
    pub name: &'a str,
    pub data: ExportData,
}

#[derive(Clone, Copy, Debug)]
pub enum ExportData {
    Func(FuncIdx),
    Table(TableIdx),
    Memory(MemoryIdx),
    Global(GlobalIdx),
}


#[derive(Clone, Copy, Debug)]
pub struct Elem<'a> {
    pub ty: ValueType,
    pub kind: ElemKind,
    pub values: &'a [u32],
}

#[derive(Clone, Copy, Debug)]
pub enum ElemKind {
    Passive,
    Active { tab_idx: u32, offset: u32 },
    Declarative,
}


#[derive(Clone, Copy, Debug)]
pub struct Data<'a> {
    pub kind: DataKind,
    pub values: &'a [u8],
}

#[derive(Clone, Copy, Debug)]
pub enum DataKind {
    Passive,
    Active { mem_idx: u32, offset: u32 },
}


#[derive(Clone, Debug, Default)]
pub struct Module<'a> {
    pub imports: Imports<'a>,

    pub start: Option<FuncIdx>,

    pub types:    &'a [FuncType<'a>],
    pub funcs:    &'a [TypeIdx],
    pub tables:   &'a [TableType],
    pub memories: &'a [MemoryType],
    pub globals:  &'a [Global],
    pub exports:  &'a [Export<'a>],
    pub elems:    &'a [Elem<'a>],
    pub datas:    &'a [Data<'a>],
}


