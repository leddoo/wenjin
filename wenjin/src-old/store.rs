/*

store data structures overview:

    store data:
        - wasm objects.
        - interpreter state.

    the store contains several KVecs, one for each of the wasm objects:
    types, modules, instances, functions, memories, globals, tables.

    mutable wasm objects are heap allocated, so they can be aliased
    while their KVec is being modified.

    internally, access to a `&mut Store` *does not* imply exclusive access
    to anything contained in the store.
    any aliasable data must be wrapped in `UnsafeCell`.
    global aliasing invariants must be documented where the data is stored.
    for example, how wasm objects are referenced by the interpreter state.
    lexically contained aliasing should be documented in the code.
    todo: when should it be documented on the function?

*/


use core::marker::PhantomData;
use core::cell::UnsafeCell;

use sti::arena::Arena;
use sti::boks::Box;
use sti::vec::Vec;
use sti::keyed::KVec;

use crate::ParaSliceMut;
use crate::value::Value;
use crate::wasm;
use crate::interp::{*, self};



#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct StoreHandle<T> {
    id: T,
}

macro_rules! define_store_handle {
    ($name:ident, $ty:ty) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(StoreHandle<$ty>);
    };
}

sti::define_key!(pub(crate), u32, TypeId);
sti::define_key!(pub(crate), u32, ModuleId);
sti::define_key!(pub(crate), u32, InstanceId, opt: OptInstanceId);
sti::define_key!(pub(crate), u32, FuncId, rng: FuncIds);
sti::define_key!(pub(crate), u32, MemoryId);
sti::define_key!(pub(crate), u32, GlobalId);
sti::define_key!(pub(crate), u32, TableId);

define_store_handle!(Type, TypeId);
define_store_handle!(Module, ModuleId);
define_store_handle!(Instance, InstanceId);
define_store_handle!(Func, FuncId);
define_store_handle!(Memory, MemoryId);


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypedFunc<Params: WasmTypes, Rets: WasmTypes> {
    func:    Func,
    phantom: PhantomData<fn (Params) -> Rets>,
}

impl<P: WasmTypes, R: WasmTypes> TypedFunc<P, R> {
    #[inline(always)]
    pub fn func(&self) -> Func { self.func }
}



#[derive(Clone, Copy)]
pub enum Extern {
    Func(Func),
}

impl From<Func> for Extern {
    #[inline(always)]
    fn from(value: Func) -> Self { Extern::Func(value) }
}


pub struct Imports<'a> {
    imports: Vec<(&'a str, &'a str, Extern)>,
}

impl<'a> Imports<'a> {
    pub fn new() -> Self {
        Imports { imports: Vec::new() }
    }

    pub fn add(&mut self, module: &'a str, name: &'a str, value: Extern) {
        assert!(self.find(module, name).is_none());
        self.imports.push((module, name, value));
    }

    pub fn find(&self, module: &str, name: &str) -> Option<Extern> {
        for (m, n, e) in self.imports.iter().copied() {
            if m == module && n == name {
                return Some(e);
            }
        }
        None
    }
}


pub struct Store {
    pub(crate) persistent: Arena,
    pub(crate) temporary:  Arena,

    pub(crate) types:      KVec<TypeId,        wasm::FuncType<'static>>,
    pub(crate) modules:    KVec<ModuleId,      StoreModule>,
    pub(crate) instances:  KVec<InstanceId,    StoreInstance>,
    pub(crate) funcs:      KVec<FuncId,        StoreFunc>,
    pub(crate) memories:   KVec<MemoryId,      Box<UnsafeCell<StoreMemory>>>,
    pub(crate) globals:    KVec<GlobalId,      StoreGlobal>,
    pub(crate) tables:     KVec<TableId,       StoreTable>,

    pub(crate) interp: Interp,
}

#[derive(Clone)]
pub(crate) struct StoreModule {
    pub module: wasm::Module<'static>,
    pub interp_funcs: InterpFuncIds,
}

pub(crate) struct StoreInstance {
    pub module_id:  ModuleId,
    pub funcs:      Vec<FuncId>,
    pub memories:   Vec<MemoryId>,
    pub globals:    Vec<GlobalId>,
    pub tables:     Vec<TableId>,
}


