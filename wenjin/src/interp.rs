use core::hint::unreachable_unchecked;

use sti::traits::UnwrapDebug;
use sti::hash::HashMap;

use wasm::Opcode;

use crate::{Error, Table, Memory, Global, InstanceId};
use crate::store::{Store, FuncKind, StackValue, StackFrame, FuncId};


#[derive(Debug)]
struct State {
    instance: InstanceId,
    func: FuncId,

    pc: *const u8,
    code_begin: *const u8,
    code_end: *const u8,
    jumps: *const HashMap<u32, wasm::Jump>,

    bp: *mut StackValue,
    sp: *mut StackValue,
    locals_end: *mut StackValue,
    stack_frame_end: *mut StackValue,
    stack_alloc_end: *mut StackValue,

    memory_data: Option<Memory<'static>>,
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
    fn next_u8(&mut self) -> u8 {
        unsafe {
            debug_assert!(self.code_end as usize - self.pc as usize >= 1);
            let result = *self.pc;
            self.pc = self.pc.add(1);
            result
        }
    }

    #[inline]
    fn next_u32(&mut self) -> u32 {
        let at = self.next_u8();
        if at & 0x80 == 0 {
            return at as u32;
        }

        let mut result = (at & 0x7f) as u32;
        let mut shift  = 7;
        loop {
            let at = self.next_u8();
            result |= ((at & 0x7f) as u32) << shift;
            shift  += 7;
            if at & 0x80 == 0 { return result }
        }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let at = self.next_u8();
        if at & 0x80 == 0 {
            return at as u64;
        }

        let mut result = (at & 0x7f) as u64;
        let mut shift  = 7;
        loop {
            let at = self.next_u8();
            result |= ((at & 0x7f) as u64) << shift;
            shift  += 7;
            if at & 0x80 == 0 { return result }
        }
    }

    #[inline]
    fn next_i32(&mut self) -> i32 {
        self.next_i64() as i32
    }

    #[inline]
    fn next_i64(&mut self) -> i64 {
        let mut result = 0;
        let mut shift  = 0;
        loop {
            let byte = self.next_u8();
            result |= ((byte & 0x7f) as i64) << shift;
            shift  += 7;

            if byte & 0x80 == 0 {
                // sign extend.
                const SIGN: u8 = 0x40;
                if shift < 64 && (byte & SIGN) == SIGN {
                    result |= !0 << shift;
                }
                return result;
            }
        }
    }

    #[inline]
    fn next_f32(&mut self) -> f32 {
        unsafe {
            debug_assert!(self.code_end as usize - self.pc as usize >= 4);
            let result = self.pc.cast::<f32>().read_unaligned();
            self.pc = self.pc.add(4);
            result
        }
    }

    #[inline]
    fn next_f64(&mut self) -> f64 {
        unsafe {
            debug_assert!(self.code_end as usize - self.pc as usize >= 8);
            let result = self.pc.cast::<f64>().read_unaligned();
            self.pc = self.pc.add(8);
            result
        }
    }

    #[inline]
    fn jump(&mut self, from_pc: *const u8) {
        let from = (from_pc as usize - self.code_begin as usize) as u32;

        let jump = unsafe { &*self.jumps }[&from];
        let wasm::Jump { target, shift_num, shift_by } = jump;

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

        debug_assert!({
            let code_len = self.code_end as usize - self.code_begin as usize;
            (target as usize) < code_len
        });
        self.pc = unsafe { self.code_begin.add(target as usize) };
    }

    #[inline]
    fn mem_bounds_check(&self, addr: u32, offset: u32, size: u32) -> Result<(), Error> {
        // check addr+offset+N <= memory_size
        if let Some(x) = addr.checked_add(offset) {
            if let Some(end) = x.checked_add(size) {
                if end as usize <= self.memory_size {
                    return Ok(())
                }
            }
        }
        return Err(Error::TrapMemoryBounds);
    }

    #[inline]
    fn load<const N: usize>(&mut self, addr: u32, offset: u32) -> Result<[u8; N], Error> {
        self.mem_bounds_check(addr, offset, N as u32)?;
        unsafe {
            let ptr = self.memory.add((addr + offset) as usize);
            Ok(ptr.cast::<[u8; N]>().read())
        }
    }

    #[inline]
    fn load_op<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let _align = self.next_u32();
        let offset = self.next_u32();
        let addr = self.pop().as_i32() as u32;
        self.load(addr, offset)
    }

    #[must_use]
    #[inline]
    fn store<const N: usize>(&mut self, addr: u32, offset: u32, value: [u8; N]) -> Result<(), Error> {
        self.mem_bounds_check(addr, offset, N as u32)?;
        unsafe {
            let ptr = self.memory.add((addr + offset) as usize);
            ptr.cast::<[u8; N]>().write(value);
            Ok(())
        }
    }

    #[must_use]
    #[inline]
    fn store_op<const N: usize>(&mut self, value: [u8; N]) -> Result<(), Error> {
        let _align = self.next_u32();
        let offset = self.next_u32();
        let addr = self.pop().as_i32() as u32;
        self.store(addr, offset, value)
    }
}

