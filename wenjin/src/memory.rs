use core::ptr::NonNull;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::size_of;

use sti::alloc::{Alloc, GlobalAlloc, Layout};
use sti::vec::Vec;

use wasm::Limits;

use crate::{Error, WasmType};
use crate::store::StackValue;



pub(crate) struct MemoryData {
    limits: Limits,
    buffer: NonNull<u8>,
    size_pages: u32,
}

const ALIGN: usize = 16;

impl MemoryData {
    pub fn new(limits: Limits) -> Result<Self, Error> {
        let mut this = Self {
            limits,
            buffer: NonNull::dangling(),
            size_pages: 0,
        };

        this.grow(limits.min).map_err(|_| Error::OutOfMemory)?;

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

impl Drop for MemoryData {
    fn drop(&mut self) {
        let size = self.size_bytes();
        unsafe {
            GlobalAlloc.free(
                self.buffer,
                Layout::from_size_align_unchecked(size, ALIGN));
        }
    }
}


#[derive(Clone, Copy)]
pub struct Memory<'a> {
    inner: NonNull<MemoryData>,
    phantom: PhantomData<&'a mut MemoryData>,
}

impl<'a> Memory<'a> {
    #[inline]
    pub(crate) fn new(memory: &UnsafeCell<MemoryData>) -> Self {
        Self { inner: NonNull::from(memory).cast(), phantom: PhantomData }
    }

    #[inline]
    pub fn size_pages(&self) -> u32 {
        unsafe { self.inner.as_ref().size_pages }
    }

    #[inline]
    pub fn size_bytes(&self) -> usize {
        unsafe { self.inner.as_ref().size_bytes() }
    }

    #[inline]
    pub fn grow(&mut self, by_pages: u32) -> Result<u32, Error> {
        unsafe { self.inner.as_mut().grow(by_pages) }
        .map_err(|_| Error::OutOfMemory)
    }

    #[inline]
    pub fn read<T: CType>(&self, ptr: WasmPtr<T>) -> Result<T, Error> {
        let Some(end) = (ptr.addr as usize).checked_add(size_of::<T>()) else { todo!() };
        if end > self.size_bytes() { todo!() }
        unsafe {
            let base = self.inner.as_ref().buffer.as_ptr();
            return Ok(base.add(ptr.addr as usize).cast::<T>().read_unaligned());
        }
    }

    #[inline]
    pub fn write<T: CType>(&mut self, ptr: WasmPtr<T>, value: T) -> Result<(), Error> {
        let Some(end) = (ptr.addr as usize).checked_add(size_of::<T>()) else { todo!() };
        if end > self.size_bytes() { todo!() }
        unsafe {
            let base = self.inner.as_mut().buffer.as_ptr();
            let ptr = base.add(ptr.addr as usize);
            ptr.cast::<T>().write_unaligned(value);
            T::clear_padding(core::slice::from_raw_parts_mut(ptr, size_of::<T>()));
            return Ok(());
        }
    }

    #[inline]
    pub fn parse_cstr(&self, ptr: WasmPtr<u8>) -> Result<WasmSlice<u8>, Error> {
        let mut at = ptr.addr as usize;
        unsafe {
            let mem_size = self.size_bytes();
            let base = self.inner.as_ref().buffer.as_ptr();
            while at < mem_size {
                if *base.add(at) == 0 {
                    return Ok(WasmSlice { ptr, len: WasmSize((at - ptr.addr as usize) as u32) });
                }
                at += 1;
            }
            todo!()
        }
    }

    #[inline]
    pub unsafe fn read_slice(&self, slice: WasmSlice<u8>, dst: *mut u8) -> Result<(), Error> {
        let addr = slice.ptr.addr as usize;
        let len = slice.len.usize();

        let Some(end) = addr.checked_add(len) else { todo!() };
        if end > self.size_bytes() { todo!() }

        unsafe {
            let base = self.inner.as_ref().buffer.as_ptr();
            core::ptr::copy_nonoverlapping(base.add(addr), dst, len);
            return Ok(());
        }
    }

    #[inline]
    pub fn read_slice_to_vec(&self, slice: WasmSlice<u8>, out: &mut Vec<u8>) -> Result<(), Error> {
        let len = slice.len.0 as usize;
        // @todo: oom.
        out.reserve_extra(len);
        unsafe {
            self.read_slice(slice, out.as_mut_ptr().add(out.len()))?;
            out.set_len(out.len() + len)
        }
        return Ok(());
    }

    #[inline]
    pub(crate) fn as_mut_ptr(&mut self) -> (*mut u8, usize) {
        let inner = unsafe { self.inner.as_mut() };
        (inner.buffer.as_ptr(), inner.size_bytes())
    }
}

impl<'a> core::fmt::Debug for Memory<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Memory")
    }
}



/// CType
/// - a trait for types that can safely be read from & written to a wasm memory.
/// - these types must have valid values for all bit patterns (excluding padding).
///   hence, booleans & enums do not implement `CType`.
/// - use `#[derive(Clone, Copy, CType)] #[repr(C)]` to implement `CType` for custom structs.
pub unsafe trait CType: Sized + Copy + 'static {
    /// clear_padding
    /// - to preserve determinism, padding bytes must be cleared,
    ///   when writing values to a memory.
    ///   because in rust, padding bytes have undefined values.
    unsafe fn clear_padding(bytes: &mut [u8]);
}

