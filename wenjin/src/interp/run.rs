use crate::Error;
use crate::store::{Store, InterpFunc, StackValue, ThreadData};


pub(crate) struct State {
    pc: *mut u8,
    code_end: *const u8,

    bp: *mut StackValue,
    sp: *mut StackValue,
    locals_end: *mut StackValue,
    stack_end: *mut StackValue,
}

impl State {
    pub fn new(func: &InterpFunc, bp: usize, thread: &mut ThreadData) -> Self {
        let stack = thread.stack.as_mut_ptr();
        let bp = unsafe { stack.add(bp) };
        let sp = unsafe { stack.add(thread.stack.len()) };
        let stack_end = unsafe { stack.add(func.stack_size as usize) };
        Self {
            pc: func.code,
            code_end: func.code_end,
            bp,
            sp,
            locals_end: sp,
            stack_end,
        }
    }

    #[inline]
    fn push(&mut self, value: StackValue) {
        unsafe {
            debug_assert!(self.sp < self.stack_end);
            *self.sp = value;
            self.sp = self.sp.add(1);
        }
    }

    #[inline]
    fn pop(&mut self) -> StackValue {
        unsafe {
            debug_assert!(self.sp > self.locals_end);
            self.sp = self.sp.sub(1);
            *self.sp
        }
    }

    #[inline]
    fn local(&self, idx: u32) -> *mut StackValue {
        unsafe {
            let idx = idx as usize;
            debug_assert!(idx < self.locals_end as usize - self.bp as usize);
            self.bp.add(idx)
        }
    }
}