#[derive(Clone, Copy)]
pub(crate) struct StoreFunc {
    pub ty: u32,
    pub data: StoreFuncData,
}

#[derive(Clone, Copy)]
pub(crate) enum StoreFuncData {
    Guest(GuestFuncData),
    Host(HostFuncData),
}

#[derive(Clone, Copy)]
pub(crate) struct GuestFuncData {
    pub instance: InstanceId,
    pub interp: InterpFuncId,
}

#[derive(Clone, Copy)]
pub struct HostFuncData {
    pub(crate) data: *const u8,
    pub(crate) call: HostFuncDataFn,
}

#[derive(Clone, Copy)]
pub enum HostFuncDataFn {
    Plain       (fn ((), *const u8, *mut StackValue) -> Result<(), ()>),
    WithMemory  (fn (*mut MemoryView<'static>, *const u8, *mut StackValue) -> Result<(), ()>),
}


#[allow(dead_code)] // @temp
pub(crate) struct StoreMemory {
    pub limits: wasm::Limits,
    pub bytes:  Vec<u8>,
}

#[allow(dead_code)] // @temp
pub(crate) struct StoreGlobal {
    pub ty:    wasm::ValueType,
    pub mutt:  bool,
    pub value: StackValue,
}

#[allow(dead_code)] // @temp
pub(crate) struct StoreTable {
    pub ty:     wasm::ValueType,
    pub limits: wasm::Limits,
    pub values: Vec<u32>,
}

impl Store {
    pub fn new() -> Store {
        let persistent = Arena::new();
        persistent.min_block_size.set(1*1024*1024);

        let temporary = Arena::new();
        temporary.min_block_size.set(1*1024*1024);

        Store {
            persistent,
            temporary,
            types:      KVec::new(),
            modules:    KVec::new(),
            instances:  KVec::new(),
            funcs:      KVec::new(),
            memories:   KVec::new(),
            globals:    KVec::new(),
            tables:     KVec::new(),
            interp: Interp::new(),
        }
    }


    pub fn load_module(&mut self, wasm: &[u8]) -> Result<Module, ()> {
        let wasm::parser::ParseModuleResult { module, custom: _, code } =
            wasm::parser::parse_module(wasm, &self.persistent)?;
        let module = unsafe { core::mem::transmute(module) };

        let interp_funcs = self.interp.compile(&module, code, &self.persistent, &mut self.temporary)?;

        let id = self.modules.push(StoreModule { module, interp_funcs });
        Ok(Module(StoreHandle { id }))
    }

    pub fn instantiate_module(&mut self, module: Module, imports: &Imports) -> Result<Instance, ()> {
        let module_id = module.0.id;
        let StoreModule { module, interp_funcs } = self.modules[module_id].clone();

        let instance = self.instances.next_key();

        // funcs.
        let mut funcs = Vec::new();
        {
            let num_funcs = module.imports.funcs.len() + interp_funcs.len();

            funcs.reserve_exact(num_funcs);
            self.funcs.inner_mut_unck().reserve_extra(num_funcs);

            // imports.
            for import in module.imports.imports {
                if import.kind != wasm::ImportKind::Func {
                    continue;
                }

                let ty = module.imports.funcs[import.index as usize];
                let ty = &module.types[ty as usize];

                let Some(ext) = imports.find(import.module, import.name) else {
                    println!("missing import {:?}::{:?}", import.module, import.name);
                    return Err(());
                };

                let Extern::Func(func) = ext;// else { return Err(()) };
                let func = func.0.id;

                let func_ty = &self.types[TypeId(self.funcs[func].ty)];
                if func_ty != ty {
                    println!("import func type mismatch");
                    return Err(());
                }
                
                funcs.push(func);
            }

            // guest funcs.
            for (i, func) in interp_funcs.enumerate() {
                let func = self.funcs.push(StoreFunc {
                    ty: module.func_types[i],
                    data: StoreFuncData::Guest(GuestFuncData {
                        instance,
                        interp: func,
                    }),
                });
                funcs.push(func);
            }
        }

        // memories.
        // @todo: imports.
        let mut memories = Vec::with_cap(module.memories.len());
        self.memories.inner_mut_unck().reserve_extra(module.memories.len());
        for memory in module.memories {
            let initial_size = memory.limits.min as usize * wasm::PAGE_SIZE;
            let mut bytes = Vec::with_cap(initial_size);
            unsafe {
                core::ptr::write_bytes(bytes.as_mut_ptr(), 0, initial_size);
                bytes.set_len(initial_size);
            }

            let memory = StoreMemory { limits: memory.limits, bytes };
            let memory = self.memories.push(Box::new(memory.into()));
            memories.push(memory);
        }

        // globals.
        // @todo: imports.
        let mut globals = Vec::with_cap(module.globals.len());
        self.globals.inner_mut_unck().reserve_extra(module.globals.len());
        for global in module.globals {
            let value = match global.init {
                wasm::ConstExpr::I32(v) => StackValue::I32(v),
                wasm::ConstExpr::I64(v) => StackValue::I64(v),
                wasm::ConstExpr::F32(v) => StackValue::F32(v),
                wasm::ConstExpr::F64(v) => StackValue::F64(v),
            };

            let global = self.globals.push(StoreGlobal { ty: global.ty.ty, mutt: global.ty.mutt, value });
            globals.push(global);
        }

        // tables.
        // @todo: imports.
        let mut tables = Vec::with_cap(module.tables.len());
        self.tables.inner_mut_unck().reserve_extra(module.tables.len());
        for table in module.tables {
            let mut values = Vec::with_cap(table.limits.min as usize);
            for _ in 0..table.limits.min {
                values.push(u32::MAX);
            }

            let table = self.tables.push(StoreTable { ty: table.ty, limits: table.limits, values });
            tables.push(table);
        }

        // elements.
        for elem in module.elems {
            let wasm::ElemKind::Active { tab_idx, offset } = elem.kind else { continue };

            let table = &mut self.tables[tables[tab_idx as usize]];
            for (i, value) in elem.values.iter().copied().enumerate() {
                table.values[offset as usize + i as usize] = value;
            }
        }

        // datas.
        for data in module.datas {
            let wasm::DataKind::Active { mem_idx, offset } = data.kind else { continue };

            // safety: this memory was just created,
            //  and access is local to this block,
            //  so the memory cannot be aliased.
            let memory = unsafe {
                &mut *self.memories[memories[mem_idx as usize]].get()
            };

            assert!(offset as usize + data.values.len() <= memory.bytes.len());
            unsafe {
                core::ptr::copy_nonoverlapping(
                    data.values.as_ptr(),
                    memory.bytes.as_mut_ptr().add(offset as usize),
                    data.values.len());
            }
        }


        let _instance = self.instances.push(StoreInstance { module_id, funcs, memories, globals, tables });
        assert_eq!(instance, _instance);

        Ok(Instance(StoreHandle { id: instance }))
    }


    pub fn exported_func_dyn(&self, instance: Instance, name: &str) -> Option<Func> {
        let inst = &self.instances[instance.0.id];
        let modd = &self.modules[inst.module_id];

        for export in modd.module.exports {
            if export.name != name { continue }

            let wasm::ExportData::Func(index) = export.data else { continue };
            let id = inst.funcs[index as usize];
            return Some(Func(StoreHandle { id }));
        }

        return None;
    }

    pub fn indexed_func_dyn(&self, instance: Instance, index: u32) -> Option<Func> {
        let inst = &self.instances[instance.0.id];

        inst.funcs.get(index as usize).copied()
        .map(|id| Func(StoreHandle { id }))
    }

    pub fn check_func_type<P: WasmTypes, R: WasmTypes>(&self, func: Func) -> Option<TypedFunc<P, R>> {
        let data = &self.funcs[func.0.id];

        let ty = data.ty;
        let ty = match data.data {
            StoreFuncData::Guest(data) => {
                let module = &self.modules[self.instances[data.instance].module_id];
                &module.module.types[ty as usize]
            }

            StoreFuncData::Host(_) => {
                &self.types[TypeId(ty)]
            }
        };

        if ty.params != P::WASM_TYPES || ty.rets != R::WASM_TYPES {
            return None;
        }

        Some(TypedFunc { func, phantom: PhantomData })
    }

    pub fn exported_func<P: WasmTypes, R: WasmTypes>(&self, instance: Instance, name: &str) -> Option<TypedFunc<P, R>> {
        let func = self.exported_func_dyn(instance, name)?;
        self.check_func_type(func)
    }

    pub fn indexed_func<P: WasmTypes, R: WasmTypes>(&self, instance: Instance, index: u32) -> Option<TypedFunc<P, R>> {
        let func = self.indexed_func_dyn(instance, index)?;
        self.check_func_type(func)
    }

    pub fn call<P: WasmTypes, R: WasmTypes>(&mut self, func: TypedFunc<P, R>, args: P) -> Result<R, ()> {
        let func = self.funcs[func.func.0.id];

        match func.data {
            StoreFuncData::Guest(data) => {
                interp::run(self, data.instance, data.interp, args)
            }

            StoreFuncData::Host(data) => {
                // @todo: use stack.
                let mut temp = Vec::with_cap(P::WASM_TYPES.len().max(R::WASM_TYPES.len()));
                args.to_stack_values(temp.as_mut_ptr());

                match data.call {
                    HostFuncDataFn::Plain(f) => {
                        f((), data.data, temp.as_mut_ptr())?;
                    }

                    HostFuncDataFn::WithMemory(_) => {
                        // @todo-error: WithMemory function can only be called by wasm.
                        return Err(());
                    }
                }

                Ok(R::from_stack_values(temp.as_ptr()))
            }
        }
    }

    pub fn call_dyn(&mut self, func: Func, args: &[Value]) -> Result<Vec<Value>, ()> {
        let func = self.funcs[func.0.id];

        match func.data {
            StoreFuncData::Guest(data) => {
                let inst = &self.instances[data.instance];
                let modd = &self.modules[inst.module_id];

                let ty = &modd.module.types[func.ty as usize];

                let mut rets = Vec::with_cap(ty.rets.len());
                for _ in 0..ty.rets.len() {
                    rets.push(Value::I32(0));
                }

                interp::run_dyn(self, data.instance, data.interp, ty, args, &mut rets)?;

                return Ok(rets);
            }

            StoreFuncData::Host(data) => {
                let ty = &self.types[TypeId(func.ty)];

                assert_eq!(args.len(), ty.params.len());
                for (i, arg) in args.iter().enumerate() {
                    assert_eq!(arg.ty(), ty.params[i]);
                }

                // @todo: use stack.
                let mut temp = Vec::with_cap(args.len().max(ty.rets.len()));
                for arg in args {
                    temp.push(arg.to_stack_value());
                }

                match data.call {
                    HostFuncDataFn::Plain(f) => {
                        f((), data.data, temp.as_mut_ptr())?;
                    }

                    HostFuncDataFn::WithMemory(_) => {
                        // @todo-error: WithMemory function can only be called by wasm.
                        return Err(());
                    }
                }
                unsafe { temp.set_len(ty.rets.len()) }

                let mut rets = Vec::with_cap(ty.rets.len());
                for (i, ret) in temp.iter().enumerate() {
                    rets.push(match ty.rets[i] {
                        wasm::ValueType::I32 => { Value::I32(ret.i32()) }
                        wasm::ValueType::I64 => { Value::I64(ret.i64()) }
                        wasm::ValueType::F32 => { Value::F32(ret.f32()) }
                        wasm::ValueType::F64 => { Value::F64(ret.f64()) }

                        _ => unimplemented!()
                    });
                }

                Ok(rets)
            }
        }
    }


    pub fn exported_memory(&self, instance: Instance, name: &str) -> Option<Memory> {
        let inst = &self.instances[instance.0.id];
        let modd = &self.modules[inst.module_id];

        for export in modd.module.exports {
            if export.name != name { continue }

            let wasm::ExportData::Memory(index) = export.data else { continue };
            let id = inst.memories[index as usize];
            return Some(Memory(StoreHandle { id }));
        }

        return None;
    }

    pub fn memory_view(&mut self, memory: Memory) -> MemoryView {
        //let mem = &mut self.memories[memory.0.id];
        //MemoryView::new_unsafe(mem.bytes.as_mut_ptr(), mem.bytes.len())
        todo!()
    }

    pub fn with_memory_view<R, F: FnOnce(&MemoryView) -> Result<R, ()>>(&self, memory: Memory, f: F) -> Result<R, ()> {
        //let mem = &self.memories[memory.0.id];
        //let mem = MemoryView::new_unsafe(mem.bytes.as_ptr() as *mut u8, mem.bytes.len());
        //f(&mem)
        todo!()
    }


    pub fn add_type(&mut self, params: &[wasm::ValueType], rets: &[wasm::ValueType]) -> Type {
        let mut ps = Vec::with_cap_in(&self.persistent, params.len());
        for param in params.iter().copied() {
            ps.push(param);
        }

        let mut rs = Vec::with_cap_in(&self.persistent, rets.len());
        for ret in rets.iter().copied() {
            rs.push(ret);
        }

        let id = self.types.push(wasm::FuncType {
            params: unsafe { &*(Vec::leak(ps) as *const _) },
            rets:   unsafe { &*(Vec::leak(rs) as *const _) },
        });
        return Type(StoreHandle { id });
    }

    pub fn add_func<P: WasmTypes, R: WasmTypes, K: HostFuncKind, H: HostFunc<P, R, K>>(&mut self, func: H) -> TypedFunc<P, R> {
        let ty = self.add_type(P::WASM_TYPES, R::WASM_TYPES).0.id.inner();

        let id = self.funcs.push(StoreFunc { ty, data: StoreFuncData::Host(
            HostFuncData {
                data: self.persistent.alloc_new(func) as *const H as *const u8,
                call: K::to_host_func_data_fn(|param, data, stack| {
                    let data = unsafe { &*(data as *const H) };
                    data.call(param, stack)
                }),
            })});

        return TypedFunc { func: Func(StoreHandle { id }), phantom: PhantomData };
    }
}




pub trait WasmType: Clone + Copy + 'static {
    const WASM_TYPE: wasm::ValueType;

    fn to_stack_value(self) -> StackValue;
    fn from_stack_value(value: StackValue) -> Self;
}

impl WasmType for i32 {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::I32;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue { StackValue::I32(self) }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> Self { value.i32() }
}

impl WasmType for u32 {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::I32;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue { StackValue::U32(self) }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> Self { value.u32() }
}

impl WasmType for i64 {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::I64;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue { StackValue::I64(self) }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> Self { value.i64() }
}

impl WasmType for u64 {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::I64;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue { StackValue::U64(self) }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> Self { value.u64() }
}

impl WasmType for f32 {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::F32;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue { StackValue::F32(self) }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> Self { value.f32() }
}

impl WasmType for f64 {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::F64;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue { StackValue::F64(self) }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> Self { value.f64() }
}


pub trait WasmTypes {
    const WASM_TYPES: &'static [wasm::ValueType];

