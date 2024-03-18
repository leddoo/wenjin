pub mod leb128;

pub mod opcode;
mod operator;
mod parser;
mod validator;

pub use operator::{Operator, OperatorVisitor, MkOperator, AndThenOp};
pub use parser::{Parser, ParseError, ParseErrorKind, ParseModuleError};
pub use validator::{Validator, ValidatorError};


pub const PAGE_SIZE: usize = 64*1024;
pub const PAGE_SIZE32: u32 = 64*1024;


pub type TypeIdx = u32;
pub type FuncIdx = u32;
pub type TableIdx = u32;
pub type MemoryIdx = u32;
pub type GlobalIdx = u32;


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
pub enum RefType {
    FuncRef,
    ExternRef,
}

impl RefType {
    pub fn from_u8(value: u8) -> Option<RefType> {
        Some(match value {
            0x70 => RefType::FuncRef,
            0x6f => RefType::ExternRef,

            _ => return None,
        })
    }

    pub fn to_value_type(self) -> ValueType {
        match self {
            RefType::FuncRef => ValueType::FuncRef,
            RefType::ExternRef => ValueType::ExternRef,
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
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

impl Default for Limits {
    #[inline]
    fn default() -> Self {
        Limits { min: 0, max: None }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct TableType {
    pub ty: RefType,
    pub limits: Limits,
}


#[derive(Clone, Copy, Debug)]
pub struct MemoryType {
    pub limits: Limits,
}


#[derive(Clone, Copy, Debug)]
pub struct GlobalType {
    pub ty: ValueType,
    pub mutable: bool,
}


#[derive(Clone, Copy, Debug)]
pub struct Import<'a> {
    pub module: &'a str,
    pub name: &'a str,
    pub kind: ImportKind,
}

#[derive(Clone, Copy, Debug)]
pub enum ImportKind {
    Func(TypeIdx),
    Table(TableType),
    Memory(MemoryType),
    Global(GlobalType),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Imports<'a> {
    pub imports: &'a [Import<'a>],

    pub funcs:    &'a [TypeIdx],
    pub tables:   &'a [TableType],
    pub memories: &'a [MemoryType],
    pub globals:  &'a [GlobalType],
}



#[derive(Clone, Copy, Debug)]
pub enum ConstExpr {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Global(GlobalIdx),
    RefNull(RefType),
}

#[derive(Clone, Copy, Debug)]
pub struct Global {
    pub ty: GlobalType,
    pub init: ConstExpr,
}


#[derive(Clone, Copy, Debug)]
pub struct Export<'a> {
    pub name: &'a str,
    pub kind: ExportKind,
}

#[derive(Clone, Copy, Debug)]
pub enum ExportKind {
    Func(FuncIdx),
    Table(TableIdx),
    Memory(MemoryIdx),
    Global(GlobalIdx),
}


#[derive(Clone, Copy, Debug)]
pub struct Element<'a> {
    pub ty: RefType,
    pub kind: ElementKind,
    pub values: &'a [u32],
}

#[derive(Clone, Copy, Debug)]
pub enum ElementKind {
    Passive,
    Active { table: TableIdx, offset: u32 },
    Declarative,
}


#[derive(Clone, Copy, Debug)]
pub struct Code<'a> {
    pub locals: &'a [ValueType],
    pub expr: SubSection,
}

#[derive(Clone, Copy, Debug)]
pub struct BrTable<'a> {
    pub num_labels: u32,
    pub labels: &'a [u8],
    pub default: u32,
}

impl<'a> BrTable<'a> {
    pub fn labels(&self) -> U32Iter<'a> {
        U32Iter { values: sti::reader::Reader::new(self.labels) }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Data<'a> {
    pub kind: DataKind,
    pub values: &'a [u8],
}

#[derive(Clone, Copy, Debug)]
pub enum DataKind {
    Passive,
    Active { mem: MemoryIdx, offset: u32 },
}