impl Store {
    pub(crate) fn run_interp(&mut self, mut state: State) -> Result<(), Error> {
        macro_rules! next_u8 {
            () => {
                unsafe {
                    debug_assert!(state.code_end as usize - state.pc as usize > 0);
                    let result = *state.pc;
                    state.pc = state.pc.add(1);
                    result
                }
            };
        }
        macro_rules! next_u32 {
            () => {
                unsafe {
                    debug_assert!(state.code_end as usize - state.pc as usize >= 4);
                    let result = (state.pc as *const u32).read_unaligned();
                    state.pc = state.pc.add(4);
                    result
                }
            };
        }

        loop {
            let op = next_u8!();
            match op {
                wasm::opcode::UNREACHABLE => {
                    todo!()
                }

                wasm::opcode::NOP => {
                    todo!()
                }

                wasm::opcode::BLOCK => {
                    todo!()
                }

                wasm::opcode::LOOP => {
                    todo!()
                }

                wasm::opcode::IF => {
                    todo!()
                }

                wasm::opcode::ELSE => {
                    todo!()
                }

                wasm::opcode::END => {
                    todo!()
                }

                wasm::opcode::BR => {
                    todo!()
                }

                wasm::opcode::BR_IF => {
                    todo!()
                }

                wasm::opcode::BR_TABLE => {
                    todo!()
                }

                wasm::opcode::RETURN => {
                    let num_rets = next_u32!() as usize;
                    unsafe {
                        let begin = state.sp.sub(num_rets);
                        if num_rets == 1 {
                            *state.bp = *begin;
                        }
                        else if num_rets != 0 {
                            core::ptr::copy(begin, state.bp, num_rets)
                        }
                    }

                    let frame = unsafe { self.thread.frames.pop().unwrap_unchecked() };
                    if frame.is_none() {
                        return Ok(())
                    }
                    else {
                        todo!()
                    }
                }

                wasm::opcode::CALL => {
                    todo!()
                }

                wasm::opcode::CALL_INDIRECT => {
                    todo!()
                }

                wasm::opcode::DROP => {
                    todo!()
                }

                wasm::opcode::SELECT => {
                    todo!()
                }

                wasm::opcode::TYPED_SELECT => {
                    todo!()
                }

                wasm::opcode::LOCAL_GET => {
                    let idx = next_u32!();
                    let v = unsafe { *state.local(idx) };
                    state.push(v);
                }

                wasm::opcode::LOCAL_SET => {
                    todo!()
                }

                wasm::opcode::LOCAL_TEE => {
                    todo!()
                }

                wasm::opcode::GLOBAL_GET => {
                    todo!()
                }

                wasm::opcode::GLOBAL_SET => {
                    todo!()
                }

                wasm::opcode::TABLE_GET => {
                    todo!()
                }

                wasm::opcode::TABLE_SET => {
                    todo!()
                }

                wasm::opcode::I32_LOAD => {
                    todo!()
                }

                wasm::opcode::I64_LOAD => {
                    todo!()
                }

                wasm::opcode::F32_LOAD => {
                    todo!()
                }

                wasm::opcode::F64_LOAD => {
                    todo!()
                }

                wasm::opcode::I32_LOAD8_S => {
                    todo!()
                }

                wasm::opcode::I32_LOAD8_U => {
                    todo!()
                }

                wasm::opcode::I32_LOAD16_S => {
                    todo!()
                }

                wasm::opcode::I32_LOAD16_U => {
                    todo!()
                }

                wasm::opcode::I64_LOAD8_S => {
                    todo!()
                }

                wasm::opcode::I64_LOAD8_U => {
                    todo!()
                }

                wasm::opcode::I64_LOAD16_S => {
                    todo!()
                }

                wasm::opcode::I64_LOAD16_U => {
                    todo!()
                }

                wasm::opcode::I64_LOAD32_S => {
                    todo!()
                }

                wasm::opcode::I64_LOAD32_U => {
                    todo!()
                }

                wasm::opcode::I32_STORE => {
                    todo!()
                }

                wasm::opcode::I64_STORE => {
                    todo!()
                }

                wasm::opcode::F32_STORE => {
                    todo!()
                }

                wasm::opcode::F64_STORE => {
                    todo!()
                }

                wasm::opcode::I32_STORE8 => {
                    todo!()
                }

                wasm::opcode::I32_STORE16 => {
                    todo!()
                }

                wasm::opcode::I64_STORE8 => {
                    todo!()
                }

                wasm::opcode::I64_STORE16 => {
                    todo!()
                }

                wasm::opcode::I64_STORE32 => {
                    todo!()
                }

                wasm::opcode::MEMORY_SIZE => {
                    todo!()
                }

                wasm::opcode::MEMORY_GROW => {
                    todo!()
                }

                wasm::opcode::I32_CONST => {
                    todo!()
                }

                wasm::opcode::I64_CONST => {
                    todo!()
                }

                wasm::opcode::F32_CONST => {
                    todo!()
                }

                wasm::opcode::F64_CONST => {
                    todo!()
                }

                wasm::opcode::I32_EQZ => {
                    todo!()
                }

                wasm::opcode::I32_EQ => {
                    todo!()
                }

                wasm::opcode::I32_NE => {
                    todo!()
                }

                wasm::opcode::I32_LT_S => {
                    todo!()
                }

                wasm::opcode::I32_LT_U => {
                    todo!()
                }

                wasm::opcode::I32_GT_S => {
                    todo!()
                }

                wasm::opcode::I32_GT_U => {
                    todo!()
                }

                wasm::opcode::I32_LE_S => {
                    todo!()
                }

                wasm::opcode::I32_LE_U => {
                    todo!()
                }

                wasm::opcode::I32_GE_S => {
                    todo!()
                }

                wasm::opcode::I32_GE_U => {
                    todo!()
                }

                wasm::opcode::I64_EQZ => {
                    todo!()
                }

                wasm::opcode::I64_EQ => {
                    todo!()
                }

                wasm::opcode::I64_NE => {
                    todo!()
                }

                wasm::opcode::I64_LT_S => {
                    todo!()
                }

                wasm::opcode::I64_LT_U => {
                    todo!()
                }

                wasm::opcode::I64_GT_S => {
                    todo!()
                }

                wasm::opcode::I64_GT_U => {
                    todo!()
                }

                wasm::opcode::I64_LE_S => {
                    todo!()
                }

                wasm::opcode::I64_LE_U => {
                    todo!()
                }

                wasm::opcode::I64_GE_S => {
                    todo!()
                }

                wasm::opcode::I64_GE_U => {
                    todo!()
                }

                wasm::opcode::F32_EQ => {
                    todo!()
                }

                wasm::opcode::F32_NE => {
                    todo!()
                }

                wasm::opcode::F32_LT => {
                    todo!()
                }

                wasm::opcode::F32_GT => {
                    todo!()
                }

                wasm::opcode::F32_LE => {
                    todo!()
                }

                wasm::opcode::F32_GE => {
                    todo!()
                }

                wasm::opcode::F64_EQ => {
                    todo!()
                }

                wasm::opcode::F64_NE => {
                    todo!()
                }

                wasm::opcode::F64_LT => {
                    todo!()
                }

                wasm::opcode::F64_GT => {
                    todo!()
                }

                wasm::opcode::F64_LE => {
                    todo!()
                }

                wasm::opcode::F64_GE => {
                    todo!()
                }

                wasm::opcode::I32_CLZ => {
                    todo!()
                }

                wasm::opcode::I32_CTZ => {
                    todo!()
                }

                wasm::opcode::I32_POPCNT => {
                    todo!()
                }

                wasm::opcode::I32_ADD => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a.wrapping_add(b)));
                }