    fn to_stack_values(self, dst: *mut StackValue);
    fn from_stack_values(src: *const StackValue) -> Self;
}

impl<T0: WasmType> WasmTypes for T0 {
    const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE];

    #[inline(always)]
    fn to_stack_values(self, dst: *mut StackValue) { unsafe {
        dst.add(0).write(self.to_stack_value());
    }}

    #[inline(always)]
    fn from_stack_values(src: *const StackValue) -> Self { unsafe {
        T0::from_stack_value(src.add(0).read())
    }}
}


pub trait WasmResult {
    type Types: WasmTypes;

    fn to_result(self) -> Result<Self::Types, ()>;
}

impl<T: WasmTypes> WasmResult for T {
    type Types = T;

    #[inline(always)]
    fn to_result(self) -> Result<T, ()> { Ok(self) }
}

impl<T: WasmTypes> WasmResult for Result<T, ()> {
    type Types = T;

    #[inline(always)]
    fn to_result(self) -> Result<T, ()> { self }
}


mod private {
    use super::*;

    pub trait HostFuncKind {
        type Param;

        fn to_host_func_data_fn(f: fn (Self::Param, *const u8, *mut StackValue) -> Result<(), ()>) -> HostFuncDataFn;
    }

    pub struct HostFuncKindPlain;
    impl HostFuncKind for HostFuncKindPlain {
        type Param = ();

