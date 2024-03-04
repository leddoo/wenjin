use core::cell::UnsafeCell;

use sti::boks::Box;


mod compiler;
mod run;

pub(crate) use compiler::Compiler;


pub(crate) struct InterpFunc {
    pub stack_size: u32,
    pub code: Box<UnsafeCell<[u8]>>,
}

