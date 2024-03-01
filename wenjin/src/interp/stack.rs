use crate::value::Value;
use crate::OptInstanceId;
use crate::interp::bytecode::Word;
use crate::interp::OptInterpFuncId;


#[cfg(wenjin_paranoia = "yas")]
#[derive(Clone, Copy, Debug)]
pub struct StackValue(Value);

#[allow(non_snake_case)]
#[cfg(wenjin_paranoia = "yas")]
impl StackValue {
    #[inline(always)]
    pub fn I32(value: i32) -> StackValue { StackValue(Value::I32(value)) }

    #[inline(always)]
    pub fn U32(value: u32) -> StackValue { StackValue(Value::I32(value as i32)) }

    #[inline(always)]
    pub fn I64(value: i64) -> StackValue { StackValue(Value::I64(value)) }

    #[inline(always)]
    pub fn U64(value: u64) -> StackValue { StackValue(Value::I64(value as i64)) }

    #[inline(always)]
    pub fn F32(value: f32) -> StackValue { StackValue(Value::F32(value)) }

    #[inline(always)]
    pub fn F64(value: f64) -> StackValue { StackValue(Value::F64(value)) }


    #[inline(always)]
    pub fn i32(&self) -> i32 {
        match self.0 { Value::I32(v) => v, _ => unreachable!() }
    }

    #[inline(always)]
    pub fn u32(&self) -> u32 { self.i32() as u32 }

    #[inline(always)]
    pub fn i64(&self) -> i64 {
        match self.0 { Value::I64(v) => v, _ => unreachable!() }
    }

    #[inline(always)]
    pub fn u64(&self) -> u64 { self.i64() as u64 }

    #[inline(always)]
    pub fn f32(&self) -> f32 {
        match self.0 { Value::F32(v) => v, _ => unreachable!() }
    }

    #[inline(always)]
    pub fn f64(&self) -> f64 {
        match self.0 { Value::F64(v) => v, _ => unreachable!() }
    }
}

#[cfg(wenjin_paranoia = "yas")]
impl Value {
    #[inline]
    pub(crate) fn to_stack_value(self) -> StackValue { StackValue(self) }
}


#[cfg(not(wenjin_paranoia = "yas"))]
#[derive(Clone, Copy)]
union RawValue {
    i32: i32,
    i64: i64,
    f32: f32,
    f64: f64,
}

#[cfg(not(wenjin_paranoia = "yas"))]
#[derive(Clone, Copy)]
pub struct StackValue(RawValue);

#[allow(non_snake_case)]
#[cfg(not(wenjin_paranoia = "yas"))]
impl StackValue {
    #[inline(always)]
    pub fn I32(value: i32) -> StackValue { StackValue(RawValue { i32: value }) }

    #[inline(always)]
    pub fn U32(value: u32) -> StackValue { StackValue(RawValue { i32: value as i32 }) }

    #[inline(always)]
    pub fn I64(value: i64) -> StackValue { StackValue(RawValue { i64: value }) }

    #[inline(always)]
    pub fn U64(value: u64) -> StackValue { StackValue(RawValue { i64: value as i64 }) }

    #[inline(always)]
    pub fn F32(value: f32) -> StackValue { StackValue(RawValue { f32: value }) }

    #[inline(always)]
    pub fn F64(value: f64) -> StackValue { StackValue(RawValue { f64: value }) }

    #[inline(always)]
    pub fn i32(&self) -> i32 {
        unsafe { self.0.i32 }
    }

    #[inline(always)]
    pub fn u32(&self) -> u32 { self.i32() as u32 }

    #[inline(always)]
    pub fn i64(&self) -> i64 {
        unsafe { self.0.i64 }
    }

    #[inline(always)]
    pub fn u64(&self) -> u64 { self.i64() as u64 }

    #[inline(always)]
    pub fn f32(&self) -> f32 {
        unsafe { self.0.f32 }
    }

    #[inline(always)]
    pub fn f64(&self) -> f64 {
        unsafe { self.0.f64 }
    }
}

#[cfg(not(wenjin_paranoia = "yas"))]
impl Value {
    #[inline]
    pub(crate) fn to_stack_value(self) -> StackValue {
        match self {
            Value::I32(value) => StackValue::I32(value),
            Value::I64(value) => StackValue::I64(value),
            Value::F32(value) => StackValue::F32(value),
            Value::F64(value) => StackValue::F64(value),
        }
    }
}


pub(crate) struct StackPtr {
    ptr: *mut StackValue,
}

impl StackPtr {
    #[inline(always)]
    pub fn from_ptr(ptr: *mut StackValue) -> StackPtr {
        StackPtr { ptr }
    }

    #[inline(always)]
    pub unsafe fn new(stack: &mut [StackValue], index: usize) -> StackPtr {
        debug_assert!(index < stack.len());
        StackPtr { ptr: unsafe { stack.as_mut_ptr().add(index) } }
    }

    #[inline(always)]
    pub fn read(&self) -> StackValue {
        unsafe { *self.ptr }
    }

    #[inline(always)]
    pub fn write(&self, value: StackValue) {
        unsafe { *self.ptr = value }
    }
}

impl core::ops::Deref for StackPtr {
    type Target = StackValue;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}


#[derive(Clone)]
pub(crate) struct Frame {
    pub(crate) instance: OptInstanceId,
    pub(crate) func: OptInterpFuncId,
    pub(crate) pc: *const Word,
    pub(crate) bp: usize,
}

impl Frame {
    #[inline]
    pub fn native() -> Frame {
        Frame {
            instance: None.into(),
            func: None.into(),
            pc: core::ptr::null(),
            bp: usize::MAX,
        }
    }
}


