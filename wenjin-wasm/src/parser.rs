use sti::reader::Reader;
use sti::arena::Arena;
use sti::vec::Vec;

use crate::{leb128, BrTable, Error, ErrorKind, Result, TypeIdx, FuncIdx, TableIdx, MemoryIdx, GlobalIdx};
use crate::{ValueType, RefType, FuncType, BlockType, Limits, TableType, MemoryType, GlobalType};
use crate::{Import, ImportKind, Imports, Global, Export, ExportKind, Element, ElementKind, Code, Data, DataKind};
use crate::{SubSection, Section, SectionKind, CustomSection};
use crate::ConstExpr;
use crate::{ModuleLimits, Module};
use crate::opcode::{self, Opcode};


impl From<leb128::Leb128Error> for ErrorKind {
    #[inline]
    fn from(value: leb128::Leb128Error) -> Self {
        match value {
            leb128::Leb128Error::Overflow => ErrorKind::Leb128Overflow,
            leb128::Leb128Error::EOI => ErrorKind::UnexpectedEof,
        }
    }
}


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
    pub fn is_done(&self) -> bool {
        self.reader.is_empty()
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
        return self.reader.next()
            .ok_or_else(|| self.error(ErrorKind::UnexpectedEof));
    }

    #[inline]
    pub fn parse_i32(&mut self) -> Result<i32> {
        leb128::decode_i32(&mut self.reader)
            .map_err(|e| self.error(e.into()))
    }

    #[inline]
    pub fn parse_i64(&mut self) -> Result<i64> {
        leb128::decode_i64(&mut self.reader)
            .map_err(|e| self.error(e.into()))
    }

    #[inline]
    pub fn parse_f32(&mut self) -> Result<f32> {
        let bytes = self.reader.next_array::<4>()
            .ok_or_else(|| self.error(ErrorKind::UnexpectedEof))?;
        return Ok(f32::from_le_bytes(bytes));
    }

    #[inline]
    pub fn parse_f64(&mut self) -> Result<f64> {
        let bytes = self.reader.next_array::<8>()
            .ok_or_else(|| self.error(ErrorKind::UnexpectedEof))?;
        return Ok(f64::from_le_bytes(bytes));
    }

    #[inline]
    pub fn parse_u32(&mut self) -> Result<u32> {
        leb128::decode_u32(&mut self.reader)
            .map_err(|e| self.error(e.into()))
    }

    #[inline]
    pub fn parse_length(&mut self) -> Result<usize> {
        return Ok(self.parse_u32()? as usize);
    }

    pub fn parse_string(&mut self) -> Result<&'a str> {
        let len = self.parse_length()?;
        let bytes = self.reader.next_n(len)
            .ok_or_else(|| self.error(ErrorKind::UnexpectedEof))?;
        let string = core::str::from_utf8(bytes)
            .map_err(|_| self.error(ErrorKind::StringNotUtf8))?;
        return Ok(string);
    }

    pub fn parse_value_type(&mut self) -> Result<ValueType> {
        let at = self.next()?;
        let ty = ValueType::from_u8(at)
            .ok_or_else(|| self.error(ErrorKind::InvalidValueType))?;
        return Ok(ty);
    }

    pub fn parse_ref_type(&mut self) -> Result<RefType> {
        let at = self.next()?;
        let ty = RefType::from_u8(at)
            .ok_or_else(|| self.error(ErrorKind::InvalidRefType))?;
        return Ok(ty);
    }

    pub fn parse_func_type<'out>(&mut self, alloc: &'out Arena) -> Result<FuncType<'out>> {
        self.reader.expect(0x60)
            .map_err(|_| self.error(ErrorKind::InvalidFuncType))?;

        let num_params = self.parse_length()?;
        let mut params = Vec::with_cap_in(alloc, num_params);
        for _ in 0..num_params {
            params.push(self.parse_value_type()?);
        }

        let num_rets = self.parse_length()?;
        let mut rets = Vec::with_cap_in(alloc, num_rets);
        for _ in 0..num_rets {
            rets.push(self.parse_value_type()?);
        }

        return Ok(FuncType { params: params.leak(), rets: rets.leak() });
    }

    pub fn parse_block_type(&mut self) -> Result<BlockType> {
        let ty = leb128::decode_i64(&mut self.reader)
            .map_err(|e| self.error(e.into()))?;

        // @todo: explain this.
        if ty < 0 {
            let high_bits = !0x7f;
            if ty & high_bits != high_bits {
                return Err(self.error(ErrorKind::InvalidBlockType));
            }
            let ty = (ty & !high_bits) as u64 as u8;

            if ty == 0x40 {
                return Ok(BlockType::Unit);
            }

            return Ok(BlockType::Value(ValueType::from_u8(ty)
                .ok_or_else(|| self.error(ErrorKind::InvalidBlockType))?));
        }
        else {
            let ty = ty.try_into()
                .map_err(|_| self.error(ErrorKind::InvalidBlockType))?;
            return Ok(BlockType::Func(ty));
        }
    }

    #[inline] pub fn parse_type_idx(&mut self) -> Result<TypeIdx> { self.parse_u32() }

    #[inline] pub fn parse_label(&mut self) -> Result<u32> { self.parse_u32() }

    #[inline] pub fn parse_local_idx(&mut self) -> Result<u32> { self.parse_u32() }

    #[inline] pub fn parse_func_idx(&mut self) -> Result<FuncIdx> { self.parse_u32() }

    #[inline] pub fn parse_table_idx(&mut self) -> Result<TableIdx> { self.parse_u32() }

    #[inline] pub fn parse_memory_idx(&mut self) -> Result<MemoryIdx> { self.parse_u32() }

    #[inline] pub fn parse_global_idx(&mut self) -> Result<GlobalIdx> { self.parse_u32() }

    pub fn parse_limits(&mut self) -> Result<Limits> {
        return Ok(match self.next()? {
            0x00 => Limits { min: self.parse_u32()?, max: None },
            0x01 => Limits { min: self.parse_u32()?, max: Some(self.parse_u32()?) },

            _ => return Err(self.error(ErrorKind::InvalidLimits))
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
        let mutable = match self.next()? {
            0 => false,
            1 => true,

            _ => return Err(self.error(ErrorKind::InvalidGlobalType))
        };
        return Ok(GlobalType { ty, mutable });
    }


    pub fn parse_module_header(&mut self) -> Result<()> {
        self.reader.expect_n(b"\0asm")
            .map_err(|_| self.error(ErrorKind::InvalidSignature))?;
        self.reader.expect_n(&[1, 0, 0, 0])
            .map_err(|_| self.error(ErrorKind::InvalidVersion))?;
        return Ok(())
    }

    pub fn parse_sub_section(&mut self) -> Result<SubSection> {
        let len = self.parse_length()?;
        let offset = self.reader.offset();
        self.reader.next_n(len)
            .ok_or_else(|| self.error(ErrorKind::UnexpectedEof))?;
        return Ok(SubSection { offset, len });
    }

    pub fn parse_section(&mut self) -> Result<Section> {
        let kind = self.next()?;
        let kind = SectionKind::from_u8(kind)
            .ok_or_else(|| self.error(ErrorKind::InvalidSectionType))?;
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

            _ => return Err(self.error(ErrorKind::InvalidImport))
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

            _ => return Err(self.error(ErrorKind::InvalidExport))
        };
        return Ok(Export { name, kind });
    }

    pub fn parse_element<'out>(&mut self, alloc: &'out Arena) -> Result<Element<'out>> {
        return Ok(match self.parse_u32()? {
            0 => {
                let ConstExpr::I32(offset) = self.parse_const_expr()? else {
                    return Err(self.error(ErrorKind::Todo));
                };

                let num_values = self.parse_length()?;
                let mut values = Vec::with_cap_in(alloc, num_values);
                for _ in 0..num_values {
                    values.push(self.parse_u32()?);
                }

                Element {
                    ty: RefType::FuncRef,
                    kind: ElementKind::Active { table: 0, offset: offset as u32 },
                    values: values.leak(),
                }
            }

            2 => {
                let table = self.parse_u32()?;

                let ConstExpr::I32(offset) = self.parse_const_expr()? else {
                    return Err(self.error(ErrorKind::Todo));
                };

                // funcref.
                if self.reader.expect(0x00).is_err() {
                    return Err(self.error(ErrorKind::InvalidElement));
                }

                let num_values = self.parse_length()?;
                let mut values = Vec::with_cap_in(alloc, num_values);
                for _ in 0..num_values {
                    values.push(self.parse_u32()?);
                }

                Element {
                    ty: RefType::FuncRef,
                    kind: ElementKind::Active { table, offset: offset as u32 },
                    values: values.leak(),
                }
            }

            _ => return Err(self.error(ErrorKind::Todo))
        });
    }

    pub fn parse_code<'out>(&mut self, max_locals: u32, alloc: &'out Arena) -> Result<Code<'out>> {
        let sub = self.parse_sub_section()?;

        let mut p = self.sub_parser(sub);

        let num_local_groups = p.parse_u32()?;

        let mut locals = Vec::new_in(alloc);
        for _ in 0..num_local_groups {
            let n = p.parse_length()?;
            let ty = p.parse_value_type()?;

            if locals.len() + n > max_locals as usize {
                return Err(self.error(ErrorKind::TooManyLocals));
            }

            locals.extend((0..n).map(|_| ty));
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
                    return Err(self.error(ErrorKind::Todo));
                };

                let len = self.parse_length()?;
                let values = self.reader.next_n(len)
                    .ok_or_else(|| self.error(ErrorKind::UnexpectedEof))?;

                let kind = DataKind::Active { mem: 0, offset: offset as u32 };

                Data { kind, values }
            }

            _ => return Err(self.error(ErrorKind::Todo))
        });
    }


    pub fn parse_const_expr(&mut self) -> Result<ConstExpr> {
        todo!()
        /*
        let result = match self.parse_operator()? {
            Operator::I32Const { value } => ConstExpr::I32(value),
            Operator::I64Const { value } => ConstExpr::I64(value),
            Operator::F32Const { value } => ConstExpr::F32(value),
            Operator::F64Const { value } => ConstExpr::F64(value),
            Operator::GlobalGet { idx } => ConstExpr::Global(idx),
            Operator::RefNull { ty } => ConstExpr::RefNull(ty),

            _ => return Err(self.error(ErrorKind::InvalidConstExpr))
        };

        let Operator::End = self.parse_operator()? else {
            return Err(self.error(ErrorKind::InvalidConstExpr));
        };

        return Ok(result);
        */
    }

    pub fn parse_opcode(&mut self) -> Result<Opcode> {
        let at = self.next()?;
        match Opcode::parse(at) {
            opcode::ParseResult::Opcode(op) => Ok(op),

            opcode::ParseResult::Prefix(p) =>
                Opcode::parse_prefixed(p, self.parse_u32()?)
                    .ok_or_else(|| self.error(ErrorKind::UnsupportedOperator)),

            opcode::ParseResult::Error => Err(self.error(ErrorKind::UnsupportedOperator)),
        }
    }


    #[inline]
    #[must_use]
    fn error(&self, kind: ErrorKind) -> Error {
        Error { pos: self.reader.offset(), kind }
    }
}