        #[inline(always)]
        fn to_host_func_data_fn(f: fn ((), *const u8, *mut StackValue) -> Result<(), ()>) -> HostFuncDataFn {
            HostFuncDataFn::Plain(f)
        }
    }

    pub struct HostFuncKindWithMemory;
    impl HostFuncKind for HostFuncKindWithMemory {
        type Param = *mut MemoryView<'static>;

        #[inline(always)]
        fn to_host_func_data_fn(f: fn (*mut MemoryView<'static>, *const u8, *mut StackValue) -> Result<(), ()>) -> HostFuncDataFn {
            HostFuncDataFn::WithMemory(f)
        }
    }
}
pub(crate) use private::*;


pub unsafe trait HostFunc<Params: WasmTypes, Rets: WasmTypes, Kind: HostFuncKind>: 'static + Sized {
    fn call(&self, param: Kind::Param, stack: *mut StackValue) -> Result<(), ()>;
}



/// CType
/// - a trait for types that can safely be read from & written to a wasm memory.
/// - these types must have valid values for all bit patterns (excluding padding).
///   hence, booleans & enums do not implement `CType`.
/// - use `#[derive(Clone, Copy, CType)] #[repr(C)]` to implement `CType` for custom structs.
pub unsafe trait CType: Copy + 'static {
    /// clear_padding
    /// - to preserve determinism, padding bytes must be cleared,
    ///   when writing values to a memory.
    ///   because in rust, padding bytes have undefined values.
    unsafe fn clear_padding(&self, bytes: ParaSliceMut<u8>);
}

