use core::cell::UnsafeCell;
use core::ptr::NonNull;
use core::marker::PhantomData;
use core::any::Any;

use sti::arena::Arena;
use sti::boks::Box;
use sti::rc::Rc;
use sti::vec::Vec;
use sti::keyed::KVec;
use sti::hash::HashMap;

use crate::{Error, Value};
use crate::table::{TableData, Table};
use crate::memory::{MemoryData, Memory};
use crate::global::{GlobalData, Global};
use crate::typed::{WasmTypes, HostFunc};
use crate::interp;


sti::define_key!(pub, u32, InstanceId);
sti::define_key!(pub, u32, FuncId);
sti::define_key!(pub, u32, TableId);
sti::define_key!(pub, u32, MemoryId);
sti::define_key!(pub, u32, GlobalId);

// @todo: fix trait impls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedFuncId<P, R> { func_id: FuncId, phantom: PhantomData<fn(P) -> R> }

impl<P, R> TypedFuncId<P, R> {
    #[inline]
    fn func_id(&self) -> FuncId { self.func_id }
}


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
impl<P, R> From<TypedFuncId<P, R>> for Extern { #[inline] fn from(value: TypedFuncId<P, R>) -> Self { Self::Func(value.func_id) } }
impl From<TableId> for Extern { #[inline] fn from(value: TableId) -> Self { Self::Table(value) } }
impl From<MemoryId> for Extern { #[inline] fn from(value: MemoryId) -> Self { Self::Memory(value) } }
impl From<GlobalId> for Extern { #[inline] fn from(value: GlobalId) -> Self { Self::Global(value) } }


pub struct Store {
    pub(crate) instances:   KVec<InstanceId,    Rc<UnsafeCell<InstanceData>>>,
    pub(crate) funcs:       KVec<FuncId,        Rc<UnsafeCell<FuncData>>>,
    pub(crate) tables:      KVec<TableId,       Rc<UnsafeCell<TableData>>>,
    pub(crate) memories:    KVec<MemoryId,      Rc<UnsafeCell<MemoryData>>>,
    pub(crate) globals:     KVec<GlobalId,      Rc<UnsafeCell<GlobalData>>>,
    pub(crate) thread: ThreadData,
}


pub(crate) struct InstanceData {
    #[allow(dead_code)]
    pub id: InstanceId,
    #[allow(dead_code)]
    pub wasm: Vec<u8>,
    #[allow(dead_code)]
    pub alloc: Arena,
    pub module: wasm::Module<'static>,
    pub funcs:    KVec<FuncId,   Rc<UnsafeCell<FuncData>>>,
    pub tables:   KVec<TableId,  Rc<UnsafeCell<TableData>>>,
    pub memories: KVec<MemoryId, Rc<UnsafeCell<MemoryData>>>,
    pub globals:  KVec<GlobalId, Rc<UnsafeCell<GlobalData>>>,
}


pub(crate) struct FuncData {
    pub id: FuncId,
    pub ty: wasm::FuncType<'static>,
    pub kind: FuncKind,
}

pub(crate) enum FuncKind {
    Interp(InterpFunc),
    Host(HostFuncData),
    Var(Option<Rc<UnsafeCell<FuncData>>>),
}

pub(crate) struct InterpFunc {
    pub instance: InstanceId,
    pub code: *const u8,
    pub code_len: usize,
    pub jumps: HashMap<u32, wasm::Jump>,
    pub num_params: u32,
    pub num_locals: u32, // including params.
    pub stack_size: u32, // including locals.
}

impl InterpFunc {
    #[inline]
    pub fn code_begin(&self) -> *const u8 {
        self.code
    }
    #[inline]
    pub fn code_end(&self) -> *const u8 {
        unsafe { self.code.add(self.code_len) }
    }
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
    pub instance: InstanceId,
    pub func: FuncId,
    pub pc: NonNull<u8>,
    pub bp_offset: u32,
}

pub(crate) struct ThreadData {
    pub stack: Vec<StackValue>,
    pub frames: Vec<Option<StackFrame>>,
    pub trapped: bool,
}

impl Store {
    pub fn new() -> Self {
        Self {
            instances: KVec::new(),
            funcs: KVec::new(),
            tables: KVec::new(),
            memories: KVec::new(),
            globals: KVec::new(),
            thread: ThreadData {
                stack: Vec::new(),
                frames: Vec::new(),
                trapped: false,
            },
        }
    }

