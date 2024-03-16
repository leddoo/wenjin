use core::ptr::NonNull;
use core::cell::UnsafeCell;
use core::marker::PhantomData;

use sti::traits::UnwrapDebug;
use sti::manual_vec::ManualVec;

use wasm::{Limits, RefType};

use crate::Error;
use crate::store::RefValue;



pub(crate) struct TableData {
    ty: RefType,
    limits: Limits,
    values: ManualVec<RefValue>,
}

impl TableData {
    pub fn new(ty: RefType, limits: Limits, default: RefValue) -> Result<Self, Error> {
        let mut this = Self {
            ty,
            limits,
            values: ManualVec::new(),
        };

        this.grow(limits.min, default).map_err(|_| Error::OOM)?;

        return Ok(this);
    }

    fn grow(&mut self, delta: u32, default: RefValue) -> Result<(), ()> {
        let Some(new_len) = (self.values.len() as u32).checked_add(delta) else { return Err(()) };

        if let Some(max_len) = self.limits.max {
            if new_len > max_len {
                return Err(());
            }
        }

        self.values.reserve_extra(delta as usize).map_err(|_| ())?;
        for _ in 0..delta {
            self.values.push(default).unwrap_debug();
        }

        return Ok(());
    }
}


pub struct Table<'a> {
    inner: NonNull<TableData>,
    phantom: PhantomData<&'a mut TableData>,
}

impl<'a> Table<'a> {
    #[inline]
    pub(crate) fn new(table: &UnsafeCell<TableData>) -> Self {
        Self { inner: NonNull::from(table).cast(), phantom: PhantomData }
    }

    #[inline]
    pub fn as_slice(&self) -> &[RefValue] {
        unsafe { &self.inner.as_ref().values }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [RefValue] {
        unsafe { &mut self.inner.as_mut().values }
    }
}


