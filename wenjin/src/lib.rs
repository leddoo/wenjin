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
    MissingImport,
    CallerNotWasm,
    CallerNoMemory,
    TrapUnreachable,
    TrapTableBounds,
    TrapMemoryBounds,
    TrapCallIndirectRefNull,
    TrapCallIndirectTypeMismatch,
    TrapDivZero,
    OOM,
    Unimplemented,
    Todo,
}


pub use wasm;
pub use value::Value;
pub use store::RefValue;
pub use table::Table;
pub use memory::{Memory, CType, WasmSize, WasmPtr, WasmSlice};
pub use global::Global;
pub use typed::{WasmType, WasmTypes, WasmResult};
pub use store::{Store, InstanceId, FuncId, TypedFuncId, MemoryId};

pub use sti::num::ceil_to_multiple_pow2;


