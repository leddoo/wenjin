use core::ptr::NonNull;
use core::cell::UnsafeCell;
use core::marker::PhantomData;

use crate::Value;
use crate::store::GlobalId;


pub(crate) struct GlobalData {
    id: GlobalId,
    mutable: bool,
    value: Value,
}

impl GlobalData {
    pub fn new(id: GlobalId, mutable: bool, value: Value) -> GlobalData {
        Self { id, mutable, value }
    }
}


pub struct Global<'a> {
    inner: NonNull<GlobalData>,
    phantom: PhantomData<&'a mut GlobalData>,
}

impl<'a> Global<'a> {
    #[inline]
    pub fn id(&self) -> GlobalId {
        unsafe { self.inner.as_ref().id }
    }

    #[inline]
    pub fn ty(&self) -> wasm::ValueType {
        unsafe { self.inner.as_ref().value.ty() }
    }

    #[inline]
    pub fn mutable(&self) -> bool {
        unsafe { self.inner.as_ref().mutable }
    }

    #[inline]
    pub fn get(&self) -> Value {
        unsafe { self.inner.as_ref().value }
    }


    #[inline]
    pub(crate) fn new(global: &UnsafeCell<GlobalData>) -> Self {
        Self { inner: NonNull::from(global).cast(), phantom: PhantomData }
    }

    // @todo: type check, mut validation.
    #[inline]
    pub(crate) fn set(&mut self, value: Value) {
        unsafe { self.inner.as_mut().value = value }
    }
}


