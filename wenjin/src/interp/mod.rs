mod instr;
mod bytecode;
mod compiler;
mod interp;


use sti::arena::Arena;
use sti::vec::Vec;
use sti::keyed::{Key, KVec};

use crate::wasm;
use interp::*;


pub(crate) use interp::{StackValue, run, run_dyn};


sti::define_key!(pub(crate), u32, InterpFuncId, opt: OptInterpFuncId, rng: InterpFuncIds);

pub(crate) struct InterpFunc {
    pub code:       *const bytecode::Word,
    pub stack_size: usize,
}


pub(crate) struct Interp {
    pub funcs: KVec<InterpFuncId, InterpFunc>,

    stack:  Vec<StackValue>, // @todo: don't use a vec. easy to cause UB w/ uninit vals.
    frames: Vec<Frame>,
}

impl Interp {
    pub fn new() -> Self {
        let mut frames = Vec::with_cap(1024);
        frames.push(Frame::native());

        Interp {
            funcs: KVec::new(),

            stack:  Vec::with_cap(64*1024),
            frames,
        }
    }

    pub fn compile(&mut self, module: &wasm::Module, code: &[u8], alloc: &Arena, temp: &mut Arena) -> Result<InterpFuncIds, ()> {
        use sti::reader::Reader;
        use crate::leb128;

        let mut reader = Reader::new(code);

        let num_entries = leb128::decode_u32(&mut reader).unwrap() as usize;

        assert_eq!(num_entries, module.func_types.len());

        let mut reader = reader.clone();

        let func_range = InterpFuncIds::new(
            self.funcs.next_key(),
            self.funcs.next_key().add(num_entries).unwrap(),
        );

        let mut codes = Vec::new_in(alloc);
        for i in 0..num_entries {
            let size = leb128::decode_u32(&mut reader).unwrap() as usize;
            let func = reader.next_n(size).unwrap();

            let ty_idx = module.func_types[i];

            use wasm::validator::Stack;
            let mut stack = Stack::new_in(&module, ty_idx, &*temp);

            let mut reader = Reader::new(func);

            // println!("\n\nFN {}", i + module.imports.funcs.len());

            let locals = {
                let mut locals = Vec::with_cap_in(&*temp, 128);

                let func_ty = &module.types[module.func_types[i] as usize];
                for param in func_ty.params {
                    locals.push(*param);
                }

                let num_locals = leb128::decode_u32(&mut reader).unwrap();
                for _ in 0..num_locals {
                    let count = leb128::decode_u32(&mut reader).unwrap();

                    let ty = wasm::parser::parse_value_type(&mut reader).unwrap();

                    for _ in 0..count {
                        locals.push(ty);
                    }
                }

                locals
            };


            let mut compiler = compiler::Compiler::new(&module, ty_idx, &locals, func_range, &*temp);


            let mut parser = wasm::parser::ExprParser::new(reader, code.as_ptr());

            // need a strong hint here to convince the compiler to inline this massive function.
            // massive cause `validate_operand` is also inlined.
            // the closure is called in every arm of the `match opcode` expression in `parse_expr`.
            // inlining this closure and `validate_operand` gets rid of the `match operand.data`
            // in `validate_operand` and inlines the code into the corresponding match arms.
            // this leads to roughly a 2x speedup.
            parser.parse_expr(#[cfg_attr(not(debug_assertions), inline(always))] |op, parser| {
                // println!("{:?}", op);

                let mut br_table = None;
                if op.data.is_br_table() {
                    let mut labels = Vec::new_in(&*temp);

                    let default = parser.parse_br_table(|label| {
                        labels.push(label)
                    })?;

                    br_table = Some((&*Vec::leak(labels), default));
                }

                wasm::validator::validate_operand(
                    &op, &locals, &mut stack, br_table.as_ref()).unwrap();

                compiler.add_operand(&op, br_table.as_ref());

                debug_assert_eq!(stack.stack_height(), compiler.stack_height());
                debug_assert_eq!(stack.num_frames(),   compiler.num_frames());

                return Ok(stack.done());
            }).unwrap();


            let (code, stack_size) = compiler.build(alloc);
            codes.push((&*Vec::leak(code), stack_size as usize));

            drop((stack, locals, compiler));
            temp.reset();
        }

        self.funcs.inner_mut_unck().reserve_extra(codes.len());
        for (code, stack_size) in codes.iter().copied() {
            self.funcs.push(InterpFunc { code: code.as_ptr(), stack_size });
        }
        debug_assert_eq!(self.funcs.next_key(), func_range.end());

        Ok(func_range)
    }
}

