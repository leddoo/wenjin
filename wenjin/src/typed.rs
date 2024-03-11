use crate::Error;
use crate::store::{Store, StackValue};



pub trait WasmType: Clone + Copy + 'static {
    const WASM_TYPE: wasm::ValueType;

    fn to_stack_value(self) -> StackValue;
    fn from_stack_value(value: StackValue) -> Self;
}

impl WasmType for i32 {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::I32;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue { StackValue::from_i32(self) }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> Self { value.as_i32() }
}

impl WasmType for u32 {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::I32;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue { StackValue::from_i32(self as i32) }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> Self { value.as_i32() as u32 }
}

impl WasmType for i64 {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::I64;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue { StackValue::from_i64(self) }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> Self { value.as_i64() }
}

impl WasmType for u64 {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::I64;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue { StackValue::from_i64(self as i64) }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> Self { value.as_i64() as u64 }
}

impl WasmType for f32 {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::F32;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue { StackValue::from_f32(self) }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> Self { value.as_f32() }
}

impl WasmType for f64 {
    const WASM_TYPE: wasm::ValueType = wasm::ValueType::F64;

    #[inline(always)]
    fn to_stack_value(self) -> StackValue { StackValue::from_f64(self) }

    #[inline(always)]
    fn from_stack_value(value: StackValue) -> Self { value.as_f64() }
}



pub trait WasmTypes {
    const WASM_TYPES: &'static [wasm::ValueType];

    unsafe fn to_stack_values(self, dst: *mut StackValue);
    unsafe fn from_stack_values(src: *const StackValue) -> Self;
}

impl<> WasmTypes for () {
    const WASM_TYPES: &'static [wasm::ValueType] = &[];

    #[inline(always)]
    unsafe fn to_stack_values(self, _dst: *mut StackValue) {}

    #[inline(always)]
    unsafe fn from_stack_values(_src: *const StackValue) -> Self { () }
}

impl<T0: WasmType> WasmTypes for T0 {
    const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE];

    #[inline(always)]
    unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
        dst.add(0).write(self.to_stack_value());
    }}

    #[inline(always)]
    unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe {
        T0::from_stack_value(src.add(0).read())
    }}
}


pub trait WasmResult {
    type Types: WasmTypes;

    fn to_result(self) -> Result<Self::Types, Error>;
}

impl<T: WasmTypes> WasmResult for T {
    type Types = T;

    #[inline(always)]
    fn to_result(self) -> Result<T, Error> { Ok(self) }
}

impl<T: WasmTypes> WasmResult for Result<T, Error> {
    type Types = T;

    #[inline(always)]
    fn to_result(self) -> Result<T, Error> { self }
}



pub unsafe trait HostFunc<Params: WasmTypes, Rets: WasmTypes, const STORE: bool>: 'static + Sized {
    fn call(&self, store: &mut Store) -> Result<(), Error>;
}

unsafe impl<R: WasmResult, F: Fn() -> R + 'static> HostFunc<(), R::Types, false> for F {
    #[inline]
    fn call(&self, store: &mut Store) -> Result<(), Error> {
        let r = (self)().to_result()?;
        let stack = &mut store.thread.stack;
        unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
        unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
        Ok(())
    }
}

unsafe impl<R: WasmResult, F: Fn(&mut Store) -> R + 'static> HostFunc<(), R::Types, true> for F {
    #[inline]
    fn call(&self, store: &mut Store) -> Result<(), Error> {
        let r = (self)(store).to_result()?;
        let stack = &mut store.thread.stack;
        unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
        unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
        Ok(())
    }
}

unsafe impl<T0: WasmType, R: WasmResult, F: Fn(T0) -> R + 'static> HostFunc<T0, R::Types, false> for F {
    #[inline]
    fn call(&self, store: &mut Store) -> Result<(), Error> {
        let stack = &mut store.thread.stack;
        unsafe { stack.set_len(stack.len() - 1) };
        let a0 = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
        let r = (self)(a0).to_result()?;
        let stack = &mut store.thread.stack;
        unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
        unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
        Ok(())
    }
}

unsafe impl<T0: WasmType, R: WasmResult, F: Fn(&mut Store, T0) -> R + 'static> HostFunc<T0, R::Types, true> for F {
    #[inline]
    fn call(&self, store: &mut Store) -> Result<(), Error> {
        let stack = &mut store.thread.stack;
        unsafe { stack.set_len(stack.len() - 1) };
        let a0 = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
        let r = (self)(store, a0).to_result()?;
        let stack = &mut store.thread.stack;
        unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
        unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
        Ok(())
    }
}


