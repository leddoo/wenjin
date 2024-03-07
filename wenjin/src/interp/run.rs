use core::hint::unreachable_unchecked;

use crate::{Error, Memory};
use crate::store::{Store, FuncKind, StackValue, StackFrame};


#[derive(Debug)]
struct State {
    instance: u32,
    func: u32,

    pc: *mut u8,
    code_begin: *const u8,
    code_end: *const u8,

    bp: *mut StackValue,
    sp: *mut StackValue,
    locals_end: *mut StackValue,
    stack_frame_end: *mut StackValue,
    stack_alloc_end: *mut StackValue,

    memory: *mut u8,
    memory_size: usize,
}

impl State {
    #[inline]
    fn push(&mut self, value: StackValue) {
        unsafe {
            debug_assert!(self.sp < self.stack_frame_end);
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
    fn skip_n(&mut self, n: usize) {
        unsafe {
            debug_assert!(self.pc as usize + n <= self.code_end as usize);
            self.pc = self.pc.add(n);
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
    fn next_shift(&mut self) -> (u32, u32) {
        (self.next_u32(), self.next_u32())
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

    #[inline]
    fn jump_and_shift(&mut self, jump: (*mut u8, i32), (shift_num, shift_by): (u32, u32)) {
        self.jump(jump);
        if shift_by != 0 { unsafe {
            let src = self.sp.sub(shift_num as usize);
            let dst = src.sub(shift_by as usize);
            if shift_num == 1 {
                *dst = *src;
            }
            else {
                core::ptr::copy(src, dst, shift_num as usize);
            }
            self.sp = self.sp.sub(shift_by as usize);
        }}
    }

    #[inline]
    fn load<const N: usize>(&mut self, addr: u32, offset: u32) -> Option<[u8; N]> {
        // check addr+offset+N <= memory_size
        let end = addr.checked_add(offset)?.checked_add(N as u32)?;
        if end as usize > self.memory_size {
            return None;
        }

        unsafe {
            let ptr = self.memory.add((addr + offset) as usize);
            Some(ptr.cast::<[u8; N]>().read())
        }
    }

    #[inline]
    fn load_op<const N: usize>(&mut self) -> Option<[u8; N]> {
        let offset = self.next_u32();
        let addr = self.pop().as_i32() as u32;
        self.load(addr, offset)
    }

    #[must_use]
    #[inline]
    fn store<const N: usize>(&mut self, addr: u32, offset: u32, value: [u8; N]) -> Option<()> {
        // check addr+offset+N <= memory_size
        let end = addr.checked_add(offset)?.checked_add(N as u32)?;
        if end as usize > self.memory_size {
            return None;
        }

        unsafe {
            let ptr = self.memory.add((addr + offset) as usize);
            ptr.cast::<[u8; N]>().write(value);
            Some(())
        }
    }

    #[must_use]
    #[inline]
    fn store_op<const N: usize>(&mut self, value: [u8; N]) -> Option<()> {
        let offset = self.next_u32();
        let addr = self.pop().as_i32() as u32;
        self.store(addr, offset, value)
    }
}

impl Store {
    pub(crate) fn run_interp(&mut self, init_func: u32) -> Result<(), Error> {
        let mut state = unsafe {
            let func = &self.funcs[init_func as usize];
            let FuncKind::Interp(f) = &func.kind else { unreachable!() };

            let stack = &mut self.thread.stack;
            if stack.reserve_extra((f.stack_size - f.num_params) as usize).is_err() {
                todo!()
            }

            let stack_ptr = stack.as_mut_ptr();

            let sp = stack_ptr.add(stack.len());
            let bp = sp.sub(f.num_params as usize);
            let locals_end = bp.add(f.num_locals as usize);
            let stack_frame_end = bp.add(f.stack_size as usize);
            let stack_alloc_end = stack_ptr.add(stack.cap());

            // init locals.
            for i in 0..(f.num_locals - f.num_params) as usize {
                *sp.add(i) = StackValue::ZERO;
            }
            let sp = locals_end;


            let inst = &self.instances[f.instance as usize];

            let mut memory = core::ptr::null_mut();
            let mut memory_size = 0;
            if let Some(mem_id) = inst.memories.get(0).copied() {
                let mut mem = Memory::new(&self.memories[mem_id as usize]);
                (memory, memory_size) = mem.as_mut_ptr();
            }


            self.thread.frames.push_or_alloc(None).map_err(|_| Error::OutOfMemory)?;

            State {
                instance: f.instance,
                func: init_func,
                pc: f.code,
                code_begin: f.code,
                code_end: f.code_end,
                bp,
                sp,
                locals_end,
                stack_frame_end,
                stack_alloc_end,
                memory,
                memory_size,
            }
        };

        loop {
            let op = state.next_u8();
            match op {
                wasm::opcode::UNREACHABLE => {
                    return Err(Error::Unreachable);
                }

                wasm::opcode::NOP => {}

                wasm::opcode::BLOCK => {}

                wasm::opcode::LOOP => {}

                wasm::opcode::IF => {
                    let els = state.next_jump();
                    let cond = state.pop().as_i32();
                    if cond == 0 {
                        state.jump(els);
                    }
                }

                wasm::opcode::ELSE => {}

                wasm::opcode::END => {}

                wasm::opcode::BR => {
                    let dst = state.next_jump();
                    let shift = state.next_shift();
                    state.jump_and_shift(dst, shift);
                }

                wasm::opcode::BR_IF => {
                    let dst = state.next_jump();
                    let shift = state.next_shift();
                    let cond = state.pop().as_i32();
                    if cond != 0 {
                        state.jump_and_shift(dst, shift);
                    }
                }

                wasm::opcode::BR_TABLE => {
                    let i = state.pop().as_i32() as usize;
                    let num_labels = state.next_u32() as usize;
                    if i < num_labels {
                        state.skip_n(i*12);
                    }
                    else {
                        state.skip_n(num_labels*12);
                    }
                    let dst = state.next_jump();
                    let shift = state.next_shift();
                    state.jump_and_shift(dst, shift);
                }

                wasm::opcode::RETURN => unsafe {
                    let num_rets = state.next_u32() as usize;
                    let rets = state.sp.sub(num_rets);
                    if num_rets == 1 {
                        *state.bp = *rets;
                    }
                    else if num_rets != 0 {
                        core::ptr::copy(rets, state.bp, num_rets)
                    }

                    let frame = self.thread.frames.pop().unwrap_unchecked();
                    if let Some(frame) = frame {
                        let bp = state.bp.sub(frame.bp_offset as usize);
                        let sp = state.bp.add(num_rets);

                        let func = &self.funcs[frame.func as usize];
                        let FuncKind::Interp(f) = &func.kind else { unreachable_unchecked() };

                        let mut memory = state.memory;
                        let mut memory_size = state.memory_size;
                        if frame.instance != state.instance {
                            let inst = &self.instances[frame.instance as usize];

                            memory = core::ptr::null_mut();
                            memory_size = 0;

                            if let Some(mem_id) = inst.memories.get(0).copied() {
                                let mut mem = Memory::new(&self.memories[mem_id as usize]);
                                (memory, memory_size) = mem.as_mut_ptr();
                            }
                        }

                        state = State {
                            instance: frame.instance,
                            func: frame.func,
                            pc: frame.pc.as_ptr(),
                            code_begin: f.code,
                            code_end: f.code_end,
                            bp,
                            sp,
                            locals_end: bp.add(f.num_locals as usize),
                            stack_frame_end: bp.add(f.stack_size as usize),
                            stack_alloc_end: state.stack_alloc_end,
                            memory,
                            memory_size,
                        };
                    }
                    else {
                        let sp = state.bp.add(num_rets);
                        let stack = &mut self.thread.stack;
                        stack.set_len(sp.offset_from(stack.as_ptr()) as usize);

                        return Ok(())
                    }
                }

                wasm::opcode::CALL => {
                    let func_idx = state.next_u32();

                    let inst = &self.instances[state.instance as usize];
                    let func_idx = inst.funcs[func_idx as usize];

                    let func = &self.funcs[func_idx as usize];
                    match &func.kind {
                        FuncKind::Interp(f) => unsafe {
                            let bp_offset = state.sp.offset_from(state.bp) as u32 - f.num_params;

                            // grow stack.
                            let (sp, stack_alloc_end);
                            let stack_remaining = state.stack_alloc_end.offset_from(state.sp) as usize;
                            let stack_required = (f.stack_size - f.num_params) as usize;
                            if stack_remaining >= stack_required {
                                sp = state.sp;
                                stack_alloc_end = state.stack_alloc_end;
                            }
                            else {
                                let stack = &mut self.thread.stack;

                                let stack_len = state.sp.offset_from(stack.as_ptr()) as usize;
                                stack.set_len(stack_len);
                                if stack.reserve_extra(stack_required).is_err() {
                                    todo!()
                                }

                                let stack_ptr = stack.as_mut_ptr();
                                sp = stack_ptr.add(stack_len);
                                stack_alloc_end = stack_ptr.add(stack.cap());
                            }

                            let bp = sp.sub(f.num_params as usize);
                            let locals_end = bp.add(f.num_locals as usize);
                            let stack_frame_end = bp.add(f.stack_size as usize);

                            // init locals.
                            for i in 0..(f.num_locals - f.num_params) as usize {
                                *sp.add(i) = StackValue::ZERO;
                            }
                            let sp = locals_end;

                            let frame = StackFrame {
                                instance: state.instance,
                                func: state.func,
                                pc: core::ptr::NonNull::new_unchecked(state.pc),
                                bp_offset,
                            };
                            if self.thread.frames.push_or_alloc(Some(frame)).is_err() {
                                todo!()
                            }

                            let mut memory = state.memory;
                            let mut memory_size = state.memory_size;
                            if f.instance != state.instance {
                                let inst = &self.instances[f.instance as usize];

                                memory = core::ptr::null_mut();
                                memory_size = 0;

                                if let Some(mem_id) = inst.memories.get(0).copied() {
                                    let mut mem = Memory::new(&self.memories[mem_id as usize]);
                                    (memory, memory_size) = mem.as_mut_ptr();
                                }
                            }

                            state = State {
                                instance: f.instance,
                                func: func_idx,
                                pc: f.code,
                                code_begin: f.code,
                                code_end: f.code_end,
                                bp,
                                sp,
                                locals_end,
                                stack_frame_end,
                                stack_alloc_end,
                                memory,
                                memory_size,
                            };
                        }

                        FuncKind::Temp => unreachable!(),
                    }
                }

                wasm::opcode::CALL_INDIRECT => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::DROP => {
                    state.pop();
                }

                wasm::opcode::SELECT => {
                    let cond = state.pop().as_i32();
                    let (b, a) = (state.pop(), state.pop());
                    state.push(if cond != 0 { a } else { b });
                }

                wasm::opcode::TYPED_SELECT => unreachable!(),

                wasm::opcode::LOCAL_GET => {
                    let idx = state.next_u32();
                    let v = unsafe { *state.local(idx) };
                    state.push(v);
                }

                wasm::opcode::LOCAL_SET => {
                    let idx = state.next_u32();
                    let v = state.pop();
                    unsafe { *state.local(idx) = v };
                }

                wasm::opcode::LOCAL_TEE => {
                    let idx = state.next_u32();
                    let v = state.top();
                    unsafe { *state.local(idx) = v };
                }

                wasm::opcode::GLOBAL_GET => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::GLOBAL_SET => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::TABLE_GET => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::TABLE_SET => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I32_LOAD => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_i32(i32::from_ne_bytes(v)));
                }

                wasm::opcode::I64_LOAD => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_i64(i64::from_ne_bytes(v)));
                }

                wasm::opcode::F32_LOAD => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_f32(f32::from_ne_bytes(v)));
                }

                wasm::opcode::F64_LOAD => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_f64(f64::from_ne_bytes(v)));
                }