#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubSection {
    pub offset: usize,
    pub len: usize,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Section {
    pub kind: SectionKind,
    pub sub: SubSection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub const COUNT: usize = 13;

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

pub struct CustomSection<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

impl<'a> core::fmt::Debug for CustomSection<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        writeln!(f, "CustomSection({:?})", self.name)
    }
}


#[derive(Clone, Copy, Debug)]
pub struct ModuleLimits {
    pub max_types:    u32,
    pub max_imports:  u32,
    pub max_funcs:    u32,
    pub max_tables:   u32,
    pub max_memories: u32,
    pub max_globals:  u32,
    pub max_exports:  u32,
    pub max_elements: u32,
    pub max_locals:   u32,
    pub max_datas:    u32,
    pub max_customs:  u32,
}

impl ModuleLimits {
    pub const DEFAULT: ModuleLimits = ModuleLimits {
        max_types:    1024,
        max_imports:   512,
        max_funcs:    4096,
        max_tables:    128,
        max_memories:   32,
        max_globals:   128,
        max_exports:  4096,
        max_elements:  128,
        max_locals:    256,
        max_datas:     512,
        max_customs:   512,
    };

    pub const UNLIMITED: ModuleLimits = ModuleLimits {
        max_types:    u32::MAX,
        max_imports:  u32::MAX,
        max_funcs:    u32::MAX,
        max_tables:   u32::MAX,
        max_memories: u32::MAX,
        max_globals:  u32::MAX,
        max_exports:  u32::MAX,
        max_elements: u32::MAX,
        max_locals:   u32::MAX,
        max_datas:    u32::MAX,
        max_customs:  u32::MAX,
    };
}

impl Default for ModuleLimits {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}


#[derive(Clone, Debug, Default)]
pub struct Module<'a> {
    pub types:      &'a [FuncType<'a>],
    pub imports:    Imports<'a>,
    pub funcs:      &'a [TypeIdx],
    pub tables:     &'a [TableType],
    pub memories:   &'a [MemoryType],
    pub globals:    &'a [Global],
    pub exports:    &'a [Export<'a>],
    pub start:      Option<FuncIdx>,
    pub elements:   &'a [Element<'a>],
    pub codes:      &'a [Code<'a>],
    pub datas:      &'a [Data<'a>],
    pub customs:    &'a [CustomSection<'a>],
}

impl<'a> Module<'a> {
    #[inline]
    pub fn get_func(&self, idx: FuncIdx) -> Option<TypeIdx> {
        let idx = idx as usize;
        let imports = self.imports.funcs;
        match imports.get(idx).copied() {
            Some(x) => Some(x),
            None => self.funcs.get(idx - imports.len()).copied()
        }
    }

    #[inline]
    pub fn get_table(&self, idx: TableIdx) -> Option<TableType> {
        let idx = idx as usize;
        let imports = self.imports.tables;
        match imports.get(idx).copied() {
            Some(x) => Some(x),
            None => self.tables.get(idx - imports.len()).copied()
        }
    }

    #[inline]
    pub fn get_memory(&self, idx: MemoryIdx) -> Option<MemoryType> {
        let idx = idx as usize;
        let imports = self.imports.memories;
        match imports.get(idx).copied() {
            Some(x) => Some(x),
            None => self.memories.get(idx - imports.len()).copied()
        }
    }

    #[inline]
    pub fn get_global(&self, idx: GlobalIdx) -> Option<GlobalType> {
        let idx = idx as usize;
        let imports = self.imports.globals;
        match imports.get(idx).copied() {
            Some(x) => Some(x),
            None => match self.globals.get(idx - imports.len()) {
                Some(x) => Some(x.ty),
                None => None
            }
        }
    }
}



pub struct U32Iter<'a> {
    values: sti::reader::Reader<'a, u8>,
}

impl<'a> Iterator for U32Iter<'a> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.values.len() == 0 {
            return None;
        }
        return Some(leb128::decode_u32(&mut self.values).unwrap());
    }
}


