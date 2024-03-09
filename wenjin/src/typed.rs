use crate::store::StackValue;


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
}


pub trait WasmResult {
    type Types: WasmTypes;

    fn to_result(self) -> Result<Self::Types, ()>;
}

impl<T: WasmTypes> WasmResult for T {
    type Types = T;

    #[inline(always)]
    fn to_result(self) -> Result<T, ()> { Ok(self) }
}

impl<T: WasmTypes> WasmResult for Result<T, ()> {
    type Types = T;

    #[inline(always)]
    fn to_result(self) -> Result<T, ()> { self }
}