#[inline(always)]
unsafe fn write_ctype<T: CType>(ptr: *mut T, value: T) { unsafe {
    ptr.write_unaligned(value);

    let bytes = ParaSliceMut {
        slice: core::slice::from_raw_parts_mut(
            ptr as *mut u8,
            core::mem::size_of::<T>()),
    };
    T::clear_padding(&value, bytes);
}}

unsafe impl CType for u8  { #[inline(always)] unsafe fn clear_padding(&self, bytes: ParaSliceMut<u8>) { unsafe { bytes.assume_len(core::mem::size_of::<Self>()) } } }
unsafe impl CType for u16 { #[inline(always)] unsafe fn clear_padding(&self, bytes: ParaSliceMut<u8>) { unsafe { bytes.assume_len(core::mem::size_of::<Self>()) } } }
unsafe impl CType for u32 { #[inline(always)] unsafe fn clear_padding(&self, bytes: ParaSliceMut<u8>) { unsafe { bytes.assume_len(core::mem::size_of::<Self>()) } } }
unsafe impl CType for u64 { #[inline(always)] unsafe fn clear_padding(&self, bytes: ParaSliceMut<u8>) { unsafe { bytes.assume_len(core::mem::size_of::<Self>()) } } }

unsafe impl CType for i8  { #[inline(always)] unsafe fn clear_padding(&self, bytes: ParaSliceMut<u8>) { unsafe { bytes.assume_len(core::mem::size_of::<Self>()) } } }
unsafe impl CType for i16 { #[inline(always)] unsafe fn clear_padding(&self, bytes: ParaSliceMut<u8>) { unsafe { bytes.assume_len(core::mem::size_of::<Self>()) } } }
unsafe impl CType for i32 { #[inline(always)] unsafe fn clear_padding(&self, bytes: ParaSliceMut<u8>) { unsafe { bytes.assume_len(core::mem::size_of::<Self>()) } } }
unsafe impl CType for i64 { #[inline(always)] unsafe fn clear_padding(&self, bytes: ParaSliceMut<u8>) { unsafe { bytes.assume_len(core::mem::size_of::<Self>()) } } }

unsafe impl CType for f32 { #[inline(always)] unsafe fn clear_padding(&self, bytes: ParaSliceMut<u8>) { unsafe { bytes.assume_len(core::mem::size_of::<Self>()) } } }
unsafe impl CType for f64 { #[inline(always)] unsafe fn clear_padding(&self, bytes: ParaSliceMut<u8>) { unsafe { bytes.assume_len(core::mem::size_of::<Self>()) } } }



#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct WasmSize(pub u32);

impl WasmSize {
    #[inline(always)]
    pub fn usize(self) -> usize { self.0 as usize }
}

impl WasmType for WasmSize {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::I32;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue {
        StackValue::U32(self.0)
    }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> WasmSize {
        WasmSize(value.u32())
    }
}

unsafe impl CType for WasmSize { #[inline(always)] unsafe fn clear_padding(&self, bytes: ParaSliceMut<u8>) { unsafe { bytes.assume_len(core::mem::size_of::<Self>()) } } }



pub struct WasmPtr<T: CType> {
    pub addr: u32,
    phantom: PhantomData<*mut T>,
}

impl<T: CType> WasmPtr<T> {
    #[inline(always)]
    pub fn new(addr: u32) -> WasmPtr<T> {
        WasmPtr { addr, phantom: PhantomData }
    }