                wasm::opcode::I32_SUB => {
                    todo!()
                }

                wasm::opcode::I32_MUL => {
                    todo!()
                }

                wasm::opcode::I32_DIV_S => {
                    todo!()
                }

                wasm::opcode::I32_DIV_U => {
                    todo!()
                }

                wasm::opcode::I32_REM_S => {
                    todo!()
                }

                wasm::opcode::I32_REM_U => {
                    todo!()
                }

                wasm::opcode::I32_AND => {
                    todo!()
                }

                wasm::opcode::I32_OR => {
                    todo!()
                }

                wasm::opcode::I32_XOR => {
                    todo!()
                }

                wasm::opcode::I32_SHL => {
                    todo!()
                }

                wasm::opcode::I32_SHR_S => {
                    todo!()
                }

                wasm::opcode::I32_SHR_U => {
                    todo!()
                }

                wasm::opcode::I32_ROTL => {
                    todo!()
                }

                wasm::opcode::I32_ROTR => {
                    todo!()
                }

                wasm::opcode::I64_CLZ => {
                    todo!()
                }

                wasm::opcode::I64_CTZ => {
                    todo!()
                }

                wasm::opcode::I64_POPCNT => {
                    todo!()
                }

                wasm::opcode::I64_ADD => {
                    todo!()
                }

                wasm::opcode::I64_SUB => {
                    todo!()
                }

                wasm::opcode::I64_MUL => {
                    todo!()
                }

                wasm::opcode::I64_DIV_S => {
                    todo!()
                }

                wasm::opcode::I64_DIV_U => {
                    todo!()
                }

                wasm::opcode::I64_REM_S => {
                    todo!()
                }

                wasm::opcode::I64_REM_U => {
                    todo!()
                }

                wasm::opcode::I64_AND => {
                    todo!()
                }

                wasm::opcode::I64_OR => {
                    todo!()
                }

                wasm::opcode::I64_XOR => {
                    todo!()
                }

                wasm::opcode::I64_SHL => {
                    todo!()
                }

                wasm::opcode::I64_SHR_S => {
                    todo!()
                }

                wasm::opcode::I64_SHR_U => {
                    todo!()
                }

                wasm::opcode::I64_ROTL => {
                    todo!()
                }

                wasm::opcode::I64_ROTR => {
                    todo!()
                }

                wasm::opcode::F32_ABS => {
                    todo!()
                }

                wasm::opcode::F32_NEG => {
                    todo!()
                }

                wasm::opcode::F32_CEIL => {
                    todo!()
                }

                wasm::opcode::F32_FLOOR => {
                    todo!()
                }

                wasm::opcode::F32_TRUNC => {
                    todo!()
                }

                wasm::opcode::F32_NEAREST => {
                    todo!()
                }

                wasm::opcode::F32_SQRT => {
                    todo!()
                }

                wasm::opcode::F32_ADD => {
                    todo!()
                }

                wasm::opcode::F32_SUB => {
                    todo!()
                }

                wasm::opcode::F32_MUL => {
                    todo!()
                }

                wasm::opcode::F32_DIV => {
                    todo!()
                }

                wasm::opcode::F32_MIN => {
                    todo!()
                }

                wasm::opcode::F32_MAX => {
                    todo!()
                }

                wasm::opcode::F32_COPYSIGN => {
                    todo!()
                }

