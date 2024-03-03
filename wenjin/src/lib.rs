mod memory;
mod store;


#[derive(Clone, Copy, Debug)]
pub enum Error {
    OutOfMemory,
    InvalidHandle,
}


pub use memory::MemoryCtx;
pub use store::{MemoryId, Store};