    #[inline(always)]
    pub fn add(self, delta: u32) -> WasmPtr<T> {
        WasmPtr::new(self.addr + delta*core::mem::size_of::<T>() as u32)
    }

    #[inline(always)]
    pub fn checked_add(self, delta: u32) -> Option<WasmPtr<T>> {
        let delta = delta.checked_mul(core::mem::size_of::<T>() as u32)?;
        Some(WasmPtr::new(self.addr.checked_add(delta)?))
    }

    #[inline(always)]
    pub fn wrapping_add(self, delta: u32) -> WasmPtr<T> {
        let delta = delta.wrapping_mul(core::mem::size_of::<T>() as u32);
        WasmPtr::new(self.addr.wrapping_add(delta))
    }


    #[inline(always)]
    pub fn byte_add(self, delta: u32) -> WasmPtr<T> {
        WasmPtr::new(self.addr + delta)
    }

    #[inline(always)]
    pub fn checked_byte_add(self, delta: u32) -> Option<WasmPtr<T>> {
        Some(WasmPtr::new(self.addr.checked_add(delta)?))
    }

    #[inline(always)]
    pub fn wrapping_byte_add(self, delta: u32) -> WasmPtr<T> {
        WasmPtr::new(self.addr.wrapping_add(delta))
    }
}

impl<T: CType> Clone for WasmPtr<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        WasmPtr { addr: self.addr, phantom: PhantomData }
    }
}

impl<T: CType> Copy for WasmPtr<T> {}

impl<T: CType> PartialEq for WasmPtr<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.addr.eq(&other.addr)
    }
}

impl<T: CType> Eq for WasmPtr<T> {}

impl<T: CType> core::fmt::Debug for WasmPtr<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "0x{:x}", self.addr)
    }
}

impl<T: CType> WasmType for WasmPtr<T> {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::I32;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue {
        StackValue::U32(self.addr)
    }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> WasmPtr<T> {
        WasmPtr { addr: value.u32(), phantom: PhantomData }
    }
}

unsafe impl<T: CType> CType for WasmPtr<T> { #[inline(always)] unsafe fn clear_padding(&self, bytes: ParaSliceMut<u8>) { unsafe { bytes.assume_len(core::mem::size_of::<Self>()) } } }



pub struct WasmRef<'a, T: CType> {
    ptr: *mut T,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, T: CType> WasmRef<'a, T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { self.ptr.read_unaligned() }
    }

