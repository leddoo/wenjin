mod value;
mod table;
mod memory;
mod global;
mod typed;
mod store;
mod interp;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    OutOfMemory,
    InvalidHandle,
    Unreachable,
    Unimplemented,
    CallerNotWasm,
    CallerNoMemory,
}


pub use value::Value;
pub use store::RefValue;
pub use table::Table;
pub use memory::Memory;
pub use global::Global;
pub use typed::{WasmType, WasmTypes, WasmResult};
pub use store::{Store, TypedFuncId, MemoryId};


