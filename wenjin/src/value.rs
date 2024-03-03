#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Value {
    pub fn ty(self) -> wasm::ValueType {
        use Value::*;
        match self {
            I32 (_) => wasm::ValueType::I32,
            I64 (_) => wasm::ValueType::I64,
            F32 (_) => wasm::ValueType::F32,
            F64 (_) => wasm::ValueType::F64,
        }
    }
}