    #[inline(always)]
    pub fn write(&self, value: T) {
        unsafe { write_ctype(self.ptr, value) }
    }
}


pub struct WasmSlice<'a, T: CType> {
    ptr: *mut T,
    len: usize,
    phantom: PhantomData<&'a mut [T]>,
}

impl<'a, T: CType> WasmSlice<'a, T> {

    #[inline(always)]
    pub fn iter(&self) -> WasmSliceIter<T> {
        let end = unsafe { self.ptr.add(self.len) };
        WasmSliceIter { ptr: self.ptr, end, phantom: PhantomData }
    }
}


pub struct WasmSliceIter<'a, T: CType> {
    ptr: *mut T,
    end: *mut T,
    phantom: PhantomData<&'a mut [T]>,
}

impl<'a, T: CType> Iterator for WasmSliceIter<'a, T> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr != self.end {
            let result = unsafe { self.ptr.read_unaligned() };
            self.ptr = unsafe { self.ptr.add(1) };
            return Some(result);
        }
        None
    }

    #[inline(always)]
    fn nth(&mut self, i: usize) -> Option<Self::Item> {
        if i < self.end as usize - self.ptr as usize {
            self.ptr = unsafe { self.ptr.add(i) };

            let result = unsafe { self.ptr.read_unaligned() };
            self.ptr = unsafe { self.ptr.add(1) };
            return Some(result);
        }
        None
    }

    #[inline(always)]
    fn last(self) -> Option<Self::Item> {
        if self.ptr != self.end {
            let last = unsafe { self.end.sub(1) };
            return Some(unsafe { last.read_unaligned() });
        }
        None
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end as usize - self.ptr as usize;
        (len, Some(len))
    }
}



