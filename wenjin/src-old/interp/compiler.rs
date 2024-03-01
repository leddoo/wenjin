use sti::alloc::Alloc;
use sti::vec::Vec;
use sti::keyed::{KVec, Key};

use crate::{wasm, interp};

use wasm::*;
use wasm::operand::*;

use interp::instr::*;
use interp::bytecode;
use interp::InterpFuncIds;




pub(crate) struct Compiler<'m, A: Alloc + Copy> {
    alloc: A,

    module: &'m Module<'m>,

    ty_idx: u32,  // @todo TypeIdx.

    locals: KVec<Local, LocalState, A>,
    stack: Vec<OptInstrId, A>,

    next_label: Label,

    frames: Vec<Frame, A>,
    frame:  Frame,

    func_range: InterpFuncIds,

    code: KVec<InstrId, InstrData, A>,

    jump_tables: KVec<JumpTable, Vec<Label, A>, A>,
}


sti::define_key!(u32, InstrId, opt: OptInstrId);

struct LocalState {
    version:     u32,
    last_access: InstrId,
}

impl<'m, A: Alloc + Copy> Compiler<'m, A> {
    pub fn new(module: &'m Module, ty_idx: u32, locals: &[ValueType], func_range: InterpFuncIds, alloc: A) -> Self {
        if locals.len() > 127 {
            unimplemented!()
        }

        let mut local_states = KVec::with_cap_in(alloc, locals.len());
        let mut stack = Vec::with_cap_in(alloc, 64);
        for _ in 0..locals.len() {
            local_states.push(LocalState {
                version:     0,
                last_access: InstrId(0),
            });
            stack.push(None.into());
        }


        let mut compiler = Compiler {
            alloc,

            module,

            ty_idx,

            locals: local_states,
            stack,

            next_label: Label::new_unck(0),
            frames: Vec::with_cap_in(alloc, 32),
            frame:  Frame { kind: FrameKind::Block, ty: BlockType::Unit, label: Label::MAX, height: 0, unreachable: false },

            func_range,

            code: KVec::with_cap_in(alloc, 512),

            jump_tables: KVec::new_in(alloc),
        };

        compiler.push_frame(FrameKind::Block, BlockType::Unit);


        let func_ty = module.types[ty_idx as usize];
        let num_params = func_ty.params.len();

        // init locals.
        for (i, local) in locals.iter().enumerate().skip(num_params) {
            let dst = Local::new_unck(i as u8).some();

            compiler.code.push(match *local {
                ValueType::I32 => InstrData::I32Const { dst, value: 0   },
                ValueType::I64 => InstrData::I64Const { dst, value: 0   },
                ValueType::F32 => InstrData::F32Const { dst, value: 0.0 },
                ValueType::F64 => InstrData::F64Const { dst, value: 0.0 },

                _ => unimplemented!()
            });
        }

        compiler
    }


    #[inline]
    pub fn stack_height(&self) -> usize {
        self.stack.len() - self.locals.len()
    }

    #[inline]
    pub fn num_frames(&self) -> usize {
        self.frames.len()
    }


