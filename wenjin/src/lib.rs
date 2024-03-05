mod value;
mod memory;
mod global;
mod store;
mod interp;


#[derive(Clone, Copy, Debug)]
pub enum Error {
    OutOfMemory,
    InvalidHandle,
    Unreachable,
    Unimplemented,
}


pub use value::Value;
pub use memory::Memory;
//pub use global::Global;
pub use store::{MemoryId, Store};

