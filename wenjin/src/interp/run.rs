use crate::Error;
use crate::store::{Store, InterpFunc, StackValue, ThreadData};


pub(crate) struct State {
    pc: *mut u8,
    code_begin: *const u8,
    code_end: *const u8,

    bp: *mut StackValue,
    sp: *mut StackValue,
    locals_end: *mut StackValue,
    stack_end: *mut StackValue,
}

impl State {
    pub fn new(func: &InterpFunc, bp: usize, thread: &mut ThreadData) -> Self {
        dbg!(func);

        let stack = thread.stack.as_mut_ptr();
        let bp = unsafe { stack.add(bp) };
        let sp = unsafe { stack.add(thread.stack.len()) };
        let stack_end = unsafe { bp.add(func.stack_size as usize) };
        Self {
            pc: func.code,
            code_begin: func.code,
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
    fn top(&mut self) -> StackValue {
        unsafe {
            debug_assert!(self.sp > self.locals_end);
            *self.sp.sub(1)
        }
    }

    #[inline]
    fn local(&self, idx: u32) -> *mut StackValue {
        unsafe {
            debug_assert!((idx as isize) < self.locals_end.offset_from(self.bp));
            self.bp.add(idx as usize)
        }
    }


    #[inline]
    fn next_u8(&mut self) -> u8 {
        unsafe {
            debug_assert!(self.code_end as usize - self.pc as usize > 0);
            let result = *self.pc;
            self.pc = self.pc.add(1);
            result
        }
    }

    #[inline]
    fn next_u32(&mut self) -> u32 {
        unsafe {
            debug_assert!(self.code_end as usize - self.pc as usize >= 4);
            let result = (self.pc as *const u32).read_unaligned();
            self.pc = self.pc.add(4);
            result
        }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        unsafe {
            debug_assert!(self.code_end as usize - self.pc as usize >= 8);
            let result = (self.pc as *const u64).read_unaligned();
            self.pc = self.pc.add(8);
            result
        }
    }

    #[inline]
    fn next_jump(&mut self) -> (*mut u8, i32) {
        (self.pc, self.next_u32() as i32)
    }

    #[inline]
    fn jump(&mut self, (from, delta): (*mut u8, i32)) {
        unsafe {
            let delta = delta as isize;
            debug_assert!({
                let dst = (from as isize + delta) as usize;
                dst >= self.code_begin as usize && dst < self.code_end as usize
            });

            self.pc = from.offset(delta);
        }
    }
}

impl Store {
    pub(crate) fn run_interp(&mut self, mut state: State) -> Result<(), Error> {
        loop {
            let op = state.next_u8();
            println!("{:x}: {}", state.pc as usize - state.code_begin as usize, wasm::opcode::name(op));
            println!("{}", unsafe { state.sp.offset_from(state.locals_end) });
            match op {
                wasm::opcode::UNREACHABLE => {
                    todo!()
                }

                wasm::opcode::NOP => {}

                wasm::opcode::BLOCK => {}

                wasm::opcode::LOOP => {}

                wasm::opcode::IF => {
                    todo!()
                }

                wasm::opcode::ELSE => {
                    todo!()
                }

                wasm::opcode::END => {}

                wasm::opcode::BR => {
                    let delta = state.next_jump();
                    state.jump(delta);
                }

                wasm::opcode::BR_IF => {
                    let delta = state.next_jump();
                    let cond = state.pop().as_i32();
                    if dbg!(cond) != 0 {
                        println!("jump by {}", delta.1);
                        state.jump(delta);
                    }
                }

                wasm::opcode::BR_TABLE => {
                    todo!()
                }

                wasm::opcode::RETURN => {
                    let num_rets = state.next_u32() as usize;
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
                    let cond = state.pop().as_i32();
                    let (b, a) = (state.pop(), state.pop());
                    state.push(if cond != 0 { a } else { b });
                }

                wasm::opcode::TYPED_SELECT => {
                    todo!()
                }

                wasm::opcode::LOCAL_GET => {
                    let idx = state.next_u32();
                    let v = unsafe { *state.local(idx) };
                    println!("{} {}", v.as_i32(), v.as_f32());
                    state.push(v);
                }

                wasm::opcode::LOCAL_SET => {
                    let idx = state.next_u32();
                    let v = state.pop();
                    println!("{} {}", v.as_i32(), v.as_f32());
                    unsafe { *state.local(idx) = v };
                }

                wasm::opcode::LOCAL_TEE => {
                    let idx = state.next_u32();
                    let v = state.top();
                    println!("{} {}", v.as_i32(), v.as_f32());
                    unsafe { *state.local(idx) = v };
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
                    let v = state.next_u32() as i32;
                    state.push(StackValue::from_i32(v));
                }

                wasm::opcode::I64_CONST => {
                    let v = state.next_u64() as i64;
                    state.push(StackValue::from_i64(v));
                }

                wasm::opcode::F32_CONST => {
                    let v = f32::from_bits(state.next_u32());
                    state.push(StackValue::from_f32(v));
                }

                wasm::opcode::F64_CONST => {
                    let v = f64::from_bits(state.next_u64());
                    state.push(StackValue::from_f64(v));
                }

                wasm::opcode::I32_EQZ => {
                    let v = state.pop().as_i32();
                    state.push(StackValue::from_i32((v == 0) as i32));
                }

                wasm::opcode::I32_EQ => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32((a == b) as i32));
                }

                wasm::opcode::I32_NE => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32((a != b) as i32));
                }

                wasm::opcode::I32_LT_S => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32((a < b) as i32));
                }

                wasm::opcode::I32_LT_U => {
                    let (b, a) = (state.pop().as_i32() as u32, state.pop().as_i32() as u32);
                    state.push(StackValue::from_i32((a < b) as i32));
                }

                wasm::opcode::I32_GT_S => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32((a > b) as i32));
                }

                wasm::opcode::I32_GT_U => {
                    let (b, a) = (state.pop().as_i32() as u32, state.pop().as_i32() as u32);
                    state.push(StackValue::from_i32((a > b) as i32));
                }

                wasm::opcode::I32_LE_S => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32((a <= b) as i32));
                }

                wasm::opcode::I32_LE_U => {
                    let (b, a) = (state.pop().as_i32() as u32, state.pop().as_i32() as u32);
                    state.push(StackValue::from_i32((a <= b) as i32));
                }

                wasm::opcode::I32_GE_S => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32((a >= b) as i32));
                }

                wasm::opcode::I32_GE_U => {
                    let (b, a) = (state.pop().as_i32() as u32, state.pop().as_i32() as u32);
                    state.push(StackValue::from_i32((a >= b) as i32));
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
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a.wrapping_sub(b)));
                }

                wasm::opcode::I32_MUL => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a.wrapping_mul(b)));
                }

                wasm::opcode::I32_DIV_S => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    if b == 0 {
                        todo!()
                    }
                    state.push(StackValue::from_i32(a.wrapping_div(b)));
                }

                wasm::opcode::I32_DIV_U => {
                    let (b, a) = (state.pop().as_i32() as u32, state.pop().as_i32() as u32);
                    if b == 0 {
                        todo!()
                    }
                    state.push(StackValue::from_i32(a.wrapping_div(b) as i32));
                }

                wasm::opcode::I32_REM_S => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    if b == 0 {
                        todo!()
                    }
                    state.push(StackValue::from_i32(a.wrapping_rem(b)));
                }

                wasm::opcode::I32_REM_U => {
                    let (b, a) = (state.pop().as_i32() as u32, state.pop().as_i32() as u32);
                    if b == 0 {
                        todo!()
                    }
                    state.push(StackValue::from_i32(a.wrapping_rem(b) as i32));
                }

                wasm::opcode::I32_AND => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a & b));
                }

                wasm::opcode::I32_OR => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a | b));
                }

                wasm::opcode::I32_XOR => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a ^ b));
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
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a + b));
                }

                wasm::opcode::F32_SUB => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a - b));
                }

                wasm::opcode::F32_MUL => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a * b));
                }

                wasm::opcode::F32_DIV => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a / b));
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
                    let op = state.next_u32();
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


