//use core::num::NonZeroU32;
use core::cell::UnsafeCell;
use core::ptr::NonNull;

use sti::alloc::GlobalAlloc;
use sti::boks::Box;
use sti::manual_vec::ManualVec;

use crate::Error;
use crate::memory::{Memory, MemoryCtx};


pub struct MemoryId {
    id: u32,
    //gen: NonZeroU32
}


pub struct Store {
    memories: ManualVec<Box<UnsafeCell<Memory>>>
}

impl Store {
    pub fn new() -> Self {
        Self {
            memories: ManualVec::new(),
        }
    }

    pub fn new_memory(&mut self, limits: wasm::Limits) -> Result<MemoryId, Error> {
        let id = MemoryId {
            id: self.memories.len().try_into().map_err(|_| Error::OutOfMemory)?,
            //gen: NonZeroU32::MIN,
        };

        let memory = Memory::new(limits).map_err(|_| Error::OutOfMemory)?;

        let memory = Box::try_new_in(GlobalAlloc, UnsafeCell::new(memory)).ok_or_else(|| Error::OutOfMemory)?;

        self.memories.push_or_alloc(memory).map_err(|_| Error::OutOfMemory)?;

        return Ok(id);
    }

    pub fn memory<'a>(&'a self, id: MemoryId) -> Result<MemoryCtx<'a>, Error> {
        Ok(MemoryCtx::new(box_to_nonnull(
            self.memories.get(id.id as usize)
            .ok_or(Error::InvalidHandle)?)))
    }
}

#[inline(always)]
fn box_to_nonnull<T>(boks: &Box<UnsafeCell<T>>) -> NonNull<T> {
    (boks.inner() as NonNull<UnsafeCell<T>>).cast()
}


