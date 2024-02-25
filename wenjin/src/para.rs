
#[macro_export]
macro_rules! para_assert {
    ($expr:expr) => {
        #[cfg(wenjin_paranoia = "yas")]
        assert!($expr);
    };
}

#[macro_export]
macro_rules! para_assert_eq {
    ($a:expr, $b:expr) => {
        #[cfg(wenjin_paranoia = "yas")]
        assert_eq!($a, $b);
    };
}



pub struct ParaPtr<T: Copy> {
    ptr: *const T,

    #[cfg(wenjin_paranoia = "yas")]
    len: usize,
}

impl<T: Copy> ParaPtr<T> {
    #[inline(always)]
    pub fn new(slice: &[T]) -> ParaPtr<T> {
        ParaPtr {
            ptr: slice.as_ptr(),

            #[cfg(wenjin_paranoia = "yas")]
            len: slice.len(),
        }
    }

    #[inline(always)]
    pub unsafe fn read(&self, index: usize) -> T {
        para_assert!(index < self.len);
        unsafe { *self.ptr.add(index) }
    }
}



pub struct ParaSliceMut<'a, T: Copy> {
    pub slice: &'a mut [T],
}

impl<'a, T: Copy> ParaSliceMut<'a, T> {
    #[inline(always)]
    pub fn new(slice: &'a mut [T]) -> ParaSliceMut<'a, T> {
        ParaSliceMut { slice }
    }

    #[inline(always)]
    pub fn len(&self) -> usize { self.slice.len() }

    #[inline(always)]
    pub fn para_slice_mut(&mut self, range: core::ops::Range<usize>) -> ParaSliceMut<T> {
        para_assert!(range.start + range.len() <= self.slice.len());
        ParaSliceMut {
            slice: unsafe { core::slice::from_raw_parts_mut(
                self.slice.as_mut_ptr().add(range.start), range.len()) },
        }
    }

    #[inline(always)]
    pub unsafe fn assume_len(&self, len: usize) {
        para_assert_eq!(self.len(), len);
        if self.len() != len {
            unsafe { core::hint::unreachable_unchecked() }
        }
    }

    #[inline(always)]
    pub fn fill(&mut self, value: T) {
        unsafe {
            let mut at = self.slice.as_mut_ptr();
            let end = at.add(self.len());

            while at < end {
                at.write(value);
                at = at.add(1);
            }
        }
    }
}

