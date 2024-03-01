use crate::ParaPtr;
use crate::value::Value;
use crate::wasm::*;
use crate::store::*;

use crate::interp::stack::{StackPtr, StackValue, Frame};

use super::bytecode::{Op, Word};
use super::*;


pub(crate) fn run<P: WasmTypes, R: WasmTypes>
    (store: &mut Store,
     inst: InstanceId,
     func: InterpFuncId,
     args: P)
    -> Result<R, ()>
{
    let this = &mut store.interp;

    let InterpFunc { code, stack_size } = this.funcs[func];
    unsafe { check_stack(this, stack_size as usize) }

    args.to_stack_values(this.stack.as_mut_ptr());

    this.frames.push(Frame {
        instance: inst.some(),
        func: func.some(),
        pc: code,
        bp: 0,
    });
    interp(store).0?;

    Ok(R::from_stack_values(store.interp.stack.as_ptr()))
}

pub(crate) fn run_dyn
    (store: &mut Store,
     inst: InstanceId,
     func: InterpFuncId,
     ty: &wasm::FuncType,
     args: &[Value],
     rets: &mut [Value])
    -> Result<(), ()>
{
    let this = &mut store.interp;

    debug_assert_eq!(args.len(), ty.params.len());
    debug_assert_eq!(rets.len(), ty.rets.len());
    for (i, arg) in args.iter().enumerate() {
        debug_assert_eq!(arg.ty(), ty.params[i]);
    }

    let InterpFunc { code, stack_size } = this.funcs[func];
    unsafe { check_stack(this, stack_size as usize) }

    // copy in args.
    for (i, arg) in args.iter().enumerate() {
        let slot = unsafe { StackPtr::new(&mut this.stack, i) };
        slot.write(arg.to_stack_value());
    }

    this.frames.push(Frame {
        instance: inst.some(),
        func: func.some(),
        pc: code,
        bp: 0,
    });
    interp(store).0?;

    // copy out rets.
    for (i, ret) in rets.iter_mut().enumerate() {
        let value = unsafe { StackPtr::new(&mut store.interp.stack, i) };

        *ret = match ty.rets[i] {
            ValueType::I32 => { Value::I32(value.i32()) }
            ValueType::I64 => { Value::I64(value.i64()) }
            ValueType::F32 => { Value::F32(value.f32()) }
            ValueType::F64 => { Value::F64(value.f64()) }

            _ => unimplemented!()
        }
    }

    return Ok(());
}






struct InterpState {
    instance: OptInstanceId,
    func:     OptInterpFuncId,

    memory: MemoryView<'static>,

    memories: ParaPtr<MemoryId>,
    funcs:    ParaPtr<FuncId>,
    globals:  ParaPtr<GlobalId>,
    tables:   ParaPtr<TableId>,

    pc: *const Word,
    bp: *mut StackValue,
}

impl InterpState {
    #[inline]
    fn new() -> InterpState {
        InterpState {
            instance: None.into(),
            func: None.into(),
            memory: MemoryView::new_unsafe(core::ptr::null_mut(), 0),
            memories: ParaPtr::new(&[]),
            funcs:    ParaPtr::new(&[]),
            globals:  ParaPtr::new(&[]),
            tables:   ParaPtr::new(&[]),
            pc: core::ptr::null(),
            bp: core::ptr::null_mut(),
        }
    }

    // returns true if caller is native code.
    #[inline]
    fn pop_frame(&mut self, store: &mut Store) -> bool {
        let interp = &mut store.interp;

        let frame = interp.frames.last().unwrap();
        let Some(instance) = frame.instance.to_option() else {
            return true;
        };

        if instance.some() != self.instance {
            let inst = &store.instances[instance];
            if inst.memories.len() > 0 {
                // safety: there are no other global references right now.
                let mem = unsafe { &mut *store.memories[inst.memories[0]].get() };
                self.memory = MemoryView::new_unsafe(mem.bytes.as_mut_ptr(), mem.bytes.len());
            }
            else {
                self.memory = MemoryView::new_unsafe(core::ptr::null_mut(), 0);
            }

            self.memories = ParaPtr::new(&inst.memories);
            self.funcs    = ParaPtr::new(&inst.funcs);
            self.globals  = ParaPtr::new(&inst.globals);
            self.tables   = ParaPtr::new(&inst.tables);
        }

        self.instance = frame.instance;
        self.func     = frame.func;

        self.pc = frame.pc;
        self.bp = unsafe { interp.stack.as_mut_ptr().add(frame.bp) };

        interp.frames.pop();
        return false;
    }