pub struct MemoryView<'a> {
    base: *mut u8,
    size: usize,

    phantom: PhantomData<&'a ()>,
}

impl<'a> MemoryView<'a> {
    #[inline(always)]
    pub(crate) fn new_unsafe(base: *mut u8, size: usize) -> Self {
        MemoryView { base, size, phantom: PhantomData }
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline(always)]
    fn check_range(&self, begin: usize, len: usize) -> Result<(), ()> {
        let end = begin.checked_add(len).ok_or(())?;
        if end > self.size {
            return Err(());
        }
        Ok(())
    }

    pub fn deref<T: CType>(&self, ptr: WasmPtr<T>) -> Result<WasmRef<T>, ()> {
        let begin = ptr.addr as usize;
        self.check_range(begin, core::mem::size_of::<T>())?;

        Ok(WasmRef{ ptr: unsafe { self.base.add(begin) as *mut T }, phantom: PhantomData })
    }

    pub fn read<T: CType>(&self, ptr: WasmPtr<T>) -> Result<T, ()> {
        let begin = ptr.addr as usize;
        self.check_range(begin, core::mem::size_of::<T>())?;

        Ok(unsafe { (self.base.add(begin) as *const T).read_unaligned() })
    }

    pub fn write<T: CType>(&mut self, ptr: WasmPtr<T>, value: T) -> Result<(), ()> {
        let begin = ptr.addr as usize;
        self.check_range(begin, core::mem::size_of::<T>())?;

        unsafe { write_ctype(self.base.add(begin) as *mut T, value) }

        Ok(())
    }

    pub fn slice<T: CType>(&self, ptr: WasmPtr<T>, len: WasmSize) -> Result<WasmSlice<T>, ()> {
        let begin = ptr.addr as usize;

        let size = core::mem::size_of::<T>().checked_mul(len.usize()).ok_or(())?;
        self.check_range(begin, size)?;

        let ptr = unsafe { self.base.add(begin) as *mut T };
        Ok(WasmSlice { ptr, len: len.usize(), phantom: PhantomData })
    }

    pub fn bytes(&self, ptr: WasmPtr<u8>, len: WasmSize) -> Result<&[u8], ()> {
        let begin = ptr.addr as usize;
        self.check_range(begin, len.usize())?;

        Ok(unsafe { core::slice::from_raw_parts(self.base.add(begin), len.usize()) })
    }

    pub fn bytes_mut(&mut self, ptr: WasmPtr<u8>, len: WasmSize) -> Result<&mut [u8], ()> {
        let begin = ptr.addr as usize;
        self.check_range(begin, len.usize())?;

        Ok(unsafe { core::slice::from_raw_parts_mut(self.base.add(begin), len.usize()) })
    }

    pub fn copy(&mut self, dst: WasmPtr<u8>, src: WasmPtr<u8>, len: WasmSize) -> Result<(), ()> {
        let dst = dst.addr as usize;
        let src = src.addr as usize;
        let len = len.usize();
        self.check_range(dst, len)?;
        self.check_range(src, len)?;

        unsafe {
            let dst = self.base.add(dst);
            let src = self.base.add(src);
            core::ptr::copy(src, dst, len);
        }
        Ok(())
    }

    pub fn fill(&mut self, dst: WasmPtr<u8>, val: u8, len: WasmSize) -> Result<(), ()> {
        let dst = dst.addr as usize;
        let len = len.usize();
        self.check_range(dst, len)?;

        unsafe {
            let dst = self.base.add(dst);
            core::ptr::write_bytes(dst, val, len);
        }
        Ok(())
    }
}