#[cfg(not(target_endian = "little"))]
compile_error!("non-little endian architectures currently not supported");

unsafe impl CType for u8  { #[inline(always)] unsafe fn clear_padding(bytes: &mut [u8]) { debug_assert_eq!(bytes.len(), size_of::<Self>()) } }
unsafe impl CType for u16 { #[inline(always)] unsafe fn clear_padding(bytes: &mut [u8]) { debug_assert_eq!(bytes.len(), size_of::<Self>()) } }
unsafe impl CType for u32 { #[inline(always)] unsafe fn clear_padding(bytes: &mut [u8]) { debug_assert_eq!(bytes.len(), size_of::<Self>()) } }
unsafe impl CType for u64 { #[inline(always)] unsafe fn clear_padding(bytes: &mut [u8]) { debug_assert_eq!(bytes.len(), size_of::<Self>()) } }

unsafe impl CType for i8  { #[inline(always)] unsafe fn clear_padding(bytes: &mut [u8]) { debug_assert_eq!(bytes.len(), size_of::<Self>()) } }
unsafe impl CType for i16 { #[inline(always)] unsafe fn clear_padding(bytes: &mut [u8]) { debug_assert_eq!(bytes.len(), size_of::<Self>()) } }
unsafe impl CType for i32 { #[inline(always)] unsafe fn clear_padding(bytes: &mut [u8]) { debug_assert_eq!(bytes.len(), size_of::<Self>()) } }
unsafe impl CType for i64 { #[inline(always)] unsafe fn clear_padding(bytes: &mut [u8]) { debug_assert_eq!(bytes.len(), size_of::<Self>()) } }

unsafe impl CType for f32 { #[inline(always)] unsafe fn clear_padding(bytes: &mut [u8]) { debug_assert_eq!(bytes.len(), size_of::<Self>()) } }
unsafe impl CType for f64 { #[inline(always)] unsafe fn clear_padding(bytes: &mut [u8]) { debug_assert_eq!(bytes.len(), size_of::<Self>()) } }


unsafe impl<T: CType, const N: usize> CType for [T; N] {
    #[inline(always)]
    unsafe fn clear_padding(bytes: &mut [u8]) {
        debug_assert_eq!(bytes.len(), size_of::<Self>());
        for i in 0..N {
            unsafe { T::clear_padding(bytes.get_unchecked_mut(i*size_of::<T>()..(i+1)*size_of::<T>())) }
        }
    }
}



#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct WasmSize(pub u32);

impl WasmSize {
    #[inline(always)]
    pub fn usize(self) -> usize { self.0 as usize }
}

impl WasmType for WasmSize {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::I32;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue {
        StackValue::from_i32(self.0 as i32)
    }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> WasmSize {
        WasmSize(value.as_i32() as u32)
    }
}

unsafe impl CType for WasmSize { #[inline(always)] unsafe fn clear_padding(bytes: &mut [u8]) { debug_assert_eq!(bytes.len(), size_of::<Self>()) } }



#[repr(transparent)]
pub struct WasmPtr<T: CType> {
    pub addr: u32,
    pub phantom: PhantomData<*mut T>,
}

impl<T: CType> WasmPtr<T> {
    #[inline(always)]
    pub fn new(addr: u32) -> WasmPtr<T> {
        WasmPtr { addr, phantom: PhantomData }
    }

    #[inline(always)]
    pub fn is_null(self) -> bool {
        self.addr == 0
    }

    #[inline(always)]
    pub fn add(self, delta: u32) -> WasmPtr<T> {
        WasmPtr::new(self.addr + delta*size_of::<T>() as u32)
    }

    #[inline(always)]
    pub fn checked_add(self, delta: u32) -> Option<WasmPtr<T>> {
        let delta = delta.checked_mul(size_of::<T>() as u32)?;
        Some(WasmPtr::new(self.addr.checked_add(delta)?))
    }

    #[inline(always)]
    pub fn wrapping_add(self, delta: u32) -> WasmPtr<T> {
        let delta = delta.wrapping_mul(size_of::<T>() as u32);
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

impl<T: CType> PartialOrd for WasmPtr<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.addr.partial_cmp(&other.addr)
    }
}

impl<T: CType> Ord for WasmPtr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.addr.cmp(&other.addr)
    }
}

impl<T: CType> core::hash::Hash for WasmPtr<T> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.addr.hash(state)
    }
}

impl<T: CType> core::fmt::Debug for WasmPtr<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "0x{:x}", self.addr)
    }
}

impl<T: CType> WasmType for WasmPtr<T> {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::I32;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue {
        StackValue::from_i32(self.addr as i32)
    }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> WasmPtr<T> {
        WasmPtr { addr: value.as_i32() as u32, phantom: PhantomData }
    }
}

unsafe impl<T: CType> CType for WasmPtr<T> { #[inline(always)] unsafe fn clear_padding(bytes: &mut [u8]) { debug_assert_eq!(bytes.len(), size_of::<Self>()) } }



#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct WasmSlice<T: CType> {
    pub ptr: WasmPtr<T>,
    pub len: WasmSize,
}


