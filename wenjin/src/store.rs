//use core::num::NonZeroU32;
use core::cell::UnsafeCell;
use core::ptr::NonNull;

use sti::alloc::GlobalAlloc;
use sti::arena::Arena;
use sti::boks::Box;
use sti::manual_vec::ManualVec;

use wasm::{Parser, Validator};

use crate::Error;
use crate::memory::{MemoryData, Memory};
use crate::interp::Compiler;


pub struct ModuleId { id: u32 }
pub struct InstanceId { id: u32 }
pub struct FuncId { id: u32 }
pub struct TableId { id: u32 }
pub struct MemoryId { id: u32 }
pub struct GlobalId { id: u32 }


pub enum Extern {
    Func(FuncId),
    Table(FuncId),
    Memory(FuncId),
    Global(FuncId),
}


pub struct Store {
    memories: ManualVec<Box<UnsafeCell<MemoryData>>>
}

impl Store {
    pub fn new() -> Self {
        Self {
            memories: ManualVec::new(),
        }
    }

    pub fn new_module(&mut self, wasm: &[u8]) -> Result<ModuleId, Error> {
        let alloc = Arena::new();

        let module = Parser::parse_module(wasm, Default::default(), &alloc)
            .map_err(|_| todo!())?;

        let mut validator = Validator::new(&module);
        let mut compiler = Compiler::new();

        for (i, code) in module.codes.iter().enumerate() {
            let mut p = Parser::from_sub_section(wasm, code.expr);

            validator.begin_func(module.funcs[i], code.locals).unwrap();
            compiler.begin_func();

            while !p.is_done() {
                let ((), ()) =
                    p.parse_operator_with(wasm::AndThenOp(&mut validator, &mut compiler))
                    .map_err(|_| todo!())?
                    .map_err(|_| todo!())?;
            }
        }

        todo!()
    }

    pub fn new_instance(&mut self, module: ModuleId, imports: &[(&str, &str, Extern)]) -> Result<ModuleId, Error> {
        todo!()
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


