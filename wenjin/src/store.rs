//use core::num::NonZeroU32;
use core::cell::UnsafeCell;
use core::ptr::NonNull;

use sti::traits::{CopyIt, UnwrapDebug};
use sti::alloc::GlobalAlloc;
use sti::arena::Arena;
use sti::boks::Box;
use sti::manual_vec::ManualVec;

use crate::{Error, Value};
use crate::memory::{MemoryData, Memory};
use crate::interp;


pub struct ModuleId { id: u32 }
pub struct InstanceId { id: u32 }
pub struct FuncId { id: u32 }
pub struct TableId { id: u32 }
pub struct MemoryId { id: u32 }
pub struct GlobalId { id: u32 }


pub enum Extern {
    Func(FuncId),
    Table(TableId),
    Memory(MemoryId),
    Global(GlobalId),
}


pub struct Store {
    pub(crate) modules: ManualVec<ModuleData>,
    pub(crate) instances: ManualVec<InstanceData>,
    pub(crate) funcs: ManualVec<FuncData>,
    pub(crate) memories: ManualVec<Box<UnsafeCell<MemoryData>>>,
    pub(crate) thread: ThreadData,
}


pub(crate) struct ModuleData {
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
}

pub(crate) struct InterpFunc {
    pub instance: u32,
    pub code: *mut u8,
    pub code_end: *const u8,
    pub stack_size: u32,
    pub num_locals: u32,
}