    #[inline]
    pub fn add_operand(&mut self, operand: &Operand, br_table: Option<&(&[u32], u32)>) {
        debug_assert!(self.num_frames() > 0);

        if self.frame.unreachable {
            // @todo: broken?
            if let OperandData::End = operand.data {
                self.handle_end();
            }

            return;
        }

        use OperandData::*;
        match operand.data {
            Unreachable => {
                self.add_instr(InstrData::Unreachable);
                self.unreachable();
            },

            Nop => {}


            Block (ty) => {
                self.push_frame(FrameKind::Block, ty);
            }

            Loop (ty) => {
                self.push_frame(FrameKind::Loop, ty);
                self.add_instr(InstrData::Label { id: self.frame.label });
            }

            If (ty) => {
                let src = self.pop_local();

                let else_label = self.next_label.next().unwrap();

                self.add_instr(InstrData::JumpFalse { src, target: else_label });

                self.push_frame(FrameKind::If { else_label }, ty);
            }

            Else => {
                let frame = self.pop_frame();

                let FrameKind::If { else_label } = frame.kind else { unreachable!() };
                self.push_frame(FrameKind::Else, frame.ty);

                self.add_instr(InstrData::Label { id: else_label });
            }

            End => {
                self.handle_end();
            }


            Br (label) => {
                let frame = self.label(label);
                let target  = frame.label;
                let br_pops = self.frame_br_pops(frame);

                self.pop_n(br_pops);

                self.add_instr(InstrData::Jump { target });
                self.unreachable();
            }

            BrIf (label) => {
                let frame = self.label(label);
                let target  = frame.label;
                let br_pops = self.frame_br_pops(frame);

                let src = self.pop_local();
                self.pop_n(br_pops);

                self.add_instr(InstrData::JumpTrue { src, target });
            }

            BrTable => {
                let src = self.pop_local();

                let (labels, default) = *br_table.unwrap();

                let mut table = Vec::with_cap_in(self.alloc, labels.len() + 1);
                for label in labels.iter().copied() {
                    table.push(self.label(label).label);
                }
                table.push(self.label(default).label);

                let table = self.jump_tables.push(table);

                self.add_instr(InstrData::JumpTable { src, table });
                self.unreachable();
            }


            Return => {
                // @todo: special case for return 1 value.
                let rets = self.module.types[self.ty_idx as usize].rets;
                self.pop_n(rets.len());
                self.add_instr(InstrData::Return { num_rets: rets.len() as u8 });
                self.unreachable();
            }

            Call (func) => {
                let func = func as usize;

                let imports = self.module.imports.funcs;

                let ty_idx = imports.get(func).copied().unwrap_or_else(||
                    self.module.func_types[func - imports.len()]
                );

                let ty = &self.module.types[ty_idx as usize];
                self.pop_n(ty.params.len());
                self.push_n(ty.rets.len());

                let num_args = ty.params.len() as u8;
                let num_rets = ty.rets.len()   as u8;

                if func < imports.len() {
                    self.add_instr(InstrData::CallIndirect { num_args, num_rets, index: func as u32 });
                }
                else {
                    let index = func - imports.len();
                    let func = self.func_range.idx(index);
                    self.add_instr(InstrData::CallBytecode { num_args, num_rets, func });
                }
            }

            CallIndirect (tab_idx, ty_idx) => {
                let ty = self.module.types.get(ty_idx as usize).unwrap();
                let src = self.pop_local();
                self.pop_n(ty.params.len());
                self.push_n(ty.rets.len());

                let num_args = ty.params.len() as u8;
                let num_rets = ty.rets.len()   as u8;
                self.add_instr(InstrData::CallTable { num_args, num_rets, src, tab_idx });
            }


            Drop => {
                self.pop();
                self.add_instr(InstrData::Drop);
            }

            Select => {
                let cond = self.pop_local();
                let src2 = self.pop_local();
                let src1 = self.pop_local();
                let dst = self.add_instr(InstrData::Select { dst: None.into(), src1, src2, cond });
                self.push(dst);
            }


            I32Const (value) => {
                let dst = self.add_instr(InstrData::I32Const { dst: None.into(), value });
                self.push(dst);
            }

            I64Const (value) => {
                let dst = self.add_instr(InstrData::I64Const { dst: None.into(), value });
                self.push(dst);
            }

            F32Const (value) => {
                let dst = self.add_instr(InstrData::F32Const { dst: None.into(), value });
                self.push(dst);
            }

            F64Const (value) => {
                let dst = self.add_instr(InstrData::F64Const { dst: None.into(), value });
                self.push(dst);
            }


            LocalGet (local) => {
                let local = Local::new_unck(local as u8);
                let version = self.locals[local].version;
                let dst = self.add_instr(InstrData::LocalGet { local, version });
                self.push(dst);
                self.locals[local].last_access = dst;
            }

            LocalSet (local) => {
                let local = Local::new_unck(local as u8);
                if let Err(instr) = self.forward_store(local) {
                    let src = self.use_local(instr);
                    self.add_instr(InstrData::LocalSet { local, src });
                }
                self.locals[local].version += 1;
            }

            LocalTee (local) => {
                let local = Local::new_unck(local as u8);

                match self.forward_store(local) {
                    Ok(()) => {
                        self.locals[local].version += 1;
                        let version = self.locals[local].version;
                        let dst = self.add_instr(InstrData::LocalGet { local, version });
                        self.push(dst);
                        self.locals[local].last_access = dst;
                    }

                    Err(_) => {
                        self.add_instr(InstrData::LocalCopy { local, src: None.into() });
                        self.push_n(1);
                        self.locals[local].version += 1;
                    }
                }
            }

            GlobalGet (global) => {
                let dst = self.add_instr(InstrData::GlobalGet { dst: None.into(), global });
                self.push(dst);
            }

            GlobalSet (global) => {
                let src = self.pop_local();
                self.add_instr(InstrData::GlobalSet { global, src });
            }


            Load { load, align: _, offset } => {
                let addr = self.pop_local();
                let dst = self.add_instr(InstrData::Load { dst: None.into(), addr, load, offset });
                self.push(dst);
            }

            Store { store, align: _, offset } => {
                let src  = self.pop_local();
                let addr = self.pop_local();
                self.add_instr(InstrData::Store { addr, src, store, offset });
            }


            TestOp (op) => {
                let src = self.pop_local();
                let dst = self.add_instr(InstrData::Op1 { dst: None.into(), src, op: op.into() });
                self.push(dst);
            }

            RelOp (op) => {
                let src2 = self.pop_local();
                let src1 = self.pop_local();
                let dst  = self.add_instr(InstrData::Op2 { dst: None.into(), src1, src2, op: op.into() });
                self.push(dst);
            }

            Op1 (op) => {
                let src = self.pop_local();
                let dst = self.add_instr(InstrData::Op1 { dst: None.into(), src, op: op.into() });
                self.push(dst);
            }

            Op2 (op) => {
                let src2 = self.pop_local();
                let src1 = self.pop_local();
                let dst  = self.add_instr(InstrData::Op2 { dst: None.into(), src1, src2, op: op.into() });
                self.push(dst);
            }


            Convert (cvt) => {
                let src = self.pop_local();
                let dst = self.add_instr(InstrData::Op1 { dst: None.into(), src, op: cvt.into() });
                self.push(dst);
            }

            Reinterpret (_) => {
                // no-op
            }



            MemorySize => {
                let dst = self.add_instr(InstrData::MemorySize { dst: None.into() });
                self.push(dst);
            }

            MemoryGrow => {
                let delta = self.pop_local();
                let dst = self.add_instr(InstrData::MemoryGrow { dst: None.into(), delta });
                self.push(dst);
            }

            MemoryCopy => {
                let len = self.pop_local();
                let src_addr = self.pop_local();
                let dst_addr = self.pop_local();
                self.add_instr(InstrData::MemoryCopy { dst_addr, src_addr, len });
            }

            MemoryFill => {
                let len = self.pop_local();
                let val = self.pop_local();
                let dst_addr = self.pop_local();
                self.add_instr(InstrData::MemoryFill { dst_addr, val, len });
            }
        }
    }

