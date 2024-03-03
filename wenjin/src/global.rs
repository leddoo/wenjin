//use core::ptr::NonNull;
//use core::marker::PhantomData;

use crate::Value;


pub(crate) struct GlobalData {
    value: Value,
    mutable: bool,
}