    #[inline]
    fn push_frame(&self, store: &mut Store) -> usize {
        let this = &mut store.interp;

        let old_bp = unsafe { self.bp.offset_from(this.stack.as_ptr()) as usize };

        this.frames.push(Frame {
            instance: self.instance,
            func: self.func,
            pc: self.pc,
            bp: old_bp,
        });

        old_bp
    }


    #[inline]
    fn call_bytecode(&mut self, base: u8, func: InterpFuncId, store: &mut Store) {
        let InterpFunc { code: new_pc, stack_size } = store.interp.funcs[func];

        let old_bp = self.push_frame(store);
        let new_bp = old_bp + base as usize;
        unsafe { check_stack(&mut store.interp, new_bp + stack_size) }

        self.func = func.some();
        self.pc   = new_pc;
        self.bp   = unsafe { store.interp.stack.as_mut_ptr().add(new_bp) };
    }

    #[inline]
    fn call_indirect(&mut self, base: u8, func: FuncId, store: &mut Store) -> Result<(), ()> {
        let func = unsafe { self.funcs.read(func.usize()) };
        let func = store.funcs[func];

        match func.data {
            StoreFuncData::Guest(data) => {
                if data.instance.some() != self.instance {
                    unimplemented!()
                }

                self.call_bytecode(base, data.interp, store);
                return Ok(());
            }

            StoreFuncData::Host(data) => {
                // @todo: stack frame for funcs with store access.
                let base = unsafe { self.bp.add(base as usize) };
                return match data.call {
                    HostFuncDataFn::Plain(f) => {
                        f((), data.data, base)
                    }

                    HostFuncDataFn::WithMemory(f) => {
                        f(&mut self.memory, data.data, base)
                    }
                }
            }
        }
    }
}

#[inline]
unsafe fn check_stack(interp: &mut Interp, top: usize) {
    if top > interp.stack.len() { unsafe {
        interp.stack.reserve(top);
        interp.stack.set_len(top);
    }}
}