#[derive(Clone, Copy)]
#[repr(align(8))]
pub(crate) struct StackValue {
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

pub(crate) struct StackFrame {
    pub pc: NonNull<u8>,
    pub bp: usize,
}

pub(crate) struct ThreadData {
    pub stack: ManualVec<StackValue>,
    pub frames: ManualVec<Option<StackFrame>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            modules: ManualVec::new(),
            instances: ManualVec::new(),
            funcs: ManualVec::new(),
            memories: ManualVec::new(),
            thread: ThreadData {
                stack: ManualVec::new(),
                frames: ManualVec::new(),
            },
        }
    }

    pub fn new_module(&mut self, wasm: &[u8]) -> Result<ModuleId, Error> {
        let id = ModuleId {
            id: self.modules.len().try_into().map_err(|_| Error::OutOfMemory)?,
        };

        let alloc = Arena::new();

        let module = wasm::Parser::parse_module(wasm, Default::default(), &alloc)
            .map_err(|_| todo!())?;

        let mut validator = wasm::Validator::new(&module);
        let mut compiler = interp::Compiler::new();

        let mut funcs = ManualVec::with_cap(module.codes.len()).ok_or_else(|| Error::OutOfMemory)?;

        for (i, code) in module.codes.iter().enumerate() {
            let mut p = wasm::Parser::from_sub_section(wasm, code.expr);

            let ty_idx = module.funcs[i];
            let ty = module.types[module.funcs[i] as usize];

            validator.begin_func(ty_idx, code.locals).unwrap();
            compiler.begin_func(ty.rets.len() as u32);

            while !p.is_done() {
                let ((), ()) =
                    p.parse_operator_with(wasm::AndThenOp(&mut validator, &mut compiler))
                    .map_err(|_| todo!())?
                    .map_err(|_| todo!())?;
            }


            let ty = unsafe { core::mem::transmute::<wasm::FuncType, wasm::FuncType>(ty) };
            funcs.push(ModuleFunc {
                ty,
                code: compiler.code(GlobalAlloc).ok_or_else(|| Error::OutOfMemory)?,
                stack_size: validator.num_locals() + validator.max_stack(),
                num_locals: validator.num_locals(),
            }).unwrap_debug();
        }

        let module = unsafe { core::mem::transmute::<wasm::Module, wasm::Module>(module) };
        self.modules.push_or_alloc(ModuleData {
            alloc,
            wasm: module,
            funcs,
        }).map_err(|_| Error::OutOfMemory)?;

        return Ok(id);
    }

    pub fn new_instance(&mut self, module_id: ModuleId, imports: &[(&str, &str, Extern)]) -> Result<InstanceId, Error> {
        let module = self.modules.get(module_id.id as usize).ok_or_else(|| todo!())?;

        let id = InstanceId {
            id: self.instances.len().try_into().map_err(|_| Error::OutOfMemory)?,
        };

        if imports.len() > 0 || module.wasm.imports.imports.len() > 0 {
            todo!()
        }

        let mut funcs = ManualVec::with_cap(module.funcs.len()).ok_or_else(|| Error::OutOfMemory)?;
        self.funcs.reserve_extra(module.funcs.len()).map_err(|_| Error::OutOfMemory)?;
        if u32::try_from(self.funcs.len() + module.funcs.len()).is_err() {
            return Err(Error::OutOfMemory);
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
                    num_locals: func.num_locals,
                }),
            }).unwrap_debug();
        }

        let tables = ManualVec::new();

        let mut memories = ManualVec::with_cap(module.wasm.memories.len()).ok_or_else(|| Error::OutOfMemory)?;
        for mem in module.wasm.memories {
            memories.push(self.new_memory(mem.limits)?.id).unwrap_debug();
        }

        let globals = ManualVec::new();

        self.instances.push_or_alloc(InstanceData {
            module: module_id.id,
            funcs,
            tables,
            memories,
            globals,
        }).map_err(|_| Error::OutOfMemory)?;

        return Ok(id);
    }

    pub fn get_export(&self, instance_id: InstanceId, name: &str) -> Result<Extern, Error> {
        let instance = self.instances.get(instance_id.id as usize).ok_or_else(|| todo!())?;

        let module = &self.modules[instance.module as usize];
        for export in module.wasm.exports {
            if export.name == name {
                return Ok(match export.kind {
                    wasm::ExportKind::Func(idx) => Extern::Func(FuncId { id: instance.funcs[idx as usize] }),
                    wasm::ExportKind::Table(_) => todo!(),
                    wasm::ExportKind::Memory(_) => todo!(),
                    wasm::ExportKind::Global(_) => todo!(),
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

    pub fn call_dyn(&mut self, func_id: FuncId, args: &[Value], rets: &mut [Value]) -> Result<(), Error> {
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

        if rets.len() != ty.rets.len() {
            todo!()
        }


        // push args onto stack.
        let bp = self.thread.stack.len();
        self.thread.stack.reserve_extra(args.len()).map_err(|_| Error::OutOfMemory)?;
        for arg in args {
            self.thread.stack.push(StackValue::from_value(*arg)).unwrap_debug();
        }

        self.run_func(func_id.id, bp)?;

        // pop rets from stack.
        for i in 0..ty.rets.len() {
            rets[i] = self.thread.stack[bp + i].to_value(ty.rets[i]);
        }

        return Ok(());
    }

    fn run_func(&mut self, id: u32, bp: usize) -> Result<(), Error> {
        let func = &self.funcs[id as usize];

        match &func.kind {
            FuncKind::Interp(f) => {
                let num_params = self.thread.stack.len() - bp;
                self.thread.stack.reserve(bp + f.stack_size as usize).map_err(|_| Error::OutOfMemory)?;

                for _ in num_params..f.num_locals as usize {
                    self.thread.stack.push(StackValue::ZERO).unwrap_debug();
                }

                self.thread.frames.push_or_alloc(None).map_err(|_| Error::OutOfMemory)?;

                let state = interp::State::new(f, bp, &mut self.thread);
                self.run_interp(state)?;

                Ok(())
            }
        }
    }

    pub fn new_memory(&mut self, limits: wasm::Limits) -> Result<MemoryId, Error> {
        let id = MemoryId {
            id: self.memories.len().try_into().map_err(|_| Error::OutOfMemory)?,
        };

        let memory = MemoryData::new(limits).map_err(|_| Error::OutOfMemory)?;

        let memory = Box::try_new_in(GlobalAlloc, UnsafeCell::new(memory)).ok_or_else(|| Error::OutOfMemory)?;

        self.memories.push_or_alloc(memory).map_err(|_| Error::OutOfMemory)?;

        return Ok(id);
    }

    pub fn memory<'a>(&'a self, id: MemoryId) -> Result<Memory<'a>, Error> {
        Ok(Memory::new(box_to_nonnull(
            self.memories.get(id.id as usize)
            .ok_or(Error::InvalidHandle)?)))
    }
}

#[inline(always)]
fn box_to_nonnull<T>(boks: &Box<UnsafeCell<T>>) -> NonNull<T> {
    (boks.inner() as NonNull<UnsafeCell<T>>).cast()
}


