use core::ptr::NonNull;
use core::marker::PhantomData;

use sti::alloc::{Alloc, GlobalAlloc, Layout};

use wasm::Limits;

use crate::Error;



pub(crate) struct Memory {
    limits: Limits,
    buffer: NonNull<u8>,
    size_pages: u32,
}

const ALIGN: usize = 16;

impl Memory {
    pub fn new(limits: Limits) -> Result<Self, ()> {
        let mut this = Self {
            limits,
            buffer: NonNull::dangling(),
            size_pages: 0,
        };

        this.grow(limits.min)?;

        return Ok(this);
    }

    #[inline]
    fn size_bytes(&self) -> usize {
        self.size_pages as usize * wasm::PAGE_SIZE
    }

    fn grow(&mut self, by_pages: u32) -> Result<u32, ()> {
        let old_pages  = self.size_pages;
        let old_size   = old_pages as usize * wasm::PAGE_SIZE;
        let old_layout = unsafe { Layout::from_size_align_unchecked(old_size, ALIGN) };

        let Some(new_pages) = old_pages.checked_add(by_pages) else { return Err(()) };
        if let Some(max_pages) = self.limits.max {
            if new_pages > max_pages {
                return Err(());
            }
        }

        let Some(new_size) = (new_pages as usize).checked_mul(wasm::PAGE_SIZE) else { return Err(()) };
        let Ok(new_layout) = Layout::from_size_align(new_size, ALIGN)          else { return Err(()) };

        // allocate.
        let Some(new_buffer) = (unsafe { GlobalAlloc.realloc(self.buffer, old_layout, new_layout) }) else { return Err(()); };

        // zero init.
        unsafe {
            core::ptr::write_bytes(
                new_buffer.as_ptr().add(old_size),
                0x00,
                new_size - old_size);
        }

        self.buffer = new_buffer;
        self.size_pages = new_pages;

        return Ok(old_pages);
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        let size = self.size_bytes();
        unsafe {
            GlobalAlloc.free(
                self.buffer,
                Layout::from_size_align_unchecked(size, ALIGN));
        }
    }
}


pub struct MemoryCtx<'a> {
    memory: NonNull<Memory>,
    phantom: PhantomData<&'a mut Memory>,
}

impl<'a> MemoryCtx<'a> {
    #[inline]
    pub(crate) fn new(memory: NonNull<Memory>) -> Self {
        Self { memory, phantom: PhantomData }
    }

    #[inline]
    pub fn size_pages(&self) -> u32 {
        unsafe { self.memory.as_ref().size_pages }
    }

    #[inline]
    pub fn size_bytes(&self) -> usize {
        unsafe { self.memory.as_ref().size_bytes() }
    }

    #[inline]
    pub fn grow(&mut self, by_pages: u32) -> Result<u32, Error> {
        unsafe { self.memory.as_mut().grow(by_pages) }
        .map_err(|_| todo!())
    }
}