#[inline(never)]
fn interp(store: &mut Store) -> (Result<(), ()>,) {
    let mut state = InterpState::new();
    if state.pop_frame(store) {
        return (Ok(()),);
    };

    macro_rules! next_word {
        () => { unsafe {
            let word = *state.pc;
            state.pc = state.pc.add(1);
            word
        }};
    }

    macro_rules! next_op {
        () => {{
            let word = next_word!();
            unsafe { word.op }
        }};
    }

    macro_rules! next_word_u32 {
        () => {{
            let word = next_word!();
            unsafe { word.u32 }
        }};
    }


    let result;

    loop {
        macro_rules! reg {
            ($reg: expr) => {{
                unsafe {
                    StackPtr::from_ptr(state.bp.add($reg as usize))
                }
            }};
        }

        macro_rules! vm_err {
            ($err: expr) => {{
                result = Err($err);
                break;
            }};
        }

        macro_rules! vm_try {
            ($expr:expr) => {{
                let res = $expr;
                match res {
                    Ok(v) => v,
                    Err(e) => {
                        result = Err(e);
                        break;
                    }
                }
            }};
        }

        macro_rules! vm_jump {
            ($target: expr) => {
                unsafe { state.pc = state.pc.offset($target as isize) }
            };
        }


        let op = next_op!();

        // println!("{:?}@{}@{:?}: {:?}", state.func, store.interp.frames.len(), state.pc, op);

        use Op::*;
        // @TEMP
        #[allow(unused_variables)]
        match op {
            UNIMPLEMENTED => {
                unimplemented!()
            }

            UNREACHABLE => {
                vm_err!(());
            }

            JUMP { delta } => {
                vm_jump!(delta);
            }

            JUMP_FALSE { src, delta } => {
                let cond = reg!(src).i32();
                if cond == 0 {
                    vm_jump!(delta);
                }
            }

            JUMP_TRUE { src, delta } => {
                let cond = reg!(src).i32();
                if cond != 0 {
                    vm_jump!(delta);
                }
            }

            JUMP_TABLE { src, len } => {
                let len = len as usize;

                let table = unsafe {
                    let table = core::slice::from_raw_parts(state.pc as *const u32, len);
                    state.pc = state.pc.add(len);
                    table
                };
                // we must always have the default label.
                if table.len() < 1 {
                    debug_assert!(false);
                    unsafe { core::hint::unreachable_unchecked() }
                }

                let index = reg!(src).u32() as usize;
                let delta = table[index.min(table.len() - 1)] as u16 as i16;
                vm_jump!(delta);
            }

            RETURN { base, num_rets } => {
                // copy results.
                if base != 0 {
                    for i in 0..num_rets as usize { unsafe {
                        *state.bp.add(i) = *state.bp.add(base as usize).add(i);
                    }}
                }

                if state.pop_frame(store) {
                    return (Ok(()),);
                }
            }

            CALL_INDIRECT { base } => {
                let func = next_word_u32!();
                vm_try!(state.call_indirect(base, FuncId::new_unck(func), store));
            }

            CALL_BYTECODE { base } => {
                let func = InterpFuncId(next_word_u32!());
                state.call_bytecode(base, func, store);
            }

            CALL_TABLE { base, src } => {
                let tab_idx = next_word_u32!() as usize;
                let src = reg!(src).u32() as usize;

                let tab_idx = unsafe { state.tables.read(tab_idx) };

                let Some(func) = store.tables[tab_idx].values.get(src).copied() else {
                    vm_err!(());
                };

                vm_try!(state.call_indirect(base, FuncId::new_unck(func), store));
            }

            SELECT { dst, src1, src2 } => {
                let cond = next_word_u32!();
                if reg!(cond).i32() != 0 {
                    reg!(dst).write(reg!(src1).read());
                }
                else {
                    reg!(dst).write(reg!(src2).read());
                }
            }

            I32_CONST { dst } => {
                let value = next_word_u32!() as i32;
                reg!(dst).write(StackValue::I32(value));
            }

            I64_CONST { dst } => {
                let lo = next_word_u32!() as u64;
                let hi = next_word_u32!() as u64;
                let value = hi << 32 | lo;
                reg!(dst).write(StackValue::I64(value as i64));
            }

            F32_CONST { dst } => {
                unimplemented!()
            }

            F64_CONST { dst } => {
                unimplemented!()
            }

            COPY { dst, src } => {
                reg!(dst).write(reg!(src).read())
            }

            GLOBAL_GET { dst } => {
                let global = next_word_u32!() as usize;
                let global = unsafe { state.globals.read(global) };
                reg!(dst).write(store.globals[global].value);
            }

            GLOBAL_SET { src } => {
                let global = next_word_u32!() as usize;
                let global = unsafe { state.globals.read(global) };
                store.globals[global].value = reg!(src).read();
            }

            LOAD_I32 { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: i32 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::I32(value));
            }

            LOAD_I32_8S { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: i8 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::I32(value as i32));
            }

            LOAD_I32_8U { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: u8 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::U32(value as u32));
            }

            LOAD_I32_16S { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: i16 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::I32(value as i32));
            }

            LOAD_I32_16U { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: u16 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::U32(value as u32));
            }

            LOAD_I64 { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: i64 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::I64(value));
            }

            LOAD_I64_8S { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: i8 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::I64(value as i64));
            }

            LOAD_I64_8U { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: u8 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::U64(value as u64));
            }

            LOAD_I64_16S { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: i16 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::I64(value as i64));
            }

            LOAD_I64_16U { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: u16 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::U64(value as u64));
            }

            LOAD_I64_32S { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: i32 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::I64(value as i64));
            }

            LOAD_I64_32U { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: u32 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::U64(value as u64));
            }

            LOAD_F32 { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: f32 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::F32(value));
            }

            LOAD_F64 { dst, addr } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);

                let value: f64 = vm_try!(state.memory.read(ptr));
                reg!(dst).write(StackValue::F64(value));
            }

            STORE_I32 { addr, src } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);
                vm_try!(state.memory.write(ptr, reg!(src).u32()));
            }

            STORE_I32_8 { addr, src } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);
                vm_try!(state.memory.write(ptr, reg!(src).u32() as u8));
            }

            STORE_I32_16 { addr, src } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);
                vm_try!(state.memory.write(ptr, reg!(src).u32() as u16));
            }

            STORE_I64 { addr, src } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);
                vm_try!(state.memory.write(ptr, reg!(src).i64()));
            }

            STORE_I64_8 { addr, src } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);
                vm_try!(state.memory.write(ptr, reg!(src).u64() as u8));
            }

            STORE_I64_16 { addr, src } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);
                vm_try!(state.memory.write(ptr, reg!(src).u64() as u16));
            }

            STORE_I64_32 { addr, src } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);
                vm_try!(state.memory.write(ptr, reg!(src).u64() as u32));
            }

            STORE_F32 { addr, src } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);
                vm_try!(state.memory.write(ptr, reg!(src).f32()));
            }

            STORE_F64 { addr, src } => {
                let offset = next_word_u32!();
                let ptr = WasmPtr::new(reg!(addr).u32()).wrapping_byte_add(offset);
                vm_try!(state.memory.write(ptr, reg!(src).f64()));
            }

            I32_EQZ { dst, src } => {
                let result = reg!(src).i32() == 0;
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I64_EQZ { dst, src } => {
                let result = reg!(src).i64() == 0;
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I32_CLZ { dst, src } => {
                let result = reg!(src).u32().leading_zeros();
                reg!(dst).write(StackValue::U32(result));
            }

            I32_CTZ { dst, src } => {
                let result = reg!(src).u32().trailing_zeros();
                reg!(dst).write(StackValue::U32(result));
            }

            I32_POPCNT { dst, src } => {
                let result = reg!(src).u32().count_ones();
                reg!(dst).write(StackValue::U32(result));
            }

            I32_EXTEND8_S { dst, src } => {
                unimplemented!()
            }

            I32_EXTEND16_S { dst, src } => {
                unimplemented!()
            }

            I64_CLZ { dst, src } => {
                unimplemented!()
            }

            I64_CTZ { dst, src } => {
                unimplemented!()
            }

            I64_POPCNT { dst, src } => {
                unimplemented!()
            }

            I64_EXTEND8_S { dst, src } => {
                unimplemented!()
            }

            I64_EXTEND16_S { dst, src } => {
                unimplemented!()
            }

            I64_EXTEND32_S { dst, src } => {
                unimplemented!()
            }

            F32_ABS { dst, src } => {
                unimplemented!()
            }

            F32_NEG { dst, src } => {
                unimplemented!()
            }

            F32_CEIL { dst, src } => {
                unimplemented!()
            }

            F32_FLOOR { dst, src } => {
                unimplemented!()
            }

            F32_TRUNC { dst, src } => {
                unimplemented!()
            }

            F32_NEAREST { dst, src } => {
                unimplemented!()
            }

            F32_SQRT { dst, src } => {
                unimplemented!()
            }

            F64_ABS { dst, src } => {
                unimplemented!()
            }

            F64_NEG { dst, src } => {
                unimplemented!()
            }

            F64_CEIL { dst, src } => {
                unimplemented!()
            }

            F64_FLOOR { dst, src } => {
                unimplemented!()
            }

            F64_TRUNC { dst, src } => {
                unimplemented!()
            }

            F64_NEAREST { dst, src } => {
                unimplemented!()
            }

            F64_SQRT { dst, src } => {
                unimplemented!()
            }

            I32_WRAP_I64 { dst, src } => {
                reg!(dst).write(StackValue::U32(reg!(src).u64() as u32));
            }

            I64_EXTEND_I32_S { dst, src } => {
                reg!(dst).write(StackValue::I64(reg!(src).i32() as i64));
            }

            I64_EXTEND_I32_U { dst, src } => {
                reg!(dst).write(StackValue::U64(reg!(src).u32() as u64));
            }

            I32_TRUNC_F32_S { dst, src } => {
                unimplemented!()
            }

            I32_TRUNC_F32_U { dst, src } => {
                unimplemented!()
            }

            F32_CONVERT_I32_S { dst, src } => {
                unimplemented!()
            }

            F32_CONVERT_I32_U { dst, src } => {
                unimplemented!()
            }

            I32_TRUNC_F64_S { dst, src } => {
                unimplemented!()
            }

            I32_TRUNC_F64_U { dst, src } => {
                unimplemented!()
            }

            F64_CONVERT_I32_S { dst, src } => {
                unimplemented!()
            }

            F64_CONVERT_I32_U { dst, src } => {
                unimplemented!()
            }

            I64_TRUNC_F32_S { dst, src } => {
                unimplemented!()
            }

            I64_TRUNC_F32_U { dst, src } => {
                unimplemented!()
            }

            F32_CONVERT_I64_S { dst, src } => {
                unimplemented!()
            }

            F32_CONVERT_I64_U { dst, src } => {
                unimplemented!()
            }

            I64_TRUNC_F64_S { dst, src } => {
                unimplemented!()
            }

            I64_TRUNC_F64_U { dst, src } => {
                unimplemented!()
            }

            F64_CONVERT_I64_S { dst, src } => {
                unimplemented!()
            }

            F64_CONVERT_I64_U { dst, src } => {
                unimplemented!()
            }

            F32_DEMOTE_F64 { dst, src } => {
                unimplemented!()
            }

            F64_PROMOTE_F32 { dst, src } => {
                unimplemented!()
            }

            I32_EQ { dst, src1, src2 } => {
                let result = reg!(src1).i32() == reg!(src2).i32();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I32_NE { dst, src1, src2 } => {
                let result = reg!(src1).i32() != reg!(src2).i32();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I32_LE_S { dst, src1, src2 } => {
                let result = reg!(src1).i32() <= reg!(src2).i32();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I32_LE_U { dst, src1, src2 } => {
                let result = reg!(src1).u32() <= reg!(src2).u32();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I32_LT_S { dst, src1, src2 } => {
                let result = reg!(src1).i32() < reg!(src2).i32();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I32_LT_U { dst, src1, src2 } => {
                let result = reg!(src1).u32() < reg!(src2).u32();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I32_GE_S { dst, src1, src2 } => {
                let result = reg!(src1).i32() >= reg!(src2).i32();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I32_GE_U { dst, src1, src2 } => {
                let result = reg!(src1).u32() >= reg!(src2).u32();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I32_GT_S { dst, src1, src2 } => {
                let result = reg!(src1).i32() > reg!(src2).i32();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I32_GT_U { dst, src1, src2 } => {
                let result = reg!(src1).u32() > reg!(src2).u32();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I64_EQ { dst, src1, src2 } => {
                let result = reg!(src1).i64() == reg!(src2).i64();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I64_NE { dst, src1, src2 } => {
                let result = reg!(src1).i64() != reg!(src2).i64();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I64_LE_S { dst, src1, src2 } => {
                let result = reg!(src1).i64() <= reg!(src2).i64();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I64_LE_U { dst, src1, src2 } => {
                let result = reg!(src1).u64() <= reg!(src2).u64();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I64_LT_S { dst, src1, src2 } => {
                let result = reg!(src1).i64() < reg!(src2).i64();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I64_LT_U { dst, src1, src2 } => {
                let result = reg!(src1).u64() < reg!(src2).u64();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I64_GE_S { dst, src1, src2 } => {
                let result = reg!(src1).i64() >= reg!(src2).i64();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I64_GE_U { dst, src1, src2 } => {
                let result = reg!(src1).u64() >= reg!(src2).u64();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I64_GT_S { dst, src1, src2 } => {
                let result = reg!(src1).i64() > reg!(src2).i64();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            I64_GT_U { dst, src1, src2 } => {
                let result = reg!(src1).u64() > reg!(src2).u64();
                reg!(dst).write(StackValue::I32(result as i32));
            }

            F32_EQ { dst, src1, src2 } => {
                unimplemented!()
            }

            F32_NE { dst, src1, src2 } => {
                unimplemented!()
            }

            F32_LE { dst, src1, src2 } => {
                unimplemented!()
            }

            F32_LT { dst, src1, src2 } => {
                unimplemented!()
            }

            F32_GE { dst, src1, src2 } => {
                unimplemented!()
            }

            F32_GT { dst, src1, src2 } => {
                unimplemented!()
            }

            F64_EQ { dst, src1, src2 } => {
                unimplemented!()
            }

            F64_NE { dst, src1, src2 } => {
                unimplemented!()
            }

            F64_LE { dst, src1, src2 } => {
                unimplemented!()
            }

            F64_LT { dst, src1, src2 } => {
                unimplemented!()
            }

            F64_GE { dst, src1, src2 } => {
                unimplemented!()
            }

            F64_GT { dst, src1, src2 } => {
                unimplemented!()
            }

            I32_ADD { dst, src1, src2 } => {
                let result = reg!(src1).i32().wrapping_add(reg!(src2).i32());
                reg!(dst).write(StackValue::I32(result));
            }

            I32_SUB { dst, src1, src2 } => {
                let result = reg!(src1).i32().wrapping_sub(reg!(src2).i32());
                reg!(dst).write(StackValue::I32(result));
            }

            I32_MUL { dst, src1, src2 } => {
                let result = reg!(src1).i32().wrapping_mul(reg!(src2).i32());
                reg!(dst).write(StackValue::I32(result));
            }

            I32_DIV_S { dst, src1, src2 } => {
                // @temp
                let result = reg!(src1).i32().checked_div(reg!(src2).i32()).unwrap();
                reg!(dst).write(StackValue::I32(result));
            }

            I32_DIV_U { dst, src1, src2 } => {
                // @temp
                let result = reg!(src1).u32().checked_div(reg!(src2).u32()).unwrap();
                reg!(dst).write(StackValue::U32(result));
            }

            I32_REM_S { dst, src1, src2 } => {
                // @temp
                let result = reg!(src1).i32().checked_rem(reg!(src2).i32()).unwrap();
                reg!(dst).write(StackValue::I32(result));
            }

            I32_REM_U { dst, src1, src2 } => {
                // @temp
                let result = reg!(src1).u32().checked_rem(reg!(src2).u32()).unwrap();
                reg!(dst).write(StackValue::U32(result));
            }

            I32_AND { dst, src1, src2 } => {
                let result = reg!(src1).i32() & reg!(src2).i32();
                reg!(dst).write(StackValue::I32(result));
            }

            I32_OR { dst, src1, src2 } => {
                let result = reg!(src1).i32() | reg!(src2).i32();
                reg!(dst).write(StackValue::I32(result));
            }

            I32_XOR { dst, src1, src2 } => {
                let result = reg!(src1).i32() ^ reg!(src2).i32();
                reg!(dst).write(StackValue::I32(result));
            }

            I32_SHL { dst, src1, src2 } => {
                let result = reg!(src1).i32() << reg!(src2).i32();
                reg!(dst).write(StackValue::I32(result));
            }

            I32_SHR_S { dst, src1, src2 } => {
                let result = reg!(src1).i32() >> reg!(src2).i32();
                reg!(dst).write(StackValue::I32(result));
            }

            I32_SHR_U { dst, src1, src2 } => {
                let result = reg!(src1).u32() >> reg!(src2).u32();
                reg!(dst).write(StackValue::U32(result));
            }

            I32_ROTL { dst, src1, src2 } => {
                let result = reg!(src1).u32().rotate_left(reg!(src2).u32());
                reg!(dst).write(StackValue::U32(result));
            }

            I32_ROTR { dst, src1, src2 } => {
                let result = reg!(src1).u32().rotate_right(reg!(src2).u32());
                reg!(dst).write(StackValue::U32(result));
            }

            I64_ADD { dst, src1, src2 } => {
                let result = reg!(src1).i64().wrapping_add(reg!(src2).i64());
                reg!(dst).write(StackValue::I64(result));
            }

            I64_SUB { dst, src1, src2 } => {
                let result = reg!(src1).i64().wrapping_sub(reg!(src2).i64());
                reg!(dst).write(StackValue::I64(result));
            }

            I64_MUL { dst, src1, src2 } => {
                let result = reg!(src1).i64().wrapping_mul(reg!(src2).i64());
                reg!(dst).write(StackValue::I64(result));
            }

            I64_DIV_S { dst, src1, src2 } => {
                // @temp
                let result = reg!(src1).i64().checked_div(reg!(src2).i64()).unwrap();
                reg!(dst).write(StackValue::I64(result));
            }

            I64_DIV_U { dst, src1, src2 } => {
                // @temp
                let result = reg!(src1).u64().checked_div(reg!(src2).u64()).unwrap();
                reg!(dst).write(StackValue::U64(result));
            }

            I64_REM_S { dst, src1, src2 } => {
                // @temp
                let result = reg!(src1).i64().checked_rem(reg!(src2).i64()).unwrap();
                reg!(dst).write(StackValue::I64(result));
            }

            I64_REM_U { dst, src1, src2 } => {
                // @temp
                let result = reg!(src1).u64().checked_rem(reg!(src2).u64()).unwrap();
                reg!(dst).write(StackValue::U64(result));
            }

            I64_AND { dst, src1, src2 } => {
                let result = reg!(src1).i64() & reg!(src2).i64();
                reg!(dst).write(StackValue::I64(result));
            }

            I64_OR { dst, src1, src2 } => {
                let result = reg!(src1).i64() | reg!(src2).i64();
                reg!(dst).write(StackValue::I64(result));
            }

            I64_XOR { dst, src1, src2 } => {
                let result = reg!(src1).i64() ^ reg!(src2).i64();
                reg!(dst).write(StackValue::I64(result));
            }

            I64_SHL { dst, src1, src2 } => {
                let result = reg!(src1).i64() << reg!(src2).i64();
                reg!(dst).write(StackValue::I64(result));
            }

            I64_SHR_S { dst, src1, src2 } => {
                let result = reg!(src1).i64() >> reg!(src2).i64();
                reg!(dst).write(StackValue::I64(result));
            }

            I64_SHR_U { dst, src1, src2 } => {
                let result = reg!(src1).u64() >> reg!(src2).u64();
                reg!(dst).write(StackValue::U64(result));
            }

            I64_ROTL { dst, src1, src2 } => {
                // @temp
                let result = reg!(src1).u64().rotate_left(reg!(src2).u64().try_into().unwrap());
                reg!(dst).write(StackValue::U64(result));
            }

            I64_ROTR { dst, src1, src2 } => {
                // @temp
                let result = reg!(src1).u64().rotate_right(reg!(src2).u64().try_into().unwrap());
                reg!(dst).write(StackValue::U64(result));
            }

            F32_ADD { dst, src1, src2 } => {
                let result = reg!(src1).f32() + reg!(src2).f32();
                reg!(dst).write(StackValue::F32(result));
            }

            F32_SUB { dst, src1, src2 } => {
                let result = reg!(src1).f32() - reg!(src2).f32();
                reg!(dst).write(StackValue::F32(result));
            }

            F32_MUL { dst, src1, src2 } => {
                let result = reg!(src1).f32() * reg!(src2).f32();
                reg!(dst).write(StackValue::F32(result));
            }

            F32_DIV { dst, src1, src2 } => {
                let result = reg!(src1).f32() / reg!(src2).f32();
                reg!(dst).write(StackValue::F32(result));
            }

            F32_MIN { dst, src1, src2 } => {
                let result = reg!(src1).f32().min(reg!(src2).f32());
                reg!(dst).write(StackValue::F32(result));
            }

            F32_MAX { dst, src1, src2 } => {
                let result = reg!(src1).f32().max(reg!(src2).f32());
                reg!(dst).write(StackValue::F32(result));
            }

            F32_COPYSIGN { dst, src1, src2 } => {
                let result = reg!(src1).f32().copysign(reg!(src2).f32());
                reg!(dst).write(StackValue::F32(result));
            }

            F64_ADD { dst, src1, src2 } => {
                let result = reg!(src1).f64() + reg!(src2).f64();
                reg!(dst).write(StackValue::F64(result));
            }

            F64_SUB { dst, src1, src2 } => {
                let result = reg!(src1).f64() - reg!(src2).f64();
                reg!(dst).write(StackValue::F64(result));
            }

            F64_MUL { dst, src1, src2 } => {
                let result = reg!(src1).f64() * reg!(src2).f64();
                reg!(dst).write(StackValue::F64(result));
            }

            F64_DIV { dst, src1, src2 } => {
                let result = reg!(src1).f64() / reg!(src2).f64();
                reg!(dst).write(StackValue::F64(result));
            }

            F64_MIN { dst, src1, src2 } => {
                let result = reg!(src1).f64().min(reg!(src2).f64());
                reg!(dst).write(StackValue::F64(result));
            }

            F64_MAX { dst, src1, src2 } => {
                let result = reg!(src1).f64().max(reg!(src2).f64());
                reg!(dst).write(StackValue::F64(result));
            }

            F64_COPYSIGN { dst, src1, src2 } => {
                let result = reg!(src1).f64().copysign(reg!(src2).f64());
                reg!(dst).write(StackValue::F64(result));
            }

            MEMORY_SIZE { dst } => {
                let pages = state.memory.size() / wasm::PAGE_SIZE;
                reg!(dst).write(StackValue::U32(pages as u32));
            }

            MEMORY_GROW { dst, delta } => {
                let delta = reg!(delta).i32();
                if delta < 0 {
                    unimplemented!();
                }

                let memory = unsafe { state.memories.read(0) };
                // safety: there are no other global references right now.
                let memory = unsafe { &mut *store.memories[memory].get() };
                let old_pages = memory.bytes.len() / wasm::PAGE_SIZE;

                let old_size = memory.bytes.len();
                let new_size = old_size + (delta as usize)*wasm::PAGE_SIZE;
                unsafe {
                    memory.bytes.reserve_exact(new_size);
                    core::ptr::write_bytes(memory.bytes.as_mut_ptr().add(old_size), 0, new_size-old_size);
                    memory.bytes.set_len(new_size);
                }
                state.memory = MemoryView::new_unsafe(memory.bytes.as_mut_ptr(), memory.bytes.len());

                reg!(dst).write(StackValue::U32(old_pages as u32));
            }

            MEMORY_COPY { dst_addr, src_addr, len } => {
                let dst = WasmPtr::new(reg!(dst_addr).u32());
                let src = WasmPtr::new(reg!(src_addr).u32());
                let len = reg!(len).u32();
                vm_try!(state.memory.copy(dst, src, WasmSize(len)));
            }

            MEMORY_FILL { dst_addr, val, len } => {
                let dst = WasmPtr::new(reg!(dst_addr).u32());
                let val = reg!(val).u32();
                let len = reg!(len).u32();
                vm_try!(state.memory.fill(dst, val as u8, WasmSize(len)));
            }
        }
    }

    return (result,);
}

