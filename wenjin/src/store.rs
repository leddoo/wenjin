//use core::num::NonZeroU32;
use core::cell::UnsafeCell;
use core::ptr::NonNull;

use sti::traits::{CopyIt, UnwrapDebug};
use sti::alloc::GlobalAlloc;
use sti::arena::Arena;
use sti::boks::Box;
use sti::manual_vec::ManualVec;

use crate::Error;
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
    interp_funcs: ManualVec<InterpFunc>,
    memories: ManualVec<Box<UnsafeCell<MemoryData>>>,
}

pub(crate) struct ModuleData {
    pub alloc: Arena,
    pub wasm: wasm::Module<'static>,
    pub interp_funcs: ManualVec<u32>,
}

pub(crate) enum FuncRef {
    Interp(u32),
    Host(u32),
}

pub(crate) struct InstanceData {
    pub module: u32,
    pub funcs: ManualVec<FuncRef>,
    pub tables: ManualVec<u32>,
    pub memories: ManualVec<u32>,
    pub globals: ManualVec<u32>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            modules: ManualVec::new(),
            instances: ManualVec::new(),
            interp_funcs: ManualVec::new(),
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

        let mut interp_funcs = ManualVec::new();

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

            let func_id: u32 = self.interp_funcs.len().try_into().map_err(|_| Error::OutOfMemory)?;
            interp_funcs.push_or_alloc(func_id).map_err(|_| Error::OutOfMemory)?;

            self.interp_funcs.push_or_alloc(InterpFunc {
                stack_size: validator.max_stack(),
                code: compiler.code(GlobalAlloc).ok_or_else(|| Error::OutOfMemory)?,
            }).map_err(|_| Error::OutOfMemory)?;
        }

        let module = unsafe { core::mem::transmute::<wasm::Module, wasm::Module>(module) };
        self.modules.push_or_alloc(ModuleData {
            alloc,
            wasm: module,
            interp_funcs,
        }).map_err(|_| Error::OutOfMemory)?;

        return Ok(id);
    }

    pub fn new_instance(&mut self, module_id: ModuleId, imports: &[(&str, &str, Extern)]) -> Result<InstanceId, Error> {
        let module = self.modules.get(module_id.id as usize).ok_or_else(|| todo!())?;

        let id = InstanceId {
            id: self.modules.len().try_into().map_err(|_| Error::OutOfMemory)?,
        };

        if imports.len() > 0 || module.wasm.imports.imports.len() > 0 {
            todo!()
        }

        let mut funcs = ManualVec::with_cap(module.interp_funcs.len()).ok_or_else(|| Error::OutOfMemory)?;
        for id in module.interp_funcs.copy_it() {
            funcs.push(FuncRef::Interp(id)).unwrap_debug();
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

    pub fn new_memory(&mut self, limits: wasm::Limits) -> Result<MemoryId, Error> {
        let id = MemoryId {
            id: self.memories.len().try_into().map_err(|_| Error::OutOfMemory)?,
            //gen: NonZeroU32::MIN,
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