                wasm::opcode::F64_ABS => {
                    todo!()
                }

                wasm::opcode::F64_NEG => {
                    todo!()
                }

                wasm::opcode::F64_CEIL => {
                    todo!()
                }

                wasm::opcode::F64_FLOOR => {
                    todo!()
                }

                wasm::opcode::F64_TRUNC => {
                    todo!()
                }

                wasm::opcode::F64_NEAREST => {
                    todo!()
                }

                wasm::opcode::F64_SQRT => {
                    todo!()
                }

                wasm::opcode::F64_ADD => {
                    todo!()
                }

                wasm::opcode::F64_SUB => {
                    todo!()
                }

                wasm::opcode::F64_MUL => {
                    todo!()
                }

                wasm::opcode::F64_DIV => {
                    todo!()
                }

                wasm::opcode::F64_MIN => {
                    todo!()
                }

                wasm::opcode::F64_MAX => {
                    todo!()
                }

                wasm::opcode::F64_COPYSIGN => {
                    todo!()
                }

                wasm::opcode::I32_WRAP_I64 => {
                    todo!()
                }

                wasm::opcode::I32_TRUNC_F32_S => {
                    todo!()
                }

                wasm::opcode::I32_TRUNC_F32_U => {
                    todo!()
                }

                wasm::opcode::I32_TRUNC_F64_S => {
                    todo!()
                }

                wasm::opcode::I32_TRUNC_F64_U => {
                    todo!()
                }

                wasm::opcode::I64_EXTEND_I32_S => {
                    todo!()
                }

                wasm::opcode::I64_EXTEND_I32_U => {
                    todo!()
                }

                wasm::opcode::I64_TRUNC_F32_S => {
                    todo!()
                }

                wasm::opcode::I64_TRUNC_F32_U => {
                    todo!()
                }

                wasm::opcode::I64_TRUNC_F64_S => {
                    todo!()
                }

                wasm::opcode::I64_TRUNC_F64_U => {
                    todo!()
                }

                wasm::opcode::F32_CONVERT_I32_S => {
                    todo!()
                }

                wasm::opcode::F32_CONVERT_I32_U => {
                    todo!()
                }

                wasm::opcode::F32_CONVERT_I64_S => {
                    todo!()
                }

                wasm::opcode::F32_CONVERT_I64_U => {
                    todo!()
                }

                wasm::opcode::F32_DEMOTE_F64 => {
                    todo!()
                }

                wasm::opcode::F64_CONVERT_I32_S => {
                    todo!()
                }

                wasm::opcode::F64_CONVERT_I32_U => {
                    todo!()
                }

                wasm::opcode::F64_CONVERT_I64_S => {
                    todo!()
                }

                wasm::opcode::F64_CONVERT_I64_U => {
                    todo!()
                }

                wasm::opcode::F64_PROMOTE_F32 => {
                    todo!()
                }

                wasm::opcode::I32_REINTERPRET_F32 => {
                    todo!()
                }

                wasm::opcode::I64_REINTERPRET_F64 => {
                    todo!()
                }

                wasm::opcode::F32_REINTERPRET_I32 => {
                    todo!()
                }

                wasm::opcode::F64_REINTERPRET_I64 => {
                    todo!()
                }

                wasm::opcode::I32_EXTEND8_S => {
                    todo!()
                }

                wasm::opcode::I32_EXTEND16_S => {
                    todo!()
                }

                wasm::opcode::I64_EXTEND8_S => {
                    todo!()
                }

                wasm::opcode::I64_EXTEND16_S => {
                    todo!()
                }

                wasm::opcode::I64_EXTEND32_S => {
                    todo!()
                }

                wasm::opcode::REF_NULL => {
                    todo!()
                }

                wasm::opcode::REF_IS_NULL => {
                    todo!()
                }

                wasm::opcode::REF_FUNC => {
                    todo!()
                }

                0xfc => {
                    let op = next_u32!();
                    match op {
                        wasm::opcode::xfc::MEMORY_COPY => {
                            todo!()
                        }

                        wasm::opcode::xfc::MEMORY_FILL => {
                            todo!()
                        }

                        _ => unreachable!()
                    }
                }

                _ => unreachable!()
            }
        }
    }
}