impl<'a> Parser<'a> {
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

        let mut customs = Vec::new_in(alloc);

        while !p.is_done() {
            let section = p.parse_section()?;
            let kind = section.kind;

            if kind != SectionKind::Custom && has_section[kind as usize] {
                return Err(p.error(ErrorKind::DuplicateSection));
            }
            has_section[kind as usize] = true;

            let mut sp = p.sub_parser(section.sub);
            match kind {
                SectionKind::Custom => {
                    if customs.len() >= limits.max_customs as usize {
                        return Err(sp.error(ErrorKind::CustomSectionLimit));
                    }

                    customs.push(sp.parse_custom_section()?);
                }

                SectionKind::Type => {
                    let num_types = sp.parse_u32()?;
                    if num_types > limits.max_types {
                        return Err(sp.error(ErrorKind::TypeSectionLimit));
                    }

                    let mut types = Vec::with_cap_in(alloc, num_types as usize);
                    for _ in 0..num_types {
                        types.push(sp.parse_func_type(alloc)?);
                    }

                    module.types = types.leak();
                }

                SectionKind::Import => {
                    let num_imports = sp.parse_u32()?;
                    if num_imports > limits.max_imports {
                        return Err(sp.error(ErrorKind::ImportSectionLimit));
                    }

                    let mut imports = Vec::with_cap_in(alloc, num_imports as usize);

                    let mut num_funcs = 0;
                    let mut num_tables = 0;
                    let mut num_memories = 0;
                    let mut num_globals = 0;

                    for _ in 0..num_imports {
                        let import = sp.parse_import()?;
                        match import.kind {
                            ImportKind::Func(ty) => {
                                num_funcs += 1;
                                if module.types.get(ty as usize).is_none() {
                                    return Err(sp.error(ErrorKind::InvalidTypeIdx));
                                }
                            }

                            ImportKind::Table(_)  => num_tables   += 1,
                            ImportKind::Memory(_) => num_memories += 1,
                            ImportKind::Global(_) => num_globals  += 1,
                        }
                        imports.push(import);
                    }

                    let mut funcs = Vec::with_cap_in(alloc, num_funcs);
                    let mut tables = Vec::with_cap_in(alloc, num_tables);
                    let mut memories = Vec::with_cap_in(alloc, num_memories);
                    let mut globals = Vec::with_cap_in(alloc, num_globals);
                    for import in imports.iter().copied() {
                        match import.kind {
                            ImportKind::Func(it)   => funcs.push(it),
                            ImportKind::Table(it)  => tables.push(it),
                            ImportKind::Memory(it) => memories.push(it),
                            ImportKind::Global(it) => globals.push(it),
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
                        return Err(sp.error(ErrorKind::FuncSectionNotBeforeCode));
                    }

                    let num_funcs = sp.parse_u32()?;
                    if num_funcs > limits.max_funcs {
                        return Err(sp.error(ErrorKind::FuncSectionLimit));
                    }

                    let mut funcs = Vec::with_cap_in(alloc, num_funcs as usize);
                    for _ in 0..num_funcs {
                        let ty = sp.parse_u32()?;
                        if module.types.get(ty as usize).is_none() {
                            return Err(sp.error(ErrorKind::InvalidTypeIdx));
                        }
                        funcs.push(ty);
                    }

                    module.funcs = funcs.leak();
                }

                SectionKind::Table => {
                    let num_tables = sp.parse_u32()?;
                    if num_tables > limits.max_tables {
                        return Err(sp.error(ErrorKind::TableSectionLimit));
                    }

                    let mut tables = Vec::with_cap_in(alloc, num_tables as usize);
                    for _ in 0..num_tables {
                        tables.push(sp.parse_table_type()?);
                    }

                    module.tables = tables.leak();
                }

                SectionKind::Memory => {
                    let num_memories = sp.parse_u32()?;
                    if num_memories > limits.max_memories {
                        return Err(sp.error(ErrorKind::MemorySectionLimit));
                    }

                    let mut memories = Vec::with_cap_in(alloc, num_memories as usize);
                    for _ in 0..num_memories {
                        memories.push(sp.parse_memory_type()?);
                    }

                    module.memories = memories.leak();
                }

                SectionKind::Global => {
                    let num_globals = sp.parse_u32()?;
                    if num_globals > limits.max_globals {
                        return Err(sp.error(ErrorKind::GlobalSectionLimit));
                    }

                    let mut globals: Vec<Global, _> = Vec::with_cap_in(alloc, num_globals as usize);
                    for _ in 0..num_globals {
                        let global = sp.parse_global()?;

                        let init_ty = match global.init {
                            ConstExpr::I32(_) => ValueType::I32,
                            ConstExpr::I64(_) => ValueType::I64,
                            ConstExpr::F32(_) => ValueType::F32,
                            ConstExpr::F64(_) => ValueType::F64,
                            ConstExpr::Global(idx) => {
                                let idx = idx as usize;
                                let g = module.imports.globals.get(idx).copied()
                                    .ok_or_else(|| sp.error(ErrorKind::InvalidGlobalIdx))?;
                                if g.mutable {
                                    return Err(sp.error(ErrorKind::InvalidGlobalInit));
                                }
                                g.ty
                            }
                            ConstExpr::RefNull(ty) => ty.to_value_type(),
                        };
                        if init_ty != global.ty.ty {
                            return Err(sp.error(ErrorKind::InvalidGlobalInit));
                        }

                        globals.push(global);
                    }

                    module.globals = globals.leak();
                }

                SectionKind::Export => {
                    let num_exports = sp.parse_u32()?;
                    if num_exports > limits.max_exports {
                        return Err(sp.error(ErrorKind::ExportSectionLimit));
                    }

                    let mut exports = Vec::with_cap_in(alloc, num_exports as usize);
                    for _ in 0..num_exports {
                        let export = sp.parse_export()?;
                        match export.kind {
                            ExportKind::Func(idx) => {
                                if module.get_func(idx).is_none() {
                                    return Err(sp.error(ErrorKind::InvalidFuncIdx));
                                }
                            }

                            ExportKind::Table(idx) => {
                                if module.get_table(idx).is_none() {
                                    return Err(sp.error(ErrorKind::InvalidTableIdx));
                                }
                            }

                            ExportKind::Memory(idx) => {
                                if module.get_memory(idx).is_none() {
                                    return Err(sp.error(ErrorKind::InvalidMemoryIdx));
                                }
                            }

                            ExportKind::Global(idx) => {
                                if module.get_global(idx).is_none() {
                                    return Err(sp.error(ErrorKind::InvalidGlobalIdx));
                                }
                            }
                        }
                        exports.push(export);
                    }

                    module.exports = exports.leak();
                }

                SectionKind::Start => {
                    module.start = Some(sp.parse_u32()?);
                }

                SectionKind::Element => {
                    let num_elements = sp.parse_u32()?;
                    if num_elements > limits.max_elements {
                        return Err(sp.error(ErrorKind::ElementSectionLimit));
                    }

                    let mut elements = Vec::with_cap_in(alloc, num_elements as usize);
                    for _ in 0..num_elements {
                        let elem = sp.parse_element(alloc)?;
                        match elem.kind {
                            ElementKind::Passive => (),
                            ElementKind::Active { table, offset: _ } => {
                                if module.get_table(table).is_none() {
                                    return Err(sp.error(ErrorKind::InvalidTableIdx));
                                }
                            }
                            ElementKind::Declarative => (),
                        }
                        match elem.ty {
                            RefType::FuncRef => {
                                for idx in elem.values {
                                    if module.get_func(*idx).is_none() {
                                        return Err(sp.error(ErrorKind::InvalidFuncIdx));
                                    }
                                }
                            }
                            RefType::ExternRef => (),
                        }
                        elements.push(elem);
                    }

                    module.elements = elements.leak();
                }

                SectionKind::Code => {
                    let num_codes = sp.parse_u32()?;
                    if num_codes as usize != module.funcs.len() {
                        return Err(sp.error(ErrorKind::NumCodesNeNumFuncs));
                    }

                    let mut codes = Vec::with_cap_in(alloc, num_codes as usize);
                    for _ in 0..num_codes {
                        codes.push(sp.parse_code(limits.max_locals, alloc)?);
                    }

                    module.codes = codes.leak();
                }

                SectionKind::Data => {
                    let num_datas = sp.parse_u32()?;
                    if num_datas > limits.max_datas {
                        return Err(sp.error(ErrorKind::DataSectionLimit));
                    }

                    let mut datas = Vec::with_cap_in(alloc, num_datas as usize);
                    for _ in 0..num_datas {
                        let data = sp.parse_data()?;
                        match data.kind {
                            DataKind::Passive => (),

                            DataKind::Active { mem, offset: _ } => {
                                if module.get_memory(mem).is_none() {
                                    return Err(sp.error(ErrorKind::InvalidMemoryIdx));
                                }
                            }
                        }
                        datas.push(data);
                    }

                    module.datas = datas.leak();
                }

                SectionKind::DataCount => {
                    // @todo: validate.
                    sp.parse_u32()?;
                }
            }

            if sp.reader.len() != 0 {
                return Err(sp.error(ErrorKind::SectionTrailingData));
            }
        }

        if module.codes.len() != module.funcs.len() {
            return Err(p.error(ErrorKind::NumCodesNeNumFuncs));
        }

        module.customs = customs.leak();

        return Ok(module);
    }
}