impl Store {
    pub(crate) fn run_interp(&mut self, init_func: FuncId) -> (Result<(), Error>,) {
        assert!(!self.thread.trapped);

        let mut state = unsafe {
            let func = &*self.funcs[init_func].get();
            let FuncKind::Interp(f) = &func.kind else { unreachable!() };

            let stack = &mut self.thread.stack;
            stack.reserve_extra((f.stack_size - f.num_params) as usize);

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


            let inst = &*self.instances[f.instance].get();

            let mut memory_data = None;
            let mut memory = core::ptr::null_mut();
            let mut memory_size = 0;
            if let Some(mem) = inst.memories.inner().get(0) {
                let mut mem = Memory::new(mem);
                (memory, memory_size) = mem.as_mut_ptr();
                memory_data = Some(mem);
            }


            self.thread.frames.push(None);

            State {
                instance: f.instance,
                func: init_func,
                pc: f.code_begin(),
                code_begin: f.code_begin(),
                code_end: f.code_end(),
                jumps: &f.jumps as *const _,
                bp,
                sp,
                locals_end,
                stack_frame_end,
                stack_alloc_end,
                memory_data,
                memory,
                memory_size,
            }
        };

        let e = 'err: loop {
            macro_rules! vm_err {
                ($e:expr) => {
                    break 'err $e;
                };
            }

            macro_rules! vm_try {
                ($e:expr) => {
                    match $e {
                        Ok(r) => r,
                        Err(e) => break 'err e,
                    }
                };
            }

            let op = unsafe { match Opcode::parse(state.next_u8()) {
                wasm::opcode::ParseResult::Opcode(op) => op,

                wasm::opcode::ParseResult::Prefix(p) =>
                    Opcode::parse_prefixed(p, state.next_u32())
                        .unwrap_unchecked(),

                wasm::opcode::ParseResult::Error =>
                    unreachable_unchecked(),
            }};
            match op {
                Opcode::Unreachable => {
                    vm_err!(Error::TrapUnreachable);
                }

                Opcode::Nop => {}

                Opcode::Block => {
                    let _ty = state.next_u64();
                }

                Opcode::Loop => {
                    let _ty = state.next_u64();
                }

                Opcode::If => {
                    let _ty = state.next_u64();
                    let this = state.pc;
                    let cond = state.pop().as_i32();
                    if cond == 0 {
                        state.jump(this);
                    }
                }

                Opcode::Else => {
                    let this = state.pc;
                    state.jump(this);
                }

                Opcode::Br => {
                    let _label = state.next_u32();
                    let this = state.pc;
                    state.jump(this);
                }

                Opcode::BrIf => {
                    let _label = state.next_u32();
                    let this = state.pc;
                    let cond = state.pop().as_i32();
                    if cond != 0 {
                        state.jump(this);
                    }
                }

                Opcode::BrTable => {
                    let this = state.pc;

                    let num_labels = state.next_u32();

                    let i = state.pop().as_i32() as u32;
                    let i = if i < num_labels { i + 1 } else { 0 };

                    state.jump(unsafe { this.add(i as usize) });
                }

                Opcode::End |
                Opcode::Return => 'no_ret: { unsafe {
                    if op == Opcode::End && state.pc != state.code_end {
                        break 'no_ret;
                    }

                    let f = &*self.funcs[state.func].get();
                    let num_rets = f.ty.rets.len();

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

                        let func = &*self.funcs[frame.func].get();
                        let FuncKind::Interp(f) = &func.kind else { unreachable_unchecked() };

                        let mut memory_data = state.memory_data;
                        let mut memory = state.memory;
                        let mut memory_size = state.memory_size;
                        if frame.instance != state.instance {
                            let inst = &*self.instances[frame.instance].get();

                            memory_data = None;
                            memory = core::ptr::null_mut();
                            memory_size = 0;

                            if let Some(mem) = inst.memories.inner().get(0) {
                                let mut mem = Memory::new(mem);
                                (memory, memory_size) = mem.as_mut_ptr();
                                memory_data = Some(mem);
                            }
                        }

                        state = State {
                            instance: frame.instance,
                            func: frame.func,
                            pc: frame.pc.as_ptr(),
                            code_begin: f.code_begin(),
                            code_end: f.code_end(),
                            jumps: &f.jumps as *const _,
                            bp,
                            sp,
                            locals_end: bp.add(f.num_locals as usize),
                            stack_frame_end: bp.add(f.stack_size as usize),
                            stack_alloc_end: state.stack_alloc_end,
                            memory_data,
                            memory,
                            memory_size,
                        };
                    }
                    else {
                        let sp = state.bp.add(num_rets);
                        let stack = &mut self.thread.stack;
                        stack.set_len(sp.offset_from(stack.as_ptr()) as usize);

                        return (Ok(()),);
                    }
                }}

                Opcode::Call | Opcode::CallIndirect => {
                    let mut func = if op == Opcode::Call {
                        let func_idx = state.next_u32();

                        let inst = unsafe { &*self.instances[state.instance].get() };
                        unsafe { &*inst.funcs.inner()[func_idx as usize].get() }
                    }
                    else {
                        let type_idx = state.next_u32();
                        let tab_idx = state.next_u32();
                        let i = state.pop().as_i32() as usize;

                        let inst = unsafe { &*self.instances[state.instance].get() };
                        let tab = Table::new(&inst.tables.inner()[tab_idx as usize]);

                        let Some(id) = unsafe { tab.as_slice() }.get(i) else {
                            vm_err!(Error::TrapTableBounds);
                        };
                        let Some(func_id) = id.to_option() else {
                            vm_err!(Error::TrapCallIndirectRefNull);
                        };

                        let func = unsafe { &*self.funcs.inner()[func_id as usize].get() };

                        let expected_ty = inst.module.types[type_idx as usize];
                        if func.ty != expected_ty {
                            vm_err!(Error::TrapCallIndirectTypeMismatch);
                        }

                        func
                    };

                    while let FuncKind::Var(val) = &func.kind {
                        func = unsafe { &*val.as_ref().unwrap().get() };
                    }
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
                                stack.reserve_extra(stack_required);

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
                                pc: core::ptr::NonNull::new_unchecked(state.pc as *mut u8),
                                bp_offset,
                            };
                            self.thread.frames.push(Some(frame));

                            let mut memory_data = state.memory_data;
                            let mut memory = state.memory;
                            let mut memory_size = state.memory_size;
                            if f.instance != state.instance {
                                let inst = &*self.instances[f.instance].get();

                                memory_data = None;
                                memory = core::ptr::null_mut();
                                memory_size = 0;

                                if let Some(mem) = inst.memories.inner().get(0) {
                                    let mut mem = Memory::new(mem);
                                    (memory, memory_size) = mem.as_mut_ptr();
                                    memory_data = Some(mem);
                                }
                            }

                            state = State {
                                instance: f.instance,
                                func: func.id,
                                pc: f.code_begin(),
                                code_begin: f.code_begin(),
                                code_end: f.code_end(),
                                jumps: &f.jumps as *const _,
                                bp,
                                sp,
                                locals_end,
                                stack_frame_end,
                                stack_alloc_end,
                                memory_data,
                                memory,
                                memory_size,
                            };
                        }

                        FuncKind::Host(f) => unsafe {
                            let bp_offset = state.sp.offset_from(state.bp) as u32 - f.num_params as u32;

                            let stack = &mut self.thread.stack;

                            let bp = state.bp.offset_from(stack.as_ptr()) as usize;
                            let sp = state.sp.offset_from(stack.as_ptr()) as usize;
                            let locals_end = state.locals_end.offset_from(stack.as_ptr()) as usize;
                            let stack_frame_end = state.stack_frame_end.offset_from(stack.as_ptr()) as usize;
                            stack.set_len(sp);

                            let stack_required = sp - f.num_params as usize + f.num_rets as usize;
                            stack.reserve(stack_required);

                            let frame = StackFrame {
                                instance: state.instance,
                                func: state.func,
                                pc: core::ptr::NonNull::new_unchecked(state.pc as *mut u8),
                                bp_offset,
                            };
                            self.thread.frames.push(Some(frame));

                            if (f.call)(&*f.data as *const _ as *const u8, self).is_err() {
                                todo!()
                            }

                            self.thread.frames.pop().unwrap_debug();

                            // @cleanup: reuse `RETURN` logic?

                            let mut memory = state.memory;
                            let mut memory_size = state.memory_size;
                            if let Some(mem) = state.memory_data.as_mut() {
                                (memory, memory_size) = mem.as_mut_ptr();
                            }

                            let stack = &mut self.thread.stack;
                            let stack_ptr = stack.as_mut_ptr();
                            state = State {
                                instance: state.instance,
                                func: state.func,
                                pc: state.pc,
                                code_begin: state.code_begin,
                                code_end: state.code_end,
                                jumps: state.jumps,
                                bp: stack_ptr.add(bp),
                                sp: stack_ptr.add(stack.len()),
                                locals_end: stack_ptr.add(locals_end),
                                stack_frame_end: stack_ptr.add(stack_frame_end),
                                stack_alloc_end: stack_ptr.add(stack.cap()),
                                memory_data: state.memory_data,
                                memory,
                                memory_size,
                            };
                        }

                        FuncKind::Var(_) => unreachable!(),
                    }
                }

                Opcode::Drop => {
                    state.pop();
                }

                Opcode::Select => {
                    let cond = state.pop().as_i32();
                    let (b, a) = (state.pop(), state.pop());
                    state.push(if cond != 0 { a } else { b });
                }

                Opcode::TypedSelect => {
                    let _0 = state.next_u8();
                    let _ty = state.next_u8();
                    let cond = state.pop().as_i32();
                    let (b, a) = (state.pop(), state.pop());
                    state.push(if cond != 0 { a } else { b });
                }

                Opcode::LocalGet => {
                    let idx = state.next_u32();
                    let v = unsafe { *state.local(idx) };
                    state.push(v);
                }

                Opcode::LocalSet => {
                    let idx = state.next_u32();
                    let v = state.pop();
                    unsafe { *state.local(idx) = v };
                }

                Opcode::LocalTee => {
                    let idx = state.next_u32();
                    let v = state.top();
                    unsafe { *state.local(idx) = v };
                }

                Opcode::GlobalGet => {
                    let idx = state.next_u32();
                    let inst = unsafe { &*self.instances[state.instance].get() };
                    let global = Global::new(&inst.globals.inner()[idx as usize]);
                    state.push(StackValue::from_value(global.get()));
                }

                Opcode::GlobalSet => {
                    let idx = state.next_u32();
                    let inst = unsafe { &*self.instances[state.instance].get() };
                    let mut global = Global::new(&inst.globals.inner()[idx as usize]);
                    global.set(state.pop().to_value(global.get().ty()));
                }

                Opcode::TableGet => {
                    vm_err!(Error::Unimplemented);
                }

                Opcode::TableSet => {
                    vm_err!(Error::Unimplemented);
                }

                Opcode::I32Load => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_i32(i32::from_le_bytes(v)));
                }

                Opcode::I64Load => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_i64(i64::from_le_bytes(v)));
                }

                Opcode::F32Load => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_f32(f32::from_le_bytes(v)));
                }

                Opcode::F64Load => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_f64(f64::from_le_bytes(v)));
                }

                Opcode::I32Load8S => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_i32(i8::from_le_bytes(v) as i32));
                }

                Opcode::I32Load8U => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_i32(u8::from_le_bytes(v) as i32));
                }

                Opcode::I32Load16S => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_i32(i16::from_le_bytes(v) as i32));
                }

                Opcode::I32Load16U => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_i32(u16::from_le_bytes(v) as i32));
                }

                Opcode::I64Load8S => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_i64(i8::from_le_bytes(v) as i64));
                }

                Opcode::I64Load8U => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_i64(u8::from_le_bytes(v) as i64));
                }

                Opcode::I64Load16S => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_i64(i16::from_le_bytes(v) as i64));
                }

                Opcode::I64Load16U => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_i64(u16::from_le_bytes(v) as i64));
                }

                Opcode::I64Load32S => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_i64(i32::from_le_bytes(v) as i64));
                }

                Opcode::I64Load32U => {
                    let v = vm_try!(state.load_op());
                    state.push(StackValue::from_i64(u32::from_le_bytes(v) as i64));
                }

                Opcode::I32Store => {
                    let v = state.pop().as_i32();
                    vm_try!(state.store_op(v.to_le_bytes()));
                }

                Opcode::I64Store => {
                    let v = state.pop().as_i64();
                    vm_try!(state.store_op(v.to_le_bytes()));
                }

                Opcode::F32Store => {
                    let v = state.pop().as_f32();
                    vm_try!(state.store_op(v.to_le_bytes()));
                }

                Opcode::F64Store => {
                    let v = state.pop().as_f64();
                    vm_try!(state.store_op(v.to_le_bytes()));
                }

                Opcode::I32Store8 => {
                    let v = state.pop().as_i32() as u8;
                    vm_try!(state.store_op(v.to_le_bytes()));
                }

                Opcode::I32Store16 => {
                    let v = state.pop().as_i32() as u16;
                    vm_try!(state.store_op(v.to_le_bytes()));
                }

                Opcode::I64Store8 => {
                    let v = state.pop().as_i64() as u8;
                    vm_try!(state.store_op(v.to_le_bytes()));
                }

                Opcode::I64Store16 => {
                    let v = state.pop().as_i64() as u16;
                    vm_try!(state.store_op(v.to_le_bytes()));
                }

                Opcode::I64Store32 => {
                    let v = state.pop().as_i64() as u32;
                    vm_try!(state.store_op(v.to_le_bytes()));
                }

                Opcode::MemorySize => {
                    let mem = state.next_u32();
                    if mem != 0 { todo!() }

                    state.push(StackValue::from_i32((state.memory_size / wasm::PAGE_SIZE) as i32));
                }

                Opcode::MemoryGrow => {
                    let mem = state.next_u32();
                    if mem != 0 { todo!() }

                    let delta = state.pop().as_i32() as u32;

                    let mem = state.memory_data.as_mut().unwrap();
                    let result = match mem.grow(delta) {
                        Ok(n) => n as i32,
                        Err(_) => -1,
                    };
                    state.push(StackValue::from_i32(result));
                }

                Opcode::I32Const => {
                    let v = state.next_i32();
                    state.push(StackValue::from_i32(v));
                }

                Opcode::I64Const => {
                    let v = state.next_i64();
                    state.push(StackValue::from_i64(v));
                }

                Opcode::F32Const => {
                    let v = state.next_f32();
                    state.push(StackValue::from_f32(v));
                }

                Opcode::F64Const => {
                    let v = state.next_f64();
                    state.push(StackValue::from_f64(v));
                }

                Opcode::I32Eqz => {
                    let v = state.pop().as_i32();
                    state.push(StackValue::from_i32((v == 0) as i32));
                }

                Opcode::I32Eq => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32((a == b) as i32));
                }

                Opcode::I32Ne => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32((a != b) as i32));
                }

                Opcode::I32LtS => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32((a < b) as i32));
                }

                Opcode::I32LtU => {
                    let (b, a) = (state.pop().as_i32() as u32, state.pop().as_i32() as u32);
                    state.push(StackValue::from_i32((a < b) as i32));
                }

                Opcode::I32GtS => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32((a > b) as i32));
                }

                Opcode::I32GtU => {
                    let (b, a) = (state.pop().as_i32() as u32, state.pop().as_i32() as u32);
                    state.push(StackValue::from_i32((a > b) as i32));
                }

                Opcode::I32LeS => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32((a <= b) as i32));
                }

                Opcode::I32LeU => {
                    let (b, a) = (state.pop().as_i32() as u32, state.pop().as_i32() as u32);
                    state.push(StackValue::from_i32((a <= b) as i32));
                }

                Opcode::I32GeS => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32((a >= b) as i32));
                }

                Opcode::I32GeU => {
                    let (b, a) = (state.pop().as_i32() as u32, state.pop().as_i32() as u32);
                    state.push(StackValue::from_i32((a >= b) as i32));
                }

                Opcode::I64Eqz => {
                    let v = state.pop().as_i64();
                    state.push(StackValue::from_i32((v == 0) as i32));
                }

                Opcode::I64Eq => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i32((a == b) as i32));
                }

                Opcode::I64Ne => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i32((a != b) as i32));
                }

                Opcode::I64LtS => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i32((a < b) as i32));
                }

                Opcode::I64LtU => {
                    let (b, a) = (state.pop().as_i64() as u64, state.pop().as_i64() as u64);
                    state.push(StackValue::from_i32((a < b) as i32));
                }

                Opcode::I64GtS => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i32((a > b) as i32));
                }

                Opcode::I64GtU => {
                    let (b, a) = (state.pop().as_i64() as u64, state.pop().as_i64() as u64);
                    state.push(StackValue::from_i32((a > b) as i32));
                }

                Opcode::I64LeS => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i32((a <= b) as i32));
                }

                Opcode::I64LeU => {
                    let (b, a) = (state.pop().as_i64() as u64, state.pop().as_i64() as u64);
                    state.push(StackValue::from_i32((a <= b) as i32));
                }

                Opcode::I64GeS => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i32((a >= b) as i32));
                }

                Opcode::I64GeU => {
                    let (b, a) = (state.pop().as_i64() as u64, state.pop().as_i64() as u64);
                    state.push(StackValue::from_i32((a >= b) as i32));
                }

                Opcode::F32Eq => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_i32((a == b) as i32));
                }

                Opcode::F32Ne => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_i32((a != b) as i32));
                }

                Opcode::F32Lt => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_i32((a < b) as i32));
                }

                Opcode::F32Gt => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_i32((a > b) as i32));
                }

                Opcode::F32Le => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_i32((a <= b) as i32));
                }

                Opcode::F32Ge => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_i32((a >= b) as i32));
                }

                Opcode::F64Eq => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_i32((a == b) as i32));
                }

                Opcode::F64Ne => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_i32((a != b) as i32));
                }

                Opcode::F64Lt => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_i32((a < b) as i32));
                }

                Opcode::F64Gt => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_i32((a > b) as i32));
                }

                Opcode::F64Le => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_i32((a <= b) as i32));
                }

                Opcode::F64Ge => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_i32((a >= b) as i32));
                }

                Opcode::I32Clz => {
                    let v = state.pop().as_i32();
                    state.push(StackValue::from_i32(v.leading_zeros() as i32));
                }

                Opcode::I32Ctz => {
                    let v = state.pop().as_i32();
                    state.push(StackValue::from_i32(v.trailing_zeros() as i32));
                }

                Opcode::I32Popcnt => {
                    let v = state.pop().as_i32();
                    state.push(StackValue::from_i32(v.count_ones() as i32));
                }

                Opcode::I32Add => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a.wrapping_add(b)));
                }

                Opcode::I32Sub => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a.wrapping_sub(b)));
                }

                Opcode::I32Mul => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a.wrapping_mul(b)));
                }

                Opcode::I32DivS => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    if b == 0 {
                        vm_err!(Error::TrapDivZero);
                    }
                    state.push(StackValue::from_i32(a.wrapping_div(b)));
                }

                Opcode::I32DivU => {
                    let (b, a) = (state.pop().as_i32() as u32, state.pop().as_i32() as u32);
                    if b == 0 {
                        vm_err!(Error::TrapDivZero);
                    }
                    state.push(StackValue::from_i32(a.wrapping_div(b) as i32));
                }

                Opcode::I32RemS => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    if b == 0 {
                        vm_err!(Error::TrapDivZero);
                    }
                    state.push(StackValue::from_i32(a.wrapping_rem(b)));
                }

                Opcode::I32RemU => {
                    let (b, a) = (state.pop().as_i32() as u32, state.pop().as_i32() as u32);
                    if b == 0 {
                        vm_err!(Error::TrapDivZero);
                    }
                    state.push(StackValue::from_i32(a.wrapping_rem(b) as i32));
                }

                Opcode::I32And => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a & b));
                }

                Opcode::I32Or => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a | b));
                }

                Opcode::I32Xor => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a ^ b));
                }

                Opcode::I32Shl => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a.wrapping_shl(b as u32)));
                }

                Opcode::I32ShrS => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a.wrapping_shr(b as u32)));
                }

                Opcode::I32ShrU => {
                    let (b, a) = (state.pop().as_i32() as u32, state.pop().as_i32() as u32);
                    state.push(StackValue::from_i32(a.wrapping_shr(b as u32) as i32));
                }

                Opcode::I32Rotl => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a.rotate_left(b as u32)));
                }

                Opcode::I32Rotr => {
                    let (b, a) = (state.pop().as_i32(), state.pop().as_i32());
                    state.push(StackValue::from_i32(a.rotate_right(b as u32)));
                }

                Opcode::I64Clz => {
                    let v = state.pop().as_i64();
                    state.push(StackValue::from_i64(v.leading_zeros() as i64));
                }

                Opcode::I64Ctz => {
                    let v = state.pop().as_i64();
                    state.push(StackValue::from_i64(v.trailing_zeros() as i64));
                }

                Opcode::I64Popcnt => {
                    let v = state.pop().as_i64();
                    state.push(StackValue::from_i64(v.count_ones() as i64));
                }

                Opcode::I64Add => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a.wrapping_add(b)));
                }

                Opcode::I64Sub => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a.wrapping_sub(b)));
                }

                Opcode::I64Mul => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a.wrapping_mul(b)));
                }

                Opcode::I64DivS => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    if b == 0 {
                        vm_err!(Error::TrapDivZero);
                    }
                    state.push(StackValue::from_i64(a.wrapping_div(b)));
                }

                Opcode::I64DivU => {
                    let (b, a) = (state.pop().as_i64() as u64, state.pop().as_i64() as u64);
                    if b == 0 {
                        vm_err!(Error::TrapDivZero);
                    }
                    state.push(StackValue::from_i64(a.wrapping_div(b) as i64));
                }

                Opcode::I64RemS => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    if b == 0 {
                        vm_err!(Error::TrapDivZero);
                    }
                    state.push(StackValue::from_i64(a.wrapping_rem(b)));
                }

                Opcode::I64RemU => {
                    let (b, a) = (state.pop().as_i64() as u64, state.pop().as_i64() as u64);
                    if b == 0 {
                        vm_err!(Error::TrapDivZero);
                    }
                    state.push(StackValue::from_i64(a.wrapping_rem(b) as i64));
                }

                Opcode::I64And => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a & b));
                }

                Opcode::I64Or => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a | b));
                }

                Opcode::I64Xor => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a ^ b));
                }

                Opcode::I64Shl => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a.wrapping_shl(b as u32)));
                }

                Opcode::I64ShrS => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a.wrapping_shr(b as u32)));
                }

                Opcode::I64ShrU => {
                    let (b, a) = (state.pop().as_i64() as u64, state.pop().as_i64() as u64);
                    state.push(StackValue::from_i64(a.wrapping_shr(b as u32) as i64));
                }

                Opcode::I64Rotl => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a.rotate_left(b as u32)));
                }

                Opcode::I64Rotr => {
                    let (b, a) = (state.pop().as_i64(), state.pop().as_i64());
                    state.push(StackValue::from_i64(a.rotate_right(b as u32)));
                }

                Opcode::F32Abs => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f32(v.abs()));
                }

                Opcode::F32Neg => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f32(-v));
                }

                Opcode::F32Ceil => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f32(v.ceil()));
                }

                Opcode::F32Floor => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f32(v.floor()));
                }

                Opcode::F32Trunc => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f32(v.trunc()));
                }

                Opcode::F32Nearest => {
                    // @todo: this should be `round_ties_even`.
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f32(v.round()));
                }

                Opcode::F32Sqrt => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f32(v.sqrt()));
                }

                Opcode::F32Add => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a + b));
                }

                Opcode::F32Sub => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a - b));
                }

                Opcode::F32Mul => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a * b));
                }

                Opcode::F32Div => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a / b));
                }

                Opcode::F32Min => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a.min(b)));
                }

                Opcode::F32Max => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a.max(b)));
                }

                Opcode::F32Copysign => {
                    let (b, a) = (state.pop().as_f32(), state.pop().as_f32());
                    state.push(StackValue::from_f32(a.copysign(b)));
                }

                Opcode::F64Abs => {
                    let v = state.pop().as_f64();
                    state.push(StackValue::from_f64(v.abs()));
                }

                Opcode::F64Neg => {
                    let v = state.pop().as_f64();
                    state.push(StackValue::from_f64(-v));
                }

                Opcode::F64Ceil => {
                    let v = state.pop().as_f64();
                    state.push(StackValue::from_f64(v.ceil()));
                }

                Opcode::F64Floor => {
                    let v = state.pop().as_f64();
                    state.push(StackValue::from_f64(v.floor()));
                }

                Opcode::F64Trunc => {
                    let v = state.pop().as_f64();
                    state.push(StackValue::from_f64(v.trunc()));
                }

                Opcode::F64Nearest => {
                    // @todo: this should be `round_ties_even`.
                    let v = state.pop().as_f64();
                    state.push(StackValue::from_f64(v.round()));
                }

                Opcode::F64Sqrt => {
                    let v = state.pop().as_f64();
                    state.push(StackValue::from_f64(v.sqrt()));
                }

                Opcode::F64Add => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a + b));
                }

                Opcode::F64Sub => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a - b));
                }

                Opcode::F64Mul => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a * b));
                }

                Opcode::F64Div => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a / b));
                }

                Opcode::F64Min => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a.min(b)));
                }

                Opcode::F64Max => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a.max(b)));
                }

                Opcode::F64Copysign => {
                    let (b, a) = (state.pop().as_f64(), state.pop().as_f64());
                    state.push(StackValue::from_f64(a.copysign(b)));
                }

                Opcode::I32WrapI64 => {
                    let v = state.pop().as_i64();
                    state.push(StackValue::from_i32(v as i32));
                }

                Opcode::I32TruncF32S => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_i32(v as i32));
                }

                Opcode::I32TruncF32U => {
                    let v = state.pop().as_f32() as u32;
                    state.push(StackValue::from_i32(v as i32));
                }

                Opcode::I32TruncF64S => {
                    let v = state.pop().as_f64();
                    state.push(StackValue::from_i32(v as i32));
                }

                Opcode::I32TruncF64U => {
                    let v = state.pop().as_f64() as u32;
                    state.push(StackValue::from_i32(v as i32));
                }

                Opcode::I64ExtendI32S => {
                    let v = state.pop().as_i32();
                    state.push(StackValue::from_i64(v as i64));
                }

                Opcode::I64ExtendI32U => {
                    let v = state.pop().as_i32() as u32;
                    state.push(StackValue::from_i64(v as i64));
                }

                Opcode::I64TruncF32S => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_i64(v as i64));
                }

                Opcode::I64TruncF32U => {
                    let v = state.pop().as_f32() as u64;
                    state.push(StackValue::from_i64(v as i64));
                }

                Opcode::I64TruncF64S => {
                    let v = state.pop().as_f64();
                    state.push(StackValue::from_i64(v as i64));
                }

                Opcode::I64TruncF64U => {
                    let v = state.pop().as_f64() as u64;
                    state.push(StackValue::from_i64(v as i64));
                }

                Opcode::F32ConvertI32S => {
                    let v = state.pop().as_i32();
                    state.push(StackValue::from_f32(v as f32));
                }

                Opcode::F32ConvertI32U => {
                    let v = state.pop().as_i32() as u32;
                    state.push(StackValue::from_f32(v as f32));
                }

                Opcode::F32ConvertI64S => {
                    let v = state.pop().as_i64();
                    state.push(StackValue::from_f32(v as f32));
                }

                Opcode::F32ConvertI64U => {
                    let v = state.pop().as_i64() as u64;
                    state.push(StackValue::from_f32(v as f32));
                }

                Opcode::F32DemoteF64 => {
                    let v = state.pop().as_f64();
                    state.push(StackValue::from_f32(v as f32));
                }

                Opcode::F64ConvertI32S => {
                    let v = state.pop().as_i32();
                    state.push(StackValue::from_f64(v as f64));
                }

                Opcode::F64ConvertI32U => {
                    let v = state.pop().as_i32() as u32;
                    state.push(StackValue::from_f64(v as f64));
                }

                Opcode::F64ConvertI64S => {
                    let v = state.pop().as_i64();
                    state.push(StackValue::from_f64(v as f64));
                }

                Opcode::F64ConvertI64U => {
                    let v = state.pop().as_i64() as u64;
                    state.push(StackValue::from_f64(v as f64));
                }

                Opcode::F64PromoteF32 => {
                    let v = state.pop().as_f32();
                    state.push(StackValue::from_f64(v as f64));
                }

                Opcode::I32ReinterpretF32 => {}

                Opcode::I64ReinterpretF64 => {}

                Opcode::F32ReinterpretI32 => {}

                Opcode::F64ReinterpretI64 => {}

                Opcode::I32Extend8S => {
                    let v = state.pop().as_i32() as i8;
                    state.push(StackValue::from_i32(v as i32));
                }

                Opcode::I32Extend16S => {
                    let v = state.pop().as_i32() as i16;
                    state.push(StackValue::from_i32(v as i32));
                }

                Opcode::I64Extend8S => {
                    let v = state.pop().as_i64() as i8;
                    state.push(StackValue::from_i64(v as i64));
                }

                Opcode::I64Extend16S => {
                    let v = state.pop().as_i64() as i16;
                    state.push(StackValue::from_i64(v as i64));
                }

                Opcode::I64Extend32S => {
                    let v = state.pop().as_i64() as i32;
                    state.push(StackValue::from_i64(v as i64));
                }

                Opcode::RefNull => {
                    state.push(StackValue::from_i32(u32::MAX as i32));
                }

                Opcode::RefIsNull => {
                    let v = state.pop().as_i32() as u32;
                    state.push(StackValue::from_i32((v == u32::MAX) as i32));
                }

                Opcode::RefFunc => {
                    vm_err!(Error::Unimplemented);
                }

                Opcode::MemoryCopy => {
                    let (dst_mem, src_mem) = (state.next_u32(), state.next_u32());
                    if dst_mem != 0 || src_mem != 0 {
                        vm_err!(Error::Unimplemented);
                    }

                    let n = state.pop().as_i32() as usize;
                    let src = state.pop().as_i32() as usize;
                    let dst = state.pop().as_i32() as usize;

                    let Some(src_end) = src.checked_add(n) else {
                        vm_err!(Error::TrapMemoryBounds);
                    };
                    let Some(dst_end) = dst.checked_add(n) else {
                        vm_err!(Error::TrapMemoryBounds);
                    };
                    if src_end > state.memory_size || dst_end > state.memory_size {
                        vm_err!(Error::TrapMemoryBounds);
                    }

                    unsafe {
                        core::ptr::copy(state.memory.add(src), state.memory.add(dst), n);
                    }
                }

                Opcode::MemoryFill => {
                    let mem = state.next_u32();
                    if mem != 0 {
                        vm_err!(Error::Unimplemented);
                    }

                    let n = state.pop().as_i32() as usize;
                    let v = state.pop().as_i32() as u8;
                    let dst = state.pop().as_i32() as usize;

                    let Some(dst_end) = dst.checked_add(n) else {
                        vm_err!(Error::TrapMemoryBounds);
                    };
                    if dst_end > state.memory_size {
                        vm_err!(Error::TrapMemoryBounds);
                    }

                    unsafe {
                        core::ptr::write_bytes(state.memory.add(dst), v, n);
                    }
                }
            }
        };

        self.thread.trapped = true;
        return (Err(e),);
    }
}

