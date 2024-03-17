mod value;
mod table;
mod memory;
mod global;
mod typed;
mod store;
mod interp;


#[derive(Clone, Copy, Debug)]
pub enum Error {
    Parse(wasm::ParseError),
    Validation(usize, wasm::ValidatorError),
    InvalidHandle,
    Unimplemented,
    CallerNotWasm,
    CallerNoMemory,
    TrapUnreachable,
    TrapTableBounds,
    TrapMemoryBounds,
    TrapCallIndirectRefNull,
    TrapCallIndirectTypeMismatch,
    TrapDivZero,
    OOM,
    Todo,
}


pub use wasm;
pub use value::Value;
pub use store::RefValue;
pub use table::Table;
pub use memory::{Memory, CType, WasmSize, WasmPtr, WasmSlice};
pub use global::Global;
pub use typed::{WasmType, WasmTypes, WasmResult};
pub use store::{Store, TypedFuncId, MemoryId};

pub use sti::num::ceil_to_multiple_pow2;