                wasm::opcode::I32_LOAD8_S => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_i32(i8::from_ne_bytes(v) as i32));
                }

                wasm::opcode::I32_LOAD8_U => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_i32(u8::from_ne_bytes(v) as i32));
                }

                wasm::opcode::I32_LOAD16_S => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_i32(i16::from_ne_bytes(v) as i32));
                }

                wasm::opcode::I32_LOAD16_U => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_i32(u16::from_ne_bytes(v) as i32));
                }

                wasm::opcode::I64_LOAD8_S => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_i64(i8::from_ne_bytes(v) as i64));
                }

                wasm::opcode::I64_LOAD8_U => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_i64(u8::from_ne_bytes(v) as i64));
                }

                wasm::opcode::I64_LOAD16_S => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_i64(i16::from_ne_bytes(v) as i64));
                }

                wasm::opcode::I64_LOAD16_U => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_i64(u16::from_ne_bytes(v) as i64));
                }

                wasm::opcode::I64_LOAD32_S => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_i64(i32::from_ne_bytes(v) as i64));
                }

                wasm::opcode::I64_LOAD32_U => {
                    let Some(v) = state.load_op() else { todo!() };
                    state.push(StackValue::from_i64(u32::from_ne_bytes(v) as i64));
                }

                wasm::opcode::I32_STORE => {
                    let v = state.pop().as_i32();
                    let Some(()) = state.store_op(v.to_ne_bytes()) else { todo!() };
                }

                wasm::opcode::I64_STORE => {
                    let v = state.pop().as_i64();
                    let Some(()) = state.store_op(v.to_ne_bytes()) else { todo!() };
                }

                wasm::opcode::F32_STORE => {
                    let v = state.pop().as_f32();
                    let Some(()) = state.store_op(v.to_ne_bytes()) else { todo!() };
                }

                wasm::opcode::F64_STORE => {
                    let v = state.pop().as_f64();
                    let Some(()) = state.store_op(v.to_ne_bytes()) else { todo!() };
                }

                wasm::opcode::I32_STORE8 => {
                    let v = state.pop().as_i32() as u8;
                    let Some(()) = state.store_op(v.to_ne_bytes()) else { todo!() };
                }

                wasm::opcode::I32_STORE16 => {
                    let v = state.pop().as_i32() as u16;
                    let Some(()) = state.store_op(v.to_ne_bytes()) else { todo!() };
                }

                wasm::opcode::I64_STORE8 => {
                    let v = state.pop().as_i64() as u8;
                    let Some(()) = state.store_op(v.to_ne_bytes()) else { todo!() };
                }

                wasm::opcode::I64_STORE16 => {
                    let v = state.pop().as_i64() as u16;
                    let Some(()) = state.store_op(v.to_ne_bytes()) else { todo!() };
                }

                wasm::opcode::I64_STORE32 => {
                    let v = state.pop().as_i64() as u32;
                    let Some(()) = state.store_op(v.to_ne_bytes()) else { todo!() };
                }

                wasm::opcode::MEMORY_SIZE => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::MEMORY_GROW => {
                    return Err(Error::Unimplemented);
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
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_EQ => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_NE => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_LT_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_LT_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_GT_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_GT_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_LE_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_LE_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_GE_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_GE_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F32_EQ => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_i32((a == b) as i32));
                }

                wasm::opcode::F32_NE => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_i32((a != b) as i32));
                }

                wasm::opcode::F32_LT => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_i32((a < b) as i32));
                }

                wasm::opcode::F32_GT => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_i32((a > b) as i32));
                }

                wasm::opcode::F32_LE => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_i32((a <= b) as i32));
                }

                wasm::opcode::F32_GE => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_i32((a >= b) as i32));
                }

                wasm::opcode::F64_EQ => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_i32((a == b) as i32));
                }

                wasm::opcode::F64_NE => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_i32((a != b) as i32));
                }

                wasm::opcode::F64_LT => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_i32((a < b) as i32));
                }

                wasm::opcode::F64_GT => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_i32((a > b) as i32));
                }

                wasm::opcode::F64_LE => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_i32((a <= b) as i32));
                }

                wasm::opcode::F64_GE => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_i32((a >= b) as i32));
                }

                wasm::opcode::I32_CLZ => {
                    let v = state.pop().as_i32();
                    state.push(StackValue::from_i32(v.leading_zeros() as i32));
                }

                wasm::opcode::I32_CTZ => {
                    let v = state.pop().as_i32();
                    state.push(StackValue::from_i32(v.trailing_zeros() as i32));
                }

                wasm::opcode::I32_POPCNT => {
                    let v = state.pop().as_i32();
                    state.push(StackValue::from_i32(v.count_ones() as i32));
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
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I32_SHR_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I32_SHR_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I32_ROTL => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I32_ROTR => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_CLZ => {
                    let v = state.pop().as_i64();
                    state.push(StackValue::from_i64(v.leading_zeros() as i64));
                }

                wasm::opcode::I64_CTZ => {
                    let v = state.pop().as_i64();
                    state.push(StackValue::from_i64(v.trailing_zeros() as i64));
                }

                wasm::opcode::I64_POPCNT => {
                    let v = state.pop().as_i64();
                    state.push(StackValue::from_i64(v.count_ones() as i64));
                }

                wasm::opcode::I64_ADD => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a.wrapping_add(b)));
                }

                wasm::opcode::I64_SUB => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a.wrapping_sub(b)));
                }

                wasm::opcode::I64_MUL => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a.wrapping_mul(b)));
                }

                wasm::opcode::I64_DIV_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_DIV_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_REM_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_REM_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_AND => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a & b));
                }

                wasm::opcode::I64_OR => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a | b));
                }

                wasm::opcode::I64_XOR => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a ^ b));
                }

                wasm::opcode::I64_SHL => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_SHR_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_SHR_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_ROTL => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_ROTR => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F32_ABS => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f32(v.abs()));
                }

                wasm::opcode::F32_NEG => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f32(-v));
                }

                wasm::opcode::F32_CEIL => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f32(v.ceil()));
                }

                wasm::opcode::F32_FLOOR => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f32(v.floor()));
                }

                wasm::opcode::F32_TRUNC => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f32(v.trunc()));
                }

                wasm::opcode::F32_NEAREST => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F32_SQRT => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f32(v.sqrt()));
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
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a.min(b)));
                }

                wasm::opcode::F32_MAX => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a.max(b)));
                }

                wasm::opcode::F32_COPYSIGN => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a.copysign(b)));
                }

                wasm::opcode::F64_ABS => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F64_NEG => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F64_CEIL => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F64_FLOOR => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F64_TRUNC => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F64_NEAREST => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F64_SQRT => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F64_ADD => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a + b));
                }

                wasm::opcode::F64_SUB => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a - b));
                }

                wasm::opcode::F64_MUL => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a * b));
                }

                wasm::opcode::F64_DIV => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a / b));
                }

                wasm::opcode::F64_MIN => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a.min(b)));
                }

                wasm::opcode::F64_MAX => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a.max(b)));
                }

                wasm::opcode::F64_COPYSIGN => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a.copysign(b)));
                }

                wasm::opcode::I32_WRAP_I64 => {
                    let v = state.pop().as_i64();
                    state.push(StackValue::from_i32(v as i32));
                }

                wasm::opcode::I32_TRUNC_F32_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I32_TRUNC_F32_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I32_TRUNC_F64_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I32_TRUNC_F64_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_EXTEND_I32_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_EXTEND_I32_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_TRUNC_F32_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_TRUNC_F32_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_TRUNC_F64_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_TRUNC_F64_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F32_CONVERT_I32_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F32_CONVERT_I32_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F32_CONVERT_I64_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F32_CONVERT_I64_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F32_DEMOTE_F64 => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F64_CONVERT_I32_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F64_CONVERT_I32_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F64_CONVERT_I64_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F64_CONVERT_I64_U => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F64_PROMOTE_F32 => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I32_REINTERPRET_F32 => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_REINTERPRET_F64 => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F32_REINTERPRET_I32 => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::F64_REINTERPRET_I64 => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I32_EXTEND8_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I32_EXTEND16_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_EXTEND8_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_EXTEND16_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::I64_EXTEND32_S => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::REF_NULL => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::REF_IS_NULL => {
                    return Err(Error::Unimplemented);
                }

                wasm::opcode::REF_FUNC => {
                    return Err(Error::Unimplemented);
                }

                0xfc => {
                    let op = state.next_u32();
                    match op {
                        wasm::opcode::xfc::MEMORY_COPY => {
                            return Err(Error::Unimplemented);
                        }

                        wasm::opcode::xfc::MEMORY_FILL => {
                            return Err(Error::Unimplemented);
                        }

                        _ => unreachable!()
                    }
                }

                _ => unreachable!()
            }
        }
    }
}


