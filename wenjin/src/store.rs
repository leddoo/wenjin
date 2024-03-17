use core::cell::UnsafeCell;
use core::ptr::NonNull;
use core::marker::PhantomData;
use core::any::Any;

use sti::traits::UnwrapDebug;
use sti::alloc::GlobalAlloc;
use sti::arena::Arena;
use sti::boks::Box;
use sti::manual_vec::ManualVec;

use crate::{Error, Value};
use crate::table::{TableData, Table};
use crate::memory::{MemoryData, Memory};
use crate::global::{GlobalData, Global};
use crate::typed::{WasmTypes, HostFunc};
use crate::interp;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleId { id: u32 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceId { id: u32 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FuncId { id: u32 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedFuncId<P, R> { id: u32, phantom: PhantomData<fn(P) -> R> }

impl<P, R> TypedFuncId<P, R> {
    #[inline]
    fn func(self) -> FuncId { FuncId { id: self.id } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableId { id: u32 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryId { id: u32 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlobalId { id: u32 }


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RefValue { pub(crate) id: u32 }

impl RefValue {
    pub const NULL: RefValue = RefValue { id: u32::MAX };

    #[inline(always)]
    pub fn is_null(self) -> bool {
        self.id == u32::MAX
    }

    #[inline(always)]
    pub fn to_option(self) -> Option<u32> {
        if self.id == u32::MAX { None }
        else { Some(self.id) }
    }
}


#[derive(Clone, Copy, Debug)]
pub enum Extern {
    Func(FuncId),
    Table(TableId),
    Memory(MemoryId),
    Global(GlobalId),
}

impl From<FuncId> for Extern { #[inline] fn from(value: FuncId) -> Self { Self::Func(value) } }
impl<P, R> From<TypedFuncId<P, R>> for Extern { #[inline] fn from(value: TypedFuncId<P, R>) -> Self { Self::Func(value.func()) } }
impl From<TableId> for Extern { #[inline] fn from(value: TableId) -> Self { Self::Table(value) } }
impl From<MemoryId> for Extern { #[inline] fn from(value: MemoryId) -> Self { Self::Memory(value) } }
impl From<GlobalId> for Extern { #[inline] fn from(value: GlobalId) -> Self { Self::Global(value) } }


pub struct Store {
    pub(crate) modules: ManualVec<ModuleData>,
    pub(crate) instances: ManualVec<InstanceData>,
    pub(crate) funcs: ManualVec<FuncData>,
    pub(crate) tables: ManualVec<Box<UnsafeCell<TableData>>>,
    pub(crate) memories: ManualVec<Box<UnsafeCell<MemoryData>>>,
    pub(crate) globals: ManualVec<Box<UnsafeCell<GlobalData>>>,
    pub(crate) thread: ThreadData,
}


pub(crate) struct ModuleData {
    #[allow(dead_code)]
    pub alloc: Arena,
    pub wasm: wasm::Module<'static>,
    pub funcs: ManualVec<ModuleFunc>,
}

pub(crate) struct ModuleFunc {
    pub ty: wasm::FuncType<'static>,
    pub code: Box<UnsafeCell<[u8]>>,
    pub stack_size: u32,
    pub num_locals: u32, // including params.
}


pub(crate) struct InstanceData {
    pub module: u32,
    pub funcs: ManualVec<u32>,
    pub tables: ManualVec<u32>,
    pub memories: ManualVec<u32>,
    pub globals: ManualVec<u32>,
}


pub(crate) struct FuncData {
    pub ty: wasm::FuncType<'static>,
    pub kind: FuncKind,
}

pub(crate) enum FuncKind {
    Interp(InterpFunc),
    Host(HostFuncData),
    Var(Option<u32>),
}

pub(crate) struct InterpFunc {
    pub instance: u32,
    pub code: *mut u8,
    pub code_end: *const u8,
    pub stack_size: u32,
    pub num_params: u32,
    pub num_locals: u32,
}

pub(crate) struct HostFuncData {
    pub data: Box<dyn Any>,
    pub call: fn(*const u8, &mut Store) -> Result<(), Error>,
    pub num_params: u8,
    pub num_rets: u8,
}


#[derive(Clone, Copy)]
#[repr(align(8))]
pub struct StackValue {
    bytes: [u8; 8],
}

impl StackValue {
    pub const ZERO: StackValue = StackValue { bytes: [0; 8] };

    #[inline]
    pub fn from_i32(v: i32) -> Self {
        Self { bytes: unsafe { core::mem::transmute([v.to_ne_bytes(), [0; 4]]) } }
    }

    #[inline]
    pub fn from_i64(v: i64) -> Self {
        Self { bytes: v.to_ne_bytes() }
    }

    #[inline]
    pub fn from_f32(v: f32) -> Self {
        Self { bytes: unsafe { core::mem::transmute([v.to_ne_bytes(), [0; 4]]) } }
    }

    #[inline]
    pub fn from_f64(v: f64) -> Self {
        Self { bytes: v.to_ne_bytes() }
    }

    #[inline]
    pub fn from_value(v: Value) -> Self {
        match v {
            Value::I32(v) => Self::from_i32(v),
            Value::I64(v) => Self::from_i64(v),
            Value::F32(v) => Self::from_f32(v),
            Value::F64(v) => Self::from_f64(v),
            Value::FuncRef(v) => Self::from_i32(v.id as i32),
            Value::ExternRef(v) => Self::from_i32(v.id as i32),
        }
    }

    #[inline]
    pub fn as_i32(self) -> i32 {
        unsafe { i32::from_ne_bytes(core::mem::transmute::<[u8; 8], [[u8; 4]; 2]>(self.bytes)[0]) }
    }

    #[inline]
    pub fn as_i64(self) -> i64 {
        i64::from_ne_bytes(self.bytes)
    }

    #[inline]
    pub fn as_f32(self) -> f32 {
        unsafe { f32::from_ne_bytes(core::mem::transmute::<[u8; 8], [[u8; 4]; 2]>(self.bytes)[0]) }
    }

    #[inline]
    pub fn as_f64(self) -> f64 {
        f64::from_ne_bytes(self.bytes)
    }

    #[inline]
    pub fn to_value(self, ty: wasm::ValueType) -> Value {
        match ty {
            wasm::ValueType::I32 => Value::I32(self.as_i32()),
            wasm::ValueType::I64 => Value::I64(self.as_i64()),
            wasm::ValueType::F32 => Value::F32(self.as_f32()),
            wasm::ValueType::F64 => Value::F64(self.as_f64()),
            wasm::ValueType::V128 => todo!(),
            wasm::ValueType::FuncRef => todo!(),
            wasm::ValueType::ExternRef => todo!(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct StackFrame {
    pub instance: u32,
    pub func: u32,
    pub pc: NonNull<u8>,
    pub bp_offset: u32,
}

pub(crate) struct ThreadData {
    pub stack: ManualVec<StackValue>,
    pub frames: ManualVec<Option<StackFrame>>,
    pub trapped: bool,
}

impl Store {
    pub fn new() -> Self {
        Self {
            modules: ManualVec::new(),
            instances: ManualVec::new(),
            funcs: ManualVec::new(),
            tables: ManualVec::new(),
            memories: ManualVec::new(),
            globals: ManualVec::new(),
            thread: ThreadData {
                stack: ManualVec::new(),
                frames: ManualVec::new(),
                trapped: false,
            },
        }
    }

    pub fn new_module(&mut self, wasm: &[u8]) -> Result<ModuleId, Error> {
        let id = ModuleId {
            id: self.modules.len().try_into().map_err(|_| Error::OOM)?,
        };

        let alloc = Arena::new();

        let module = wasm::Parser::parse_module(wasm, Default::default(), &alloc)
            .map_err(|e| match e {
                wasm::ParseModuleError::Parse(e) => Error::Parse(e),
                wasm::ParseModuleError::Validation(o, e) => Error::Validation(o, e),
            })?;

        let mut validator = wasm::Validator::new(&module);
        let mut compiler = interp::Compiler::new(&module);

        let mut funcs = ManualVec::with_cap(module.codes.len()).ok_or_else(|| Error::OOM)?;

        for (i, code) in module.codes.iter().enumerate() {
            let mut p = wasm::Parser::from_sub_section(wasm, code.expr);

            let ty_idx = module.funcs[i];
            let ty = module.types[module.funcs[i] as usize];

            validator.begin_func(ty_idx, code.locals).unwrap();
            compiler.begin_func(ty_idx);

            while !p.is_done() {
                let offset = p.reader.offset();
                let ((), ()) =
                    p.parse_operator_with(wasm::AndThenOp(&mut validator, &mut compiler))
                        .map_err(|e| Error::Parse(e))?
                        .map_err(|e| Error::Validation(offset, e))?;

                debug_assert_eq!(validator.num_stack(), compiler.num_stack());
                debug_assert_eq!(validator.num_frames(), compiler.num_frames());
                if validator.num_frames() != 0 {
                    debug_assert_eq!(validator.is_unreachable(), compiler.is_unreachable());
                }
            }
            assert!(validator.num_stack() == 0
                && validator.num_frames() == 0
                && compiler.num_stack() == 0
                && compiler.num_frames() == 0);

            if 0==1 { println!("{i}"); interp::dump(compiler.peek_code()); }


            let ty = unsafe { core::mem::transmute::<wasm::FuncType, wasm::FuncType>(ty) };
            funcs.push(ModuleFunc {
                ty,
                code: compiler.code(GlobalAlloc).ok_or_else(|| Error::OOM)?,
                stack_size: validator.num_locals() + validator.max_stack(),
                num_locals: validator.num_locals(),
            }).unwrap_debug();
        }

        let module = unsafe { core::mem::transmute::<wasm::Module, wasm::Module>(module) };
        self.modules.push_or_alloc(ModuleData {
            alloc,
            wasm: module,
            funcs,
        }).map_err(|_| Error::OOM)?;

        return Ok(id);
    }

    pub fn new_instance(&mut self, module_id: ModuleId, imports: &[(&str, &str, Extern)]) -> Result<InstanceId, Error> {
        let module = self.modules.get(module_id.id as usize).ok_or_else(|| todo!())?;

        // @temp
        let wasm = unsafe { core::mem::transmute::<&wasm::Module, &wasm::Module>(&module.wasm) };

        let id = InstanceId {
            id: self.instances.len().try_into().map_err(|_| Error::OOM)?,
        };


        let num_funcs = wasm.imports.funcs.len() + wasm.funcs.len();
        let mut funcs = ManualVec::with_cap(num_funcs).ok_or_else(|| Error::OOM)?;
        self.funcs.reserve_extra(num_funcs).map_err(|_| Error::OOM)?;
        if u32::try_from(self.funcs.len() + num_funcs).is_err() {
            return Err(Error::OOM);
        }

        let num_tables = wasm.imports.tables.len() + wasm.tables.len();
        let mut tables = ManualVec::with_cap(num_tables).ok_or_else(|| Error::OOM)?;

        let num_memories = wasm.imports.memories.len() + wasm.memories.len();
        let mut memories = ManualVec::with_cap(num_memories).ok_or_else(|| Error::OOM)?;

        let num_globals = wasm.imports.globals.len() + wasm.globals.len();
        let mut globals = ManualVec::with_cap(num_globals).ok_or_else(|| Error::OOM)?;

        for import in wasm.imports.imports {
            let lookup_import = |module, name| {
                for (m, n, import) in imports.iter().copied() {
                    if m == module && n == name {
                        return Ok(import);
                    }
                }
                todo!();
            };

            match import.kind {
                wasm::ImportKind::Func(ty) => {
                    let ty = wasm.types[ty as usize];

                    let Extern::Func(func_id) = lookup_import(import.module, import.name)? else {
                        todo!()
                    };
                    let func = &self.funcs[func_id.id as usize];
                    if func.ty != ty {
                        dbg!(func.ty);
                        dbg!(ty);
                        todo!()
                    }

                    funcs.push(func_id.id).unwrap_debug();
                }

                wasm::ImportKind::Table(_) => {
                    return Err(Error::Todo);
                }

                wasm::ImportKind::Memory(_) => {
                    return Err(Error::Todo);
                }

                wasm::ImportKind::Global(ty) => {
                    let Extern::Global(global_id) = lookup_import(import.module, import.name)? else {
                        todo!()
                    };
                    let global = Global::new(&self.globals[global_id.id as usize]);
                    if ty.ty != global.ty() {
                        todo!()
                    }
                    if ty.mutable && !global.mutable() {
                        todo!()
                    }

                    globals.push(global_id.id).unwrap_debug();
                }
            }
        }

        for func in module.funcs.iter() {
            let code = func.code.inner().as_ptr() as *mut u8;
            let code_end = unsafe {
                code.add((&*func.code.get()).len())
            };

            let i = self.funcs.len();
            funcs.push(i as u32).unwrap_debug();

            self.funcs.push(FuncData {
                ty: func.ty,
                kind: FuncKind::Interp(InterpFunc {
                    instance: id.id,
                    code,
                    code_end,
                    stack_size: func.stack_size,
                    num_params: func.ty.params.len() as u32,
                    num_locals: func.num_locals,
                }),
            }).unwrap_debug();
        }
        debug_assert_eq!(funcs.len(), num_funcs);

        for tab in wasm.tables {
            tables.push(self.new_table(tab.ty, tab.limits)?.id).unwrap_debug();
        }
        debug_assert_eq!(tables.len(), num_tables);

        for mem in wasm.memories {
            memories.push(self.new_memory(mem.limits)?.id).unwrap_debug();
        }
        debug_assert_eq!(memories.len(), num_memories);

        for global in wasm.globals {
            let init = match global.init {
                wasm::ConstExpr::I32(v) => Value::I32(v),
                wasm::ConstExpr::I64(v) => Value::I64(v),
                wasm::ConstExpr::F32(v) => Value::F32(v),
                wasm::ConstExpr::F64(v) => Value::F64(v),
                wasm::ConstExpr::Global(idx) => Global::new(&self.globals[globals[idx as usize] as usize]).get(),
                wasm::ConstExpr::RefNull(ty) => match ty {
                    wasm::RefType::FuncRef => Value::FuncRef(RefValue::NULL),
                    wasm::RefType::ExternRef => Value::ExternRef(RefValue::NULL),
                },
            };

            globals.push(self.new_global(global.ty.mutable, init)?.id).unwrap_debug();
        }
        debug_assert_eq!(globals.len(), num_globals);


        for elem in wasm.elements {
            match elem.kind {
                wasm::ElementKind::Passive => (),

                wasm::ElementKind::Active { table, offset } => {
                    let tab_id = tables[table as usize];
                    let mut tab = Table::new(&self.tables[tab_id as usize]);
                    let values = tab.as_mut_slice();

                    let Some(end) = (offset as usize).checked_add(elem.values.len()) else {
                        todo!()
                    };
                    if end > values.len() {
                        todo!()
                    }

                    for i in 0..elem.values.len() {
                        match elem.ty {
                            wasm::RefType::FuncRef => {
                                let func_idx = funcs[elem.values[i] as usize];
                                values[offset as usize + i] = RefValue { id: func_idx };
                            }

                            wasm::RefType::ExternRef => todo!(),
                        }
                    }
                }

                wasm::ElementKind::Declarative => (),
            }
        }

        for data in wasm.datas {
            let bytes = data.values;
            match data.kind {
                wasm::DataKind::Passive => (),

                wasm::DataKind::Active { mem, offset } => {
                    let mem_id = memories[mem as usize];
                    let mut mem = Memory::new(&self.memories[mem_id as usize]);
                    let (ptr, mem_len) = mem.as_mut_ptr();

                    let Some(end) = (offset as usize).checked_add(bytes.len()) else {
                        todo!()
                    };
                    if end > mem_len {
                        todo!()
                    }

                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            bytes.as_ptr(),
                            ptr.add(offset as usize),
                            bytes.len());
                    }
                }
            }
        }

        self.instances.push_or_alloc(InstanceData {
            module: module_id.id,
            funcs,
            tables,
            memories,
            globals,
        }).map_err(|_| Error::OOM)?;

        return Ok(id);
    }

    pub fn get_export(&self, instance_id: InstanceId, name: &str) -> Result<Extern, Error> {
        let instance = self.instances.get(instance_id.id as usize).ok_or_else(|| todo!())?;

        let module = &self.modules[instance.module as usize];
        for export in module.wasm.exports {
            if export.name == name {
                return Ok(match export.kind {
                    wasm::ExportKind::Func(idx)   => Extern::Func(FuncId { id: instance.funcs[idx as usize] }),
                    wasm::ExportKind::Table(idx)  => Extern::Table(TableId { id: instance.tables[idx as usize] }),
                    wasm::ExportKind::Memory(idx) => Extern::Memory(MemoryId { id: instance.memories[idx as usize] }),
                    wasm::ExportKind::Global(idx) => Extern::Global(GlobalId { id: instance.globals[idx as usize] }),
                });
            }
        }

        todo!()
    }

    pub fn get_export_func_dyn(&self, instance_id: InstanceId, name: &str) -> Result<FuncId, Error> {
        let Extern::Func(func) = self.get_export(instance_id, name)? else {
            todo!()
        };
        return Ok(func);
    }

    pub fn check_func_type<P: WasmTypes, R: WasmTypes>(&self, func: FuncId) -> Result<TypedFuncId<P, R>, Error> {
        let data = &self.funcs[func.id as usize];
        if data.ty.params != P::WASM_TYPES || data.ty.rets != R::WASM_TYPES {
            todo!()
        }

        Ok(TypedFuncId { id: func.id, phantom: PhantomData })
    }

    pub fn get_export_func<P: WasmTypes, R: WasmTypes>(&self, instance_id: InstanceId, name: &str) -> Result<TypedFuncId<P, R>, Error> {
        let func = self.get_export_func_dyn(instance_id, name)?;
        self.check_func_type(func)
    }

    pub fn get_export_table(&self, instance_id: InstanceId, name: &str) -> Result<TableId, Error> {
        let Extern::Table(tab) = self.get_export(instance_id, name)? else {
            todo!()
        };
        return Ok(tab);
    }

    pub fn get_export_memory(&self, instance_id: InstanceId, name: &str) -> Result<MemoryId, Error> {
        let Extern::Memory(mem) = self.get_export(instance_id, name)? else {
            todo!()
        };
        return Ok(mem);
    }

    pub fn get_export_global(&self, instance_id: InstanceId, name: &str) -> Result<GlobalId, Error> {
        let Extern::Global(glob) = self.get_export(instance_id, name)? else {
            todo!()
        };
        return Ok(glob);
    }

    pub fn call_dyn_ex<'r>(&mut self, func_id: FuncId, args: &[Value], rets: &'r mut [Value], allow_rets_mismatch: bool) -> Result<&'r mut [Value], Error> {
        let func = self.funcs.get(func_id.id as usize).ok_or_else(|| todo!())?;

        // @todo: this may not be safe in the future.
        //  a wasm function could delete its own instance/module.
        //  which would then presumably be freed on return (?),
        //  and that would invalidate `ty` (which points into the
        //  module's arena).
        let ty = func.ty;
        if args.len() != ty.params.len() {
            todo!()
        }
        for i in 0..args.len() {
            if args[i].ty() != ty.params[i] {
                todo!()
            }
        }

        if !allow_rets_mismatch && rets.len() != ty.rets.len() {
            todo!()
        }


        let is_root = self.thread.frames.len() == 0;

        // push args onto stack.
        let bp = self.thread.stack.len();
        self.thread.stack.reserve_extra(args.len().max(rets.len())).map_err(|_| Error::OOM)?;
        for arg in args {
            self.thread.stack.push(StackValue::from_value(*arg)).unwrap_debug();
        }

        if let Err(e) = self.run_func(func_id.id) {
            if is_root {
                self.thread.stack.clear();
                self.thread.frames.clear();
                self.thread.trapped = false;
            }
            return Err(e);
        }

        debug_assert_eq!(self.thread.stack.len(), bp + ty.rets.len());

        // pop rets from stack.
        let num_rets = ty.rets.len().min(rets.len());
        for i in 0..num_rets {
            rets[i] = self.thread.stack[bp + i].to_value(ty.rets[i]);
        }
        self.thread.stack.truncate(bp);

        return Ok(&mut rets[..num_rets]);
    }

    pub fn call_dyn<'r>(&mut self, func_id: FuncId, args: &[Value], rets: &'r mut [Value]) -> Result<&'r mut [Value], Error> {
        self.call_dyn_ex(func_id, args, rets, false)
    }

    // @todo: debug check types.
    pub fn call<P: WasmTypes, R: WasmTypes>(&mut self, func_id: TypedFuncId<P, R>, args: P) -> Result<R, Error> {
        let func = self.funcs.get(func_id.id as usize).ok_or_else(|| todo!())?;
        debug_assert_eq!(func.ty.params, P::WASM_TYPES);
        debug_assert_eq!(func.ty.rets,   R::WASM_TYPES);

        // push args onto stack.
        let stack = &mut self.thread.stack;
        let bp = stack.len();
        stack.reserve_extra(P::WASM_TYPES.len().max(R::WASM_TYPES.len())).map_err(|_| Error::OOM)?;
        unsafe {
            args.to_stack_values(stack.as_mut_ptr().add(bp));
            stack.set_len(bp + P::WASM_TYPES.len());
        }

        self.run_func(func_id.id)?;

        let stack = &mut self.thread.stack;
        debug_assert_eq!(stack.len(), bp + R::WASM_TYPES.len());

        // pop results from stack.
        let result = unsafe { R::from_stack_values(stack.as_mut_ptr().add(bp)) };
        stack.truncate(bp);

        return Ok(result);
    }

    fn run_func(&mut self, id: u32) -> Result<(), Error> {
        let stack = &mut self.thread.stack;

        let old_num_frames = self.thread.frames.len();

        let mut id = id;
        let mut func = &self.funcs[id as usize];
        while let FuncKind::Var(ref_id) = func.kind {
            let Some(ref_id) = ref_id else { todo!() };
            id = ref_id;
            func = &self.funcs[ref_id as usize];
        }

        let result = match &func.kind {
            FuncKind::Interp(f) => {
                debug_assert!(stack.len() >= f.num_params as usize);
                self.run_interp(id).0
            }

            FuncKind::Host(f) => {
                debug_assert!(stack.len() >= f.num_params as usize);
                debug_assert!(stack.cap() >= stack.len() - f.num_params as usize + f.num_rets as usize);
                (f.call)(&*f.data as *const _ as *const u8, self)
            }

            FuncKind::Var(_) => unreachable!()
        };

        if !self.thread.trapped {
            debug_assert_eq!(self.thread.frames.len(), old_num_frames);
        }

        return result;
    }

    pub fn new_host_func<P: WasmTypes, R: WasmTypes, const STORE: bool, H: HostFunc<P, R, STORE>>
        (&mut self, f: H) -> Result<TypedFuncId<P, R>, Error>
    {
        let id = TypedFuncId {
            id: self.funcs.len().try_into().map_err(|_| Error::OOM)?,
            phantom: PhantomData,
        };

        self.funcs.push_or_alloc(FuncData {
            ty: wasm::FuncType { params: P::WASM_TYPES, rets: R::WASM_TYPES },
            kind: FuncKind::Host(
                HostFuncData {
                    data: {
                        let data = Box::try_new_in(GlobalAlloc, f).ok_or(Error::OOM)?;
                        let (data, alloc) = data.into_raw_parts();
                        unsafe { Box::from_raw_parts(data as NonNull<dyn Any>, alloc) }
                    },
                    call: |this, store| {
                        H::call(unsafe { &*(this as *const H) }, store)
                    },
                    // have impls for tuples up to len 16.
                    num_params: P::WASM_TYPES.len().try_into().unwrap(),
                    num_rets: R::WASM_TYPES.len().try_into().unwrap(),
                })
        }).map_err(|_| Error::OOM)?;

        return Ok(id);
    }

    pub fn new_func_var<P: WasmTypes, R: WasmTypes>(&mut self) -> Result<TypedFuncId<P, R>, Error> {
        let id = TypedFuncId {
            id: self.funcs.len().try_into().map_err(|_| Error::OOM)?,
            phantom: PhantomData,
        };

        self.funcs.push_or_alloc(FuncData {
            ty: wasm::FuncType { params: P::WASM_TYPES, rets: R::WASM_TYPES },
            kind: FuncKind::Var(None),
        }).map_err(|_| Error::OOM)?;

        return Ok(id);
    }

    pub fn assign_func_var<P: WasmTypes, R: WasmTypes>(&mut self, var: TypedFuncId<P, R>, value: TypedFuncId<P, R>) -> Result<(), Error> {
        let Some(mut v) = self.funcs.get(value.id as usize) else { todo!() };
        debug_assert!(v.ty.params == P::WASM_TYPES && v.ty.rets == R::WASM_TYPES);
        // occurs check.
        while let FuncKind::Var(Some(id)) = v.kind {
            if id == var.id {
                todo!()
            }
            v = &self.funcs[id as usize];
        }

        let Some(var) = self.funcs.get_mut(var.id as usize) else { todo!() };
        debug_assert!(var.ty.params == P::WASM_TYPES && var.ty.rets == R::WASM_TYPES);
        let FuncKind::Var(id) = &mut var.kind else { todo!() };
        *id = Some(value.id);

        return Ok(());
    }

    pub fn new_table(&mut self, ty: wasm::RefType, limits: wasm::Limits) -> Result<TableId, Error> {
        let id = TableId {
            id: self.tables.len().try_into().map_err(|_| Error::OOM)?,
        };

        let table = TableData::new(ty, limits, RefValue::NULL)?;
        let table = Box::try_new_in(GlobalAlloc, UnsafeCell::new(table)).ok_or_else(|| Error::OOM)?;
        self.tables.push_or_alloc(table).map_err(|_| Error::OOM)?;
        return Ok(id);
    }

    pub fn new_memory(&mut self, limits: wasm::Limits) -> Result<MemoryId, Error> {
        let id = MemoryId {
            id: self.memories.len().try_into().map_err(|_| Error::OOM)?,
        };

        let memory = MemoryData::new(limits)?;
        let memory = Box::try_new_in(GlobalAlloc, UnsafeCell::new(memory)).ok_or_else(|| Error::OOM)?;
        self.memories.push_or_alloc(memory).map_err(|_| Error::OOM)?;
        return Ok(id);
    }

    pub fn memory<'a>(&'a self, id: MemoryId) -> Result<Memory<'a>, Error> {
        Ok(Memory::new(
            self.memories.get(id.id as usize)
            .ok_or(Error::InvalidHandle)?))
    }

    pub fn new_global(&mut self, mutable: bool, value: Value) -> Result<GlobalId, Error> {
        let id = GlobalId {
            id: self.globals.len().try_into().map_err(|_| Error::OOM)?,
        };

        let global = GlobalData::new(mutable, value);
        let global = Box::try_new_in(GlobalAlloc, UnsafeCell::new(global)).ok_or_else(|| Error::OOM)?;
        self.globals.push_or_alloc(global).map_err(|_| Error::OOM)?;
        return Ok(id);
    }


    pub fn caller_instance(&self) -> Result<InstanceId, Error> {
        // @speed: cache?
        let Some(Some(frame)) = self.thread.frames.last() else { return Err(Error::CallerNotWasm) };
        return Ok(InstanceId { id: frame.instance });
    }

    pub fn caller_memory<'a>(&'a self) -> Result<Memory<'a>, Error> {
        // @speed: cache?
        let Some(Some(frame)) = self.thread.frames.last() else { return Err(Error::CallerNotWasm) };
        let inst = &self.instances[frame.instance as usize];
        let Some(mem_id) = inst.memories.get(0) else { return Err(Error::CallerNoMemory) };
        return Ok(Memory::new(&self.memories[*mem_id as usize]));
    }
}