    fn handle_end(&mut self) {
        if self.num_frames() > 1 {
            let frame = self.pop_frame();

            let num_rets = frame.ty.end_types(self.module).len();
            self.push_n(num_rets);

            match frame.kind {
                FrameKind::Block |
                FrameKind::Else => {
                    self.add_instr(InstrData::Label { id: frame.label });
                }

                FrameKind::If { else_label } => {
                    // this if doesn't have an else.
                    self.add_instr(InstrData::Label { id: else_label });
                    self.add_instr(InstrData::Label { id: frame.label });
                }

                // loop has label at beginning.
                FrameKind::Loop => {},
            }
        }
        else {
            if self.frame.unreachable == false {
                let rets = self.module.types[self.ty_idx as usize].rets;
                self.pop_n(rets.len());
                self.add_instr(InstrData::Return { num_rets: rets.len() as u8 });
            }
            self.pop_frame();
            debug_assert_eq!(self.stack_height(), 0);
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FrameKind {
    Block,
    Loop,
    If { else_label: Label },
    Else,
}

#[derive(Clone, Copy)]
struct Frame {
    kind:        FrameKind,
    ty:          BlockType,
    label:       Label,
    height:      u32,
    unreachable: bool,
}

impl<'m, A: Alloc + Copy> Compiler<'m, A> {
    fn push_frame(&mut self, kind: FrameKind, ty: BlockType) {
        let label = self.next_label.next().unwrap();

        self.frames.push(self.frame);
        self.frame = Frame {
            kind,
            ty,
            label,
            height:      self.stack.len() as u32,
            unreachable: self.frame.unreachable,
        };
    }

    fn pop_frame(&mut self) -> Frame {
        if self.frame.unreachable == false {
            let num_rets = self.frame.ty.end_types(self.module).len() as u32;
            debug_assert_eq!(self.stack.len() as u32, self.frame.height + num_rets);
        }

        let result = self.frame;
        self.frame = self.frames.pop().unwrap();
        self.stack.truncate(result.height as usize);
        return result;
    }

    fn unreachable(&mut self) {
        self.stack.truncate(self.frame.height as usize);
        self.frame.unreachable = true;
    }

    fn label(&self, label: u32) -> &Frame {
        if label == 0 {
            &self.frame
        }
        else {
            self.frames.rev(label as usize - 1)
        }
    }

    #[inline]
    fn frame_br_pops(&self, frame: &Frame) -> usize {
        if frame.kind == FrameKind::Loop {
            // br in a loop means continue.
            // so we need the initial types again.
            frame.ty.begin_types(self.module).len()
        }
        else {
            // otherwise, break.
            frame.ty.end_types(self.module).len()
        }
    }


    #[inline]
    fn push(&mut self, instr: InstrId) {
        debug_assert!(self.frame.unreachable == false);

        if self.stack.len() >= 256 {
            unimplemented!()
        }

        self.stack.push(instr.some());
    }

    #[inline]
    fn push_n(&mut self, n: usize) {
        debug_assert!(self.frame.unreachable == false);

        if self.stack.len() + n > 256 {
            unimplemented!();
        }

        for _ in 0..n {
            self.stack.push(None.into());
        }
    }


    #[inline]
    fn pop(&mut self) -> OptInstrId {
        debug_assert!(self.frame.unreachable == false);
        debug_assert!(self.stack.len() > self.locals.len());
        return self.stack.pop().unwrap();
    }

    #[inline]
    fn pop_local(&mut self) -> OptLocal {
        let instr = self.pop();
        self.use_local(instr)
    }

    #[inline]
    fn use_local(&mut self, instr: OptInstrId) -> OptLocal {
        let Some(instr) = instr.to_option() else { return None.into() };

        let next_instr = self.code.next_key();

        let instr = &mut self.code[instr];
        let InstrData::LocalGet { local, version } = *instr else { return None.into() };

        let state = &mut self.locals[local];
        if state.version == version {
            state.last_access = next_instr;
            *instr = InstrData::Nop;

            local.some()
        }
        else {
            None.into()
        }
    }

    #[inline]
    fn forward_store(&mut self, local: Local) -> Result<(), OptInstrId> {
        let instr = self.pop();
        if let Some(instr) = instr.to_option() {
            if let Some(dst) = self.code[instr].dst() {
                if self.locals[local].last_access <= instr {
                    debug_assert!(dst.is_none());
                    *dst = local.some();
                    return Ok(());
                }
            }
        }
        Err(instr)
    }

    #[inline]
    fn pop_n(&mut self, n: usize) {
        debug_assert!(self.frame.unreachable == false);
        debug_assert!(self.stack.len() >= self.locals.len() + n);
        self.stack.truncate(self.stack.len() - n);
    }


    #[inline]
    fn add_instr(&mut self, instr: InstrData) -> InstrId {
        debug_assert!(self.frame.unreachable == false);

        // println!("{:?}", instr);
        return self.code.push(instr);
    }
}



impl<'m, A: Alloc + Copy> Compiler<'m, A> {
    pub fn build<B: Alloc>(&self, alloc: B) -> (Vec<bytecode::Word, B>, u32) {
        let mut builder = bytecode::Builder::new(
            self.code.len(),
            self.next_label.usize(),
            alloc,
            self.alloc);


        struct Stack<A: Alloc> {
            top:     u32,
            max_top: u32,

            unreachable: bool,
            label_tops: KVec<Label, Option<u8>, A>,
        }

        impl<A: Alloc> Stack<A> {
            #[inline]
            fn new(num_locals: u32, num_labels: u32, alloc: A) -> Self {
                let num_labels = num_labels as usize;
                let mut label_tops = KVec::with_cap_in(alloc, num_labels);
                for _ in 0..num_labels {
                    label_tops.push(None);
                }

                Self { 
                    top:     num_locals,
                    max_top: num_locals,
                    unreachable: false,
                    label_tops,
                }
            }

            #[inline]
            fn pop(&mut self) -> u8 {
                self.top -= 1;
                return self.top as u8;
            }

            #[inline]
            fn pop_local(&mut self, local: OptLocal) -> u8 {
                if let Some(local) = local.to_option() {
                    local.inner()
                }
                else {
                    self.pop()
                }
            }

            #[inline]
            fn pop_n(&mut self, n: u32) -> u8 {
                self.top -= n;
                return self.top as u8;
            }

            #[inline]
            fn push(&mut self) -> u8 {
                let result = self.top as u8;
                self.top += 1;
                self.max_top = self.max_top.max(self.top);
                return result;
            }

            #[inline]
            fn push_n(&mut self, n: u32) {
                self.top += n;
                self.max_top = self.max_top.max(self.top);
            }

            #[inline]
            fn push_local(&mut self, local: OptLocal) -> u8 {
                if let Some(local) = local.to_option() {
                    local.inner()
                }
                else {
                    self.push()
                }
            }

            #[inline]
            fn visit_label(&mut self, label: Label) {
                if let Some(top) = self.label_tops[label] {
                    if self.unreachable {
                        self.top = top as u32;
                    }
                    else {
                        assert_eq!(self.top, top as u32);
                    }
                }
                else {
                    self.label_tops[label] = Some(self.top as u8);
                }
            }
        }

        let mut stack = Stack::new(self.locals.len() as u32, self.next_label.inner(), self.alloc);

        for instr in self.code.inner() {
            use bytecode::*;

            // println!("{} - {:?}", stack.top, instr);

            if let InstrData::Label { id } = *instr {
                stack.visit_label(id);
                stack.unreachable = false;
            }
            //assert!(stack.unreachable == false);
            if stack.unreachable {
                println!("WARN: unreachable code.");
            }

            use InstrData::*;
            match *instr {
                Unimplemented { pop, push } => {
                    stack.pop_n(pop as u32);
                    stack.push_n(push as u32);
                    builder.op(Op::UNIMPLEMENTED);
                }

                Unreachable => {
                    builder.op(Op::UNREACHABLE);
                    stack.unreachable = true;
                }

                Nop => {}

                Label { id } => {
                    builder.label(id.inner());
                }

                Jump { target } => {
                    builder.op(Op::JUMP { delta: target.inner() as u16 as i16 });
                    stack.visit_label(target);
                    stack.unreachable = true;
                }

                JumpFalse { src, target } => {
                    let src = stack.pop_local(src);
                    builder.op(Op::JUMP_FALSE { src, delta: target.inner() as u16 as i16 });
                    stack.visit_label(target);
                }

                JumpTrue { src, target } => {
                    let src = stack.pop_local(src);
                    builder.op(Op::JUMP_TRUE { src, delta: target.inner() as u16 as i16 });
                    stack.visit_label(target);
                }

                JumpTable { src, table } => {
                    let src = stack.pop_local(src);
                    let table = &self.jump_tables[table];
                    builder.op(Op::JUMP_TABLE { src, len: table.len() as u8 });
                    for entry in table.iter().copied() {
                        builder.u32(entry.inner());
                        stack.visit_label(entry);
                    }
                    stack.unreachable = true;
                }

                Return { num_rets } => {
                    let base = stack.pop_n(num_rets as u32);
                    builder.op(Op::RETURN { base, num_rets });
                    stack.unreachable = true;
                }

                CallIndirect { num_args, num_rets, index } => {
                    let base = stack.pop_n(num_args as u32);
                    stack.push_n(num_rets as u32);
                    builder.op(Op::CALL_INDIRECT { base });
                    builder.u32(index);
                }

                CallBytecode { num_args, num_rets, func } => {
                    let base = stack.pop_n(num_args as u32);
                    stack.push_n(num_rets as u32);
                    builder.op(Op::CALL_BYTECODE { base });
                    builder.u32(func.inner());
                }

                CallTable { num_args, num_rets, src, tab_idx } => {
                    let src = stack.pop_local(src);
                    let base = stack.pop_n(num_args as u32);
                    stack.push_n(num_rets as u32);
                    builder.op(Op::CALL_TABLE { base, src });
                    builder.u32(tab_idx);
                }

                Drop => {
                    stack.pop();
                }

                Select { dst, src1, src2, cond } => {
                    let cond = stack.pop_local(cond);
                    let src2 = stack.pop_local(src2);
                    let src1 = stack.pop_local(src1);
                    let dst  = stack.push_local(dst);
                    builder.op(Op::SELECT { dst, src1, src2 });
                    builder.u32(cond as u32);
                }

                I32Const { dst, value } => {
                    let dst = stack.push_local(dst);
                    builder.op(Op::I32_CONST { dst });
                    builder.u32(value as u32);
                }

                I64Const { dst, value } => {
                    let dst = stack.push_local(dst);
                    builder.op(Op::I64_CONST { dst });
                    builder.u64(value as u64);
                }

                F32Const { dst, value } => {
                    let dst = stack.push_local(dst);
                    builder.op(Op::F32_CONST { dst });
                    builder.u32(value.to_bits());
                }

                F64Const { dst, value } => {
                    let dst = stack.push_local(dst);
                    builder.op(Op::F64_CONST { dst });
                    builder.u64(value.to_bits());
                }

                LocalGet { local, version: _ } => {
                    let dst = stack.push();
                    builder.op(Op::COPY { dst, src: local.inner() });
                }

                LocalSet { local, src } => {
                    let src = stack.pop_local(src);
                    builder.op(Op::COPY { dst: local.inner(), src });
                }

                LocalCopy { local, src } => {
                    let src =
                        if let Some(l) = src.to_option() {
                            l.inner()
                        }
                        else {
                            let src = stack.pop();
                            stack.push();
                            src
                        };
                    builder.op(Op::COPY { dst: local.inner(), src });
                }

                GlobalGet { dst, global } => {
                    let dst = stack.push_local(dst);
                    builder.op(Op::GLOBAL_GET { dst });
                    builder.u32(global);
                }

                GlobalSet { global, src } => {
                    let src = stack.pop_local(src);
                    builder.op(Op::GLOBAL_SET { src });
                    builder.u32(global);
                }

                Load { dst, addr, load, offset } => {
                    let addr = stack.pop_local(addr);
                    let dst  = stack.push_local(dst);
                    builder.op(Op::from_load(load, dst, addr));
                    builder.u32(offset);
                }

                Store { addr, src, store, offset } => {
                    let src  = stack.pop_local(src);
                    let addr = stack.pop_local(addr);
                    builder.op(Op::from_store(store, addr, src));
                    builder.u32(offset);
                }

                Op1 { dst, src, op } => {
                    let src = stack.pop_local(src);
                    let dst = stack.push_local(dst);
                    builder.op(Op::from_op1(op, dst, src));
                }

                Op2 { dst, src1, src2, op } => {
                    let src2 = stack.pop_local(src2);
                    let src1 = stack.pop_local(src1);
                    let dst  = stack.push_local(dst);
                    builder.op(Op::from_op2(op, dst, src1, src2));
                }

                MemorySize { dst } => {
                    let dst = stack.push_local(dst);
                    builder.op(Op::MEMORY_SIZE { dst });
                }

                MemoryGrow { dst, delta } => {
                    let delta = stack.pop_local(delta);
                    let dst = stack.push_local(dst);
                    builder.op(Op::MEMORY_GROW { dst, delta });
                }

                MemoryCopy { dst_addr, src_addr, len } => {
                    let len = stack.pop_local(len);
                    let src_addr = stack.pop_local(src_addr);
                    let dst_addr = stack.pop_local(dst_addr);
                    builder.op(Op::MEMORY_COPY { dst_addr, src_addr, len });
                }

                MemoryFill { dst_addr, val, len } => {
                    let len = stack.pop_local(len);
                    let val = stack.pop_local(val);
                    let dst_addr = stack.pop_local(dst_addr);
                    builder.op(Op::MEMORY_FILL { dst_addr, val, len });
                }
            }
        }

        return (builder.build(), stack.max_top);
    }
}