mod generated {
    use super::*;

    impl<T0: WasmType> WasmTypes for (T0,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
        )}}
    }

    unsafe impl<T0: WasmType, R: WasmResult, F: Fn(T0,) -> R + 'static> HostFunc<(T0,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 1) };
            let (a0,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, R: WasmResult, F: Fn(&mut Store, T0,) -> R + 'static> HostFunc<(T0,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 1) };
            let (a0,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType> WasmTypes for (T0, T1,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, R: WasmResult, F: Fn(T0, T1,) -> R + 'static> HostFunc<(T0, T1,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 2) };
            let (a0, a1,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1,) -> R + 'static> HostFunc<(T0, T1,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 2) };
            let (a0, a1,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType> WasmTypes for (T0, T1, T2,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, R: WasmResult, F: Fn(T0, T1, T2,) -> R + 'static> HostFunc<(T0, T1, T2,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 3) };
            let (a0, a1, a2,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2,) -> R + 'static> HostFunc<(T0, T1, T2,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 3) };
            let (a0, a1, a2,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType> WasmTypes for (T0, T1, T2, T3,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE, T3::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
            dst.add(3).write(self.3.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
            T3::from_stack_value(src.add(3).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, R: WasmResult, F: Fn(T0, T1, T2, T3,) -> R + 'static> HostFunc<(T0, T1, T2, T3,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 4) };
            let (a0, a1, a2, a3,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2, a3).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2, T3,) -> R + 'static> HostFunc<(T0, T1, T2, T3,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 4) };
            let (a0, a1, a2, a3,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2, a3).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType> WasmTypes for (T0, T1, T2, T3, T4,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE, T3::WASM_TYPE, T4::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
            dst.add(3).write(self.3.to_stack_value());
            dst.add(4).write(self.4.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
            T3::from_stack_value(src.add(3).read()),
            T4::from_stack_value(src.add(4).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, R: WasmResult, F: Fn(T0, T1, T2, T3, T4,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 5) };
            let (a0, a1, a2, a3, a4,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2, a3, a4).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2, T3, T4,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 5) };
            let (a0, a1, a2, a3, a4,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2, a3, a4).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType> WasmTypes for (T0, T1, T2, T3, T4, T5,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE, T3::WASM_TYPE, T4::WASM_TYPE, T5::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
            dst.add(3).write(self.3.to_stack_value());
            dst.add(4).write(self.4.to_stack_value());
            dst.add(5).write(self.5.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
            T3::from_stack_value(src.add(3).read()),
            T4::from_stack_value(src.add(4).read()),
            T5::from_stack_value(src.add(5).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, R: WasmResult, F: Fn(T0, T1, T2, T3, T4, T5,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 6) };
            let (a0, a1, a2, a3, a4, a5,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2, a3, a4, a5).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2, T3, T4, T5,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 6) };
            let (a0, a1, a2, a3, a4, a5,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2, a3, a4, a5).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType> WasmTypes for (T0, T1, T2, T3, T4, T5, T6,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE, T3::WASM_TYPE, T4::WASM_TYPE, T5::WASM_TYPE, T6::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
            dst.add(3).write(self.3.to_stack_value());
            dst.add(4).write(self.4.to_stack_value());
            dst.add(5).write(self.5.to_stack_value());
            dst.add(6).write(self.6.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
            T3::from_stack_value(src.add(3).read()),
            T4::from_stack_value(src.add(4).read()),
            T5::from_stack_value(src.add(5).read()),
            T6::from_stack_value(src.add(6).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, R: WasmResult, F: Fn(T0, T1, T2, T3, T4, T5, T6,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 7) };
            let (a0, a1, a2, a3, a4, a5, a6,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2, a3, a4, a5, a6).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2, T3, T4, T5, T6,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 7) };
            let (a0, a1, a2, a3, a4, a5, a6,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2, a3, a4, a5, a6).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType> WasmTypes for (T0, T1, T2, T3, T4, T5, T6, T7,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE, T3::WASM_TYPE, T4::WASM_TYPE, T5::WASM_TYPE, T6::WASM_TYPE, T7::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
            dst.add(3).write(self.3.to_stack_value());
            dst.add(4).write(self.4.to_stack_value());
            dst.add(5).write(self.5.to_stack_value());
            dst.add(6).write(self.6.to_stack_value());
            dst.add(7).write(self.7.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
            T3::from_stack_value(src.add(3).read()),
            T4::from_stack_value(src.add(4).read()),
            T5::from_stack_value(src.add(5).read()),
            T6::from_stack_value(src.add(6).read()),
            T7::from_stack_value(src.add(7).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, R: WasmResult, F: Fn(T0, T1, T2, T3, T4, T5, T6, T7,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 8) };
            let (a0, a1, a2, a3, a4, a5, a6, a7,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2, a3, a4, a5, a6, a7).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2, T3, T4, T5, T6, T7,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 8) };
            let (a0, a1, a2, a3, a4, a5, a6, a7,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2, a3, a4, a5, a6, a7).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType> WasmTypes for (T0, T1, T2, T3, T4, T5, T6, T7, T8,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE, T3::WASM_TYPE, T4::WASM_TYPE, T5::WASM_TYPE, T6::WASM_TYPE, T7::WASM_TYPE, T8::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
            dst.add(3).write(self.3.to_stack_value());
            dst.add(4).write(self.4.to_stack_value());
            dst.add(5).write(self.5.to_stack_value());
            dst.add(6).write(self.6.to_stack_value());
            dst.add(7).write(self.7.to_stack_value());
            dst.add(8).write(self.8.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
            T3::from_stack_value(src.add(3).read()),
            T4::from_stack_value(src.add(4).read()),
            T5::from_stack_value(src.add(5).read()),
            T6::from_stack_value(src.add(6).read()),
            T7::from_stack_value(src.add(7).read()),
            T8::from_stack_value(src.add(8).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, R: WasmResult, F: Fn(T0, T1, T2, T3, T4, T5, T6, T7, T8,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 9) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2, a3, a4, a5, a6, a7, a8).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2, T3, T4, T5, T6, T7, T8,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 9) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2, a3, a4, a5, a6, a7, a8).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType> WasmTypes for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE, T3::WASM_TYPE, T4::WASM_TYPE, T5::WASM_TYPE, T6::WASM_TYPE, T7::WASM_TYPE, T8::WASM_TYPE, T9::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
            dst.add(3).write(self.3.to_stack_value());
            dst.add(4).write(self.4.to_stack_value());
            dst.add(5).write(self.5.to_stack_value());
            dst.add(6).write(self.6.to_stack_value());
            dst.add(7).write(self.7.to_stack_value());
            dst.add(8).write(self.8.to_stack_value());
            dst.add(9).write(self.9.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
            T3::from_stack_value(src.add(3).read()),
            T4::from_stack_value(src.add(4).read()),
            T5::from_stack_value(src.add(5).read()),
            T6::from_stack_value(src.add(6).read()),
            T7::from_stack_value(src.add(7).read()),
            T8::from_stack_value(src.add(8).read()),
            T9::from_stack_value(src.add(9).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, R: WasmResult, F: Fn(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 10) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 10) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType> WasmTypes for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE, T3::WASM_TYPE, T4::WASM_TYPE, T5::WASM_TYPE, T6::WASM_TYPE, T7::WASM_TYPE, T8::WASM_TYPE, T9::WASM_TYPE, T10::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
            dst.add(3).write(self.3.to_stack_value());
            dst.add(4).write(self.4.to_stack_value());
            dst.add(5).write(self.5.to_stack_value());
            dst.add(6).write(self.6.to_stack_value());
            dst.add(7).write(self.7.to_stack_value());
            dst.add(8).write(self.8.to_stack_value());
            dst.add(9).write(self.9.to_stack_value());
            dst.add(10).write(self.10.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
            T3::from_stack_value(src.add(3).read()),
            T4::from_stack_value(src.add(4).read()),
            T5::from_stack_value(src.add(5).read()),
            T6::from_stack_value(src.add(6).read()),
            T7::from_stack_value(src.add(7).read()),
            T8::from_stack_value(src.add(8).read()),
            T9::from_stack_value(src.add(9).read()),
            T10::from_stack_value(src.add(10).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, R: WasmResult, F: Fn(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 11) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 11) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType> WasmTypes for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE, T3::WASM_TYPE, T4::WASM_TYPE, T5::WASM_TYPE, T6::WASM_TYPE, T7::WASM_TYPE, T8::WASM_TYPE, T9::WASM_TYPE, T10::WASM_TYPE, T11::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
            dst.add(3).write(self.3.to_stack_value());
            dst.add(4).write(self.4.to_stack_value());
            dst.add(5).write(self.5.to_stack_value());
            dst.add(6).write(self.6.to_stack_value());
            dst.add(7).write(self.7.to_stack_value());
            dst.add(8).write(self.8.to_stack_value());
            dst.add(9).write(self.9.to_stack_value());
            dst.add(10).write(self.10.to_stack_value());
            dst.add(11).write(self.11.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
            T3::from_stack_value(src.add(3).read()),
            T4::from_stack_value(src.add(4).read()),
            T5::from_stack_value(src.add(5).read()),
            T6::from_stack_value(src.add(6).read()),
            T7::from_stack_value(src.add(7).read()),
            T8::from_stack_value(src.add(8).read()),
            T9::from_stack_value(src.add(9).read()),
            T10::from_stack_value(src.add(10).read()),
            T11::from_stack_value(src.add(11).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, R: WasmResult, F: Fn(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 12) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 12) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, T12: WasmType> WasmTypes for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE, T3::WASM_TYPE, T4::WASM_TYPE, T5::WASM_TYPE, T6::WASM_TYPE, T7::WASM_TYPE, T8::WASM_TYPE, T9::WASM_TYPE, T10::WASM_TYPE, T11::WASM_TYPE, T12::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
            dst.add(3).write(self.3.to_stack_value());
            dst.add(4).write(self.4.to_stack_value());
            dst.add(5).write(self.5.to_stack_value());
            dst.add(6).write(self.6.to_stack_value());
            dst.add(7).write(self.7.to_stack_value());
            dst.add(8).write(self.8.to_stack_value());
            dst.add(9).write(self.9.to_stack_value());
            dst.add(10).write(self.10.to_stack_value());
            dst.add(11).write(self.11.to_stack_value());
            dst.add(12).write(self.12.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
            T3::from_stack_value(src.add(3).read()),
            T4::from_stack_value(src.add(4).read()),
            T5::from_stack_value(src.add(5).read()),
            T6::from_stack_value(src.add(6).read()),
            T7::from_stack_value(src.add(7).read()),
            T8::from_stack_value(src.add(8).read()),
            T9::from_stack_value(src.add(9).read()),
            T10::from_stack_value(src.add(10).read()),
            T11::from_stack_value(src.add(11).read()),
            T12::from_stack_value(src.add(12).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, T12: WasmType, R: WasmResult, F: Fn(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 13) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, T12: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 13) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, T12: WasmType, T13: WasmType> WasmTypes for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE, T3::WASM_TYPE, T4::WASM_TYPE, T5::WASM_TYPE, T6::WASM_TYPE, T7::WASM_TYPE, T8::WASM_TYPE, T9::WASM_TYPE, T10::WASM_TYPE, T11::WASM_TYPE, T12::WASM_TYPE, T13::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
            dst.add(3).write(self.3.to_stack_value());
            dst.add(4).write(self.4.to_stack_value());
            dst.add(5).write(self.5.to_stack_value());
            dst.add(6).write(self.6.to_stack_value());
            dst.add(7).write(self.7.to_stack_value());
            dst.add(8).write(self.8.to_stack_value());
            dst.add(9).write(self.9.to_stack_value());
            dst.add(10).write(self.10.to_stack_value());
            dst.add(11).write(self.11.to_stack_value());
            dst.add(12).write(self.12.to_stack_value());
            dst.add(13).write(self.13.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
            T3::from_stack_value(src.add(3).read()),
            T4::from_stack_value(src.add(4).read()),
            T5::from_stack_value(src.add(5).read()),
            T6::from_stack_value(src.add(6).read()),
            T7::from_stack_value(src.add(7).read()),
            T8::from_stack_value(src.add(8).read()),
            T9::from_stack_value(src.add(9).read()),
            T10::from_stack_value(src.add(10).read()),
            T11::from_stack_value(src.add(11).read()),
            T12::from_stack_value(src.add(12).read()),
            T13::from_stack_value(src.add(13).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, T12: WasmType, T13: WasmType, R: WasmResult, F: Fn(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 14) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, T12: WasmType, T13: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 14) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, T12: WasmType, T13: WasmType, T14: WasmType> WasmTypes for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE, T3::WASM_TYPE, T4::WASM_TYPE, T5::WASM_TYPE, T6::WASM_TYPE, T7::WASM_TYPE, T8::WASM_TYPE, T9::WASM_TYPE, T10::WASM_TYPE, T11::WASM_TYPE, T12::WASM_TYPE, T13::WASM_TYPE, T14::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
            dst.add(3).write(self.3.to_stack_value());
            dst.add(4).write(self.4.to_stack_value());
            dst.add(5).write(self.5.to_stack_value());
            dst.add(6).write(self.6.to_stack_value());
            dst.add(7).write(self.7.to_stack_value());
            dst.add(8).write(self.8.to_stack_value());
            dst.add(9).write(self.9.to_stack_value());
            dst.add(10).write(self.10.to_stack_value());
            dst.add(11).write(self.11.to_stack_value());
            dst.add(12).write(self.12.to_stack_value());
            dst.add(13).write(self.13.to_stack_value());
            dst.add(14).write(self.14.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
            T3::from_stack_value(src.add(3).read()),
            T4::from_stack_value(src.add(4).read()),
            T5::from_stack_value(src.add(5).read()),
            T6::from_stack_value(src.add(6).read()),
            T7::from_stack_value(src.add(7).read()),
            T8::from_stack_value(src.add(8).read()),
            T9::from_stack_value(src.add(9).read()),
            T10::from_stack_value(src.add(10).read()),
            T11::from_stack_value(src.add(11).read()),
            T12::from_stack_value(src.add(12).read()),
            T13::from_stack_value(src.add(13).read()),
            T14::from_stack_value(src.add(14).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, T12: WasmType, T13: WasmType, T14: WasmType, R: WasmResult, F: Fn(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 15) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, T12: WasmType, T13: WasmType, T14: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 15) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }


    impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, T12: WasmType, T13: WasmType, T14: WasmType, T15: WasmType> WasmTypes for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15,) {
        const WASM_TYPES: &'static [wasm::ValueType] = &[T0::WASM_TYPE, T1::WASM_TYPE, T2::WASM_TYPE, T3::WASM_TYPE, T4::WASM_TYPE, T5::WASM_TYPE, T6::WASM_TYPE, T7::WASM_TYPE, T8::WASM_TYPE, T9::WASM_TYPE, T10::WASM_TYPE, T11::WASM_TYPE, T12::WASM_TYPE, T13::WASM_TYPE, T14::WASM_TYPE, T15::WASM_TYPE];

        #[inline(always)]
        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {
            dst.add(0).write(self.0.to_stack_value());
            dst.add(1).write(self.1.to_stack_value());
            dst.add(2).write(self.2.to_stack_value());
            dst.add(3).write(self.3.to_stack_value());
            dst.add(4).write(self.4.to_stack_value());
            dst.add(5).write(self.5.to_stack_value());
            dst.add(6).write(self.6.to_stack_value());
            dst.add(7).write(self.7.to_stack_value());
            dst.add(8).write(self.8.to_stack_value());
            dst.add(9).write(self.9.to_stack_value());
            dst.add(10).write(self.10.to_stack_value());
            dst.add(11).write(self.11.to_stack_value());
            dst.add(12).write(self.12.to_stack_value());
            dst.add(13).write(self.13.to_stack_value());
            dst.add(14).write(self.14.to_stack_value());
            dst.add(15).write(self.15.to_stack_value());
        }}

        #[inline(always)]
        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (
            T0::from_stack_value(src.add(0).read()),
            T1::from_stack_value(src.add(1).read()),
            T2::from_stack_value(src.add(2).read()),
            T3::from_stack_value(src.add(3).read()),
            T4::from_stack_value(src.add(4).read()),
            T5::from_stack_value(src.add(5).read()),
            T6::from_stack_value(src.add(6).read()),
            T7::from_stack_value(src.add(7).read()),
            T8::from_stack_value(src.add(8).read()),
            T9::from_stack_value(src.add(9).read()),
            T10::from_stack_value(src.add(10).read()),
            T11::from_stack_value(src.add(11).read()),
            T12::from_stack_value(src.add(12).read()),
            T13::from_stack_value(src.add(13).read()),
            T14::from_stack_value(src.add(14).read()),
            T15::from_stack_value(src.add(15).read()),
        )}}
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, T12: WasmType, T13: WasmType, T14: WasmType, T15: WasmType, R: WasmResult, F: Fn(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15,), R::Types, false> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 16) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }

    unsafe impl<T0: WasmType, T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType, T5: WasmType, T6: WasmType, T7: WasmType, T8: WasmType, T9: WasmType, T10: WasmType, T11: WasmType, T12: WasmType, T13: WasmType, T14: WasmType, T15: WasmType, R: WasmResult, F: Fn(&mut Store, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15,) -> R + 'static> HostFunc<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15,), R::Types, true> for F {
        #[inline]
        fn call(&self, store: &mut Store) -> Result<(), Error> {
            let stack = &mut store.thread.stack;
            unsafe { stack.set_len(stack.len() - 16) };
            let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15,) = unsafe { WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) };
            let r = (self)(store, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15).to_result()?;
            let stack = &mut store.thread.stack;
            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };
            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };
            Ok(())
        }
    }
}


