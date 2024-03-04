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
use crate::interp::{self, InterpFunc};


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
    modules: ManualVec<ModuleData>,
    instances: ManualVec<InstanceData>,
    funcs: ManualVec<FuncData>,
    memories: ManualVec<Box<UnsafeCell<MemoryData>>>,
}

pub(crate) struct ModuleData {
    pub alloc: Arena,
    pub wasm: wasm::Module<'static>,
    pub funcs: ManualVec<u32>,
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

impl Store {
    pub fn new() -> Self {
        Self {
            modules: ManualVec::new(),
            instances: ManualVec::new(),
            funcs: ManualVec::new(),
            memories: ManualVec::new(),
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

        let mut funcs = ManualVec::new();

        for (i, code) in module.codes.iter().enumerate() {
            let mut p = wasm::Parser::from_sub_section(wasm, code.expr);

            validator.begin_func(module.funcs[i], code.locals).unwrap();
            compiler.begin_func();

            while !p.is_done() {
                let ((), ()) =
                    p.parse_operator_with(wasm::AndThenOp(&mut validator, &mut compiler))
                    .map_err(|_| todo!())?
                    .map_err(|_| todo!())?;
            }

            let func_id: u32 = self.funcs.len().try_into().map_err(|_| Error::OutOfMemory)?;
            funcs.push_or_alloc(func_id).map_err(|_| Error::OutOfMemory)?;

            let ty = module.types[module.funcs[i] as usize];
            let ty = unsafe { core::mem::transmute::<wasm::FuncType, wasm::FuncType>(ty) };

            self.funcs.push_or_alloc(FuncData {
                ty,
                kind: FuncKind::Interp(InterpFunc {
                    stack_size: validator.max_stack(),
                    code: compiler.code(GlobalAlloc).ok_or_else(|| Error::OutOfMemory)?,
                }),
            }).map_err(|_| Error::OutOfMemory)?;
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
        for id in module.funcs.copy_it() {
            funcs.push(id).unwrap_debug();
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

        if args.len() != func.ty.params.len() {
            todo!()
        }
        for i in 0..args.len() {
            if args[i].ty() != func.ty.params[i] {
                todo!()
            }
        }

        if rets.len() != func.ty.rets.len() {
            todo!()
        }


        // push args onto stack.
        // call interp.
        // pop rets from stack.

        todo!()
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