    pub fn new_instance(&mut self, wasm: &[u8], imports: &[(&str, &str, Extern)]) -> Result<InstanceId, Error> {
        let instance_id = self.instances.next_key();

        let wasm = Vec::from_slice(wasm);
        let alloc = Arena::new();

        let wasm_static = unsafe { core::mem::transmute::<&[u8], &[u8]>(wasm.as_slice()) };
        let alloc_static = unsafe { core::mem::transmute::<&Arena, &Arena>(&alloc) };

        let module = wasm::Parser::parse_module(wasm_static, Default::default(), alloc_static)
            .map_err(|e| Error::Wasm(e))?;


        let num_funcs = module.imports.funcs.len() + module.funcs.len();
        let mut funcs = KVec::with_cap(num_funcs);
        self.funcs.inner_mut_unck().reserve_extra(num_funcs);

        let num_tables = module.imports.tables.len() + module.tables.len();
        let mut tables = KVec::with_cap(num_tables);
        self.tables.inner_mut_unck().reserve_extra(num_tables);

        let num_memories = module.imports.memories.len() + module.memories.len();
        let mut memories = KVec::with_cap(num_memories);
        self.memories.inner_mut_unck().reserve_extra(num_memories);

        let num_globals = module.imports.globals.len() + module.globals.len();
        let mut globals = KVec::with_cap(num_globals);
        self.globals.inner_mut_unck().reserve_extra(num_globals);

        for import in module.imports.imports {
            let lookup_import = |module, name| {
                for (m, n, import) in imports.iter().copied() {
                    if m == module && n == name {
                        return Ok(import);
                    }
                }
                println!("missing import {module:?} {name:?}");
                return Err(Error::MissingImport);
            };

            match import.kind {
                wasm::ImportKind::Func(ty) => {
                    let ty = module.types[ty as usize];

                    let Extern::Func(func_id) = lookup_import(import.module, import.name)? else {
                        todo!()
                    };

                    let func = self.funcs[func_id].clone();
                    let func_ty = unsafe { &*func.get() }.ty;
                    if func_ty != ty {
                        dbg!(func_ty);
                        dbg!(ty);
                        todo!()
                    }

                    funcs.push(func);
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

                    let global = self.globals[global_id].clone();
                    let g = Global::new(&global);
                    if g.ty() != ty.ty {
                        todo!()
                    }
                    if !g.mutable() && ty.mutable {
                        todo!()
                    }

                    globals.push(global);
                }
            }
        }


        let mut validator = wasm::Validator::new(&module);
        for (i, code) in module.codes.iter().enumerate() {
            let mut p = wasm::Parser::from_sub_section(&*wasm, code.expr);

            let ty_idx = module.funcs[i];
            let ty = module.types[module.funcs[i] as usize];


            if 0==1 { println!("{i}") };
            let mut jumps = HashMap::new();
            validator.validate_func(&mut p, ty_idx, code.locals, Some(&mut jumps))
                .map_err(|e| Error::Wasm(e))?;

            let interp_func = InterpFunc {
                instance: instance_id,
                code: unsafe { wasm.as_ptr().add(code.expr.offset) },
                code_len: code.expr.len,
                num_params: ty.params.len() as u32,
                num_locals: validator.num_locals(),
                stack_size: validator.stack_size(),
                jumps,
            };

            let id = self.funcs.next_key();
            let func = Rc::new(UnsafeCell::new(
                FuncData { id, ty, kind: FuncKind::Interp(interp_func) }));
            funcs.push(func.clone());
            self.funcs.push(func);
        }
        debug_assert_eq!(funcs.len(), num_funcs);

        for tab in module.tables {
            let id = self.new_table(tab.ty, tab.limits)?;
            tables.push(self.tables[id].clone());
        }
        debug_assert_eq!(tables.len(), num_tables);

        for mem in module.memories {
            let id = self.new_memory(mem.limits)?;
            memories.push(self.memories[id].clone());
        }
        debug_assert_eq!(memories.len(), num_memories);

        for global in module.globals {
            let init = match global.init {
                wasm::ConstExpr::I32(v) => Value::I32(v),
                wasm::ConstExpr::I64(v) => Value::I64(v),
                wasm::ConstExpr::F32(v) => Value::F32(v),
                wasm::ConstExpr::F64(v) => Value::F64(v),
                wasm::ConstExpr::Global(idx) => Global::new(&globals.inner()[idx as usize]).get(),
                wasm::ConstExpr::RefNull(ty) => match ty {
                    wasm::RefType::FuncRef => Value::FuncRef(RefValue::NULL),
                    wasm::RefType::ExternRef => Value::ExternRef(RefValue::NULL),
                },
            };

            let id = self.new_global(global.ty.mutable, init);
            globals.push(self.globals[id].clone());
        }
        debug_assert_eq!(globals.len(), num_globals);


        for elem in module.elements {
            match elem.kind {
                wasm::ElementKind::Passive => (),

                wasm::ElementKind::Active { table, offset } => {
                    let mut tab = Table::new(&tables.inner()[table as usize]);
                    let values = unsafe { tab.as_mut_slice() };

                    let Some(end) = (offset as usize).checked_add(elem.values.len()) else {
                        todo!()
                    };
                    if end > values.len() {
                        todo!()
                    }

                    for i in 0..elem.values.len() {
                        match elem.ty {
                            wasm::RefType::FuncRef => {
                                let idx = elem.values[i];
                                let id = unsafe { &*funcs.inner()[idx as usize].get() }.id.inner();
                                values[offset as usize + i] = RefValue { id };
                            }

                            wasm::RefType::ExternRef => todo!(),
                        }
                    }
                }

                wasm::ElementKind::Declarative => (),
            }
        }

        for data in module.datas {
            let bytes = data.values;
            match data.kind {
                wasm::DataKind::Passive => (),

                wasm::DataKind::Active { mem, offset } => {
                    let mut mem = Memory::new(&memories.inner()[mem as usize]);
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

        self.instances.push(Rc::new(UnsafeCell::new(InstanceData {
            id: instance_id,
            wasm,
            alloc,
            module,
            funcs,
            tables,
            memories,
            globals,
        })));

        return Ok(instance_id);
    }

    pub fn get_export(&self, instance_id: InstanceId, name: &str) -> Result<Extern, Error> {
        let inst = unsafe { &*self.instances[instance_id].get() };

        for export in inst.module.exports {
            if export.name == name {
                return Ok(match export.kind {
                    wasm::ExportKind::Func(idx)   => Extern::Func(unsafe { &*inst.funcs.inner()[idx as usize].get() }.id),
                    wasm::ExportKind::Table(idx)  => Extern::Table(Table::new(&inst.tables.inner()[idx as usize]).id()),
                    wasm::ExportKind::Memory(idx) => Extern::Memory(Memory::new(&inst.memories.inner()[idx as usize]).id()),
                    wasm::ExportKind::Global(idx) => Extern::Global(Global::new(&inst.globals.inner()[idx as usize]).id()),
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

    pub fn check_func_type<P: WasmTypes, R: WasmTypes>(&self, func_id: FuncId) -> Result<TypedFuncId<P, R>, Error> {
        let func = unsafe { &*self.funcs[func_id].get() };
        if func.ty.params != P::WASM_TYPES || func.ty.rets != R::WASM_TYPES {
            todo!()
        }

        Ok(TypedFuncId { func_id, phantom: PhantomData })
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
        let func = unsafe { &*self.funcs[func_id].get() };

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
        self.thread.stack.reserve_extra(args.len().max(rets.len()));
        for arg in args {
            self.thread.stack.push(StackValue::from_value(*arg));
        }

        if let Err(e) = self.run_func(func_id) {
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
        let func = unsafe { &*self.funcs[func_id.func_id].get() };
        debug_assert_eq!(func.ty.params, P::WASM_TYPES);
        debug_assert_eq!(func.ty.rets,   R::WASM_TYPES);

        // push args onto stack.
        let stack = &mut self.thread.stack;
        let bp = stack.len();
        stack.reserve_extra(P::WASM_TYPES.len().max(R::WASM_TYPES.len()));
        unsafe {
            args.to_stack_values(stack.as_mut_ptr().add(bp));
            stack.set_len(bp + P::WASM_TYPES.len());
        }

        self.run_func(func_id.func_id)?;

        let stack = &mut self.thread.stack;
        debug_assert_eq!(stack.len(), bp + R::WASM_TYPES.len());

        // pop results from stack.
        let result = unsafe { R::from_stack_values(stack.as_mut_ptr().add(bp)) };
        stack.truncate(bp);

        return Ok(result);
    }

    fn run_func(&mut self, id: FuncId) -> Result<(), Error> {
        let stack = &mut self.thread.stack;

        let old_num_frames = self.thread.frames.len();

        let mut func = unsafe { &*self.funcs[id].get() };
        while let FuncKind::Var(val) = &func.kind {
            func = unsafe { &*val.as_ref().unwrap().get() };
        }
        let id = func.id;

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
        (&mut self, f: H) -> TypedFuncId<P, R>
    {
        let id = TypedFuncId {
            func_id: self.funcs.next_key(),
            phantom: PhantomData,
        };

        self.funcs.push(Rc::new(UnsafeCell::new(FuncData {
            id: id.func_id,
            ty: wasm::FuncType { params: P::WASM_TYPES, rets: R::WASM_TYPES },
            kind: FuncKind::Host(
                HostFuncData {
                    data: {
                        let data = Box::new(f);
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
        })));

        return id;
    }

    pub fn new_func_var<P: WasmTypes, R: WasmTypes>(&mut self) -> TypedFuncId<P, R> {
        let id = TypedFuncId {
            func_id: self.funcs.next_key(),
            phantom: PhantomData,
        };

        self.funcs.push(Rc::new(UnsafeCell::new(FuncData {
            id: id.func_id,
            ty: wasm::FuncType { params: P::WASM_TYPES, rets: R::WASM_TYPES },
            kind: FuncKind::Var(None),
        })));

        return id;
    }

    pub fn assign_func_var<P: WasmTypes, R: WasmTypes>(&mut self, var: TypedFuncId<P, R>, value: TypedFuncId<P, R>) -> Result<(), Error> {
        let mut value = &self.funcs[value.func_id];
        {
            let v = unsafe { &*value.get() };
            debug_assert!(v.ty.params == P::WASM_TYPES && v.ty.rets == R::WASM_TYPES);
        }

        // occurs check.
        loop {
            let v = unsafe { &*value.get() };
            let FuncKind::Var(val) = &v.kind else { break };
            if v.id == var.func_id {
                todo!()
            }
            let Some(val) = &val else { break };
            value = val;
        }

        let var = unsafe { &mut *self.funcs[var.func_id].get() };
        debug_assert!(var.ty.params == P::WASM_TYPES && var.ty.rets == R::WASM_TYPES);

        let FuncKind::Var(v) = &mut var.kind else { todo!() };
        *v = Some(value.clone());

        return Ok(());
    }

    pub fn new_table(&mut self, ty: wasm::RefType, limits: wasm::Limits) -> Result<TableId, Error> {
        let id = self.tables.next_key();
        let table = TableData::new(id, ty, limits, RefValue::NULL)?;
        let table = Rc::new(UnsafeCell::new(table));
        self.tables.push(table);
        return Ok(id);
    }

    pub fn new_memory(&mut self, limits: wasm::Limits) -> Result<MemoryId, Error> {
        let id = self.memories.next_key();
        let memory = MemoryData::new(id, limits)?;
        let memory = Rc::new(UnsafeCell::new(memory));
        self.memories.push(memory);
        return Ok(id);
    }

    pub fn memory<'a>(&'a self, id: MemoryId) -> Memory<'a> {
        Memory::new(&self.memories[id])
    }

    pub fn new_global(&mut self, mutable: bool, value: Value) -> GlobalId {
        let id = self.globals.next_key();
        let global = GlobalData::new(id, mutable, value);
        let global = Rc::new(UnsafeCell::new(global));
        self.globals.push(global);
        return id;
    }


    pub fn caller_instance(&self) -> Result<InstanceId, Error> {
        // @speed: cache?
        let Some(Some(frame)) = self.thread.frames.last() else { return Err(Error::CallerNotWasm) };
        return Ok(frame.instance);
    }

    pub fn caller_memory<'a>(&'a self) -> Result<Memory<'a>, Error> {
        // @speed: cache?
        let Some(Some(frame)) = self.thread.frames.last() else { return Err(Error::CallerNotWasm) };
        let inst = unsafe { &*self.instances[frame.instance].get() };
        let Some(mem) = inst.memories.inner().get(0) else { return Err(Error::CallerNoMemory) };
        return Ok(Memory::new(mem));
    }
}


