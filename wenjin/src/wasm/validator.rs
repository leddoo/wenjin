use sti::alloc::Alloc;
use sti::vec::Vec;

use super::*;
use super::operand::*;



#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControlFrameKind {
    Block,
    Loop,
    If,
    Else,
}

#[derive(Clone, Copy)]
struct ControlFrame {
    kind:        ControlFrameKind,
    ty:          BlockType,
    height:      usize,
    unreachable: bool,
}



pub struct Stack<'m, A: Alloc> {
    pub module: &'m Module<'m>,

    ty_idx: u32, // @todo: TypeIdx.

    values: Vec<ValueType, A>,

    frames: Vec<ControlFrame, A>,
    frame: ControlFrame,
}

impl<'m, A: Alloc> Stack<'m, A> {
    pub fn new_in(module: &'m Module<'m>, ty_idx: u32, alloc: A) -> Self  where A: Copy {
        let mut stack = Stack {
            module,
            ty_idx,
            values: Vec::with_cap_in(alloc, 256),
            frames: Vec::with_cap_in(alloc,  64),
            frame: ControlFrame {
                kind:        ControlFrameKind::Block,
                ty:          BlockType::Func(u32::MAX),
                height:      0,
                unreachable: false,
            },
        };

        stack.push_frame(ControlFrameKind::Block, BlockType::Unit);

        stack
    }

    #[inline]
    pub fn stack_height(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn num_frames(&self) -> usize {
        self.frames.len()
    }

    pub fn done(&self) -> bool {
        self.num_frames() == 0
    }


    fn unreachable(&mut self) {
        debug_assert!(self.values.len() >= self.frame.height);
        debug_assert!(self.frames.len() >  0);

        self.frame.unreachable = true;
        self.values.truncate(self.frame.height);
    }

    fn label(&self, index: u32) -> Result<&ControlFrame, ()> {
        let index = index as usize;
        if index >= self.num_frames() {
            return Err(());
        }

        Ok(if index == 0 {
            &self.frame
        }
        else {
            self.frames.rev(index - 1)
        })
    }


    #[inline]
    fn block_begin_types(&self, ty: BlockType) -> &'m [ValueType] {
        ty.begin_types(self.module)
    }

    #[inline]
    fn block_end_types(&self, ty: BlockType) -> &'m [ValueType] {
        ty.end_types(self.module)
    }

    #[inline]
    fn frame_br_types(&self, frame: &ControlFrame) -> &'m [ValueType] {
        if frame.kind == ControlFrameKind::Loop {
            // br in a loop means continue.
            // so we need the initial types again.
            self.block_begin_types(frame.ty)
        }
        else {
            // otherwise, break.
            self.block_end_types(frame.ty)
        }
    }


    fn push(&mut self, ty: ValueType) {
        debug_assert!(self.values.len() >= self.frame.height);
        debug_assert!(self.frames.len() >  0);

        if !self.frame.unreachable {
            self.values.push(ty);
        }
    }

    fn push_n(&mut self, tys: &[ValueType]) {
        for ty in tys {
            self.push(*ty);
        }
    }

    fn pop(&mut self) -> Result<Option<ValueType>, ()> {
        debug_assert!(self.values.len() >= self.frame.height);
        debug_assert!(self.frames.len() >  0);

        if self.values.len() == self.frame.height {
            if self.frame.unreachable {
                Ok(None)
            }
            else {
                Err(())
            }
        }
        else {
            Ok(Some(self.values.pop().unwrap()))
        }
    }

    fn expect(&mut self, ty: ValueType) -> Result<(), ()> {
        let top = self.pop()?;
        if top != Some(ty) && top != None {
            return Err(());
        }
        return Ok(());
    }

    fn expect_n(&mut self, tys: &[ValueType]) -> Result<(), ()> {
        for ty in tys.iter().rev() {
            self.expect(*ty)?;
        }
        Ok(())
    }


    fn push_frame(&mut self, kind: ControlFrameKind, ty: BlockType) {
        self.push_n(self.block_begin_types(ty));
        self.frames.push(self.frame);
        self.frame = ControlFrame {
            kind, ty,
            height:      self.values.len(),
            unreachable: false,
        }
    }

    fn pop_frame(&mut self) -> Result<ControlFrame, ()> {
        debug_assert!(self.frames.len() > 0);

        let end_types = self.block_end_types(self.frame.ty);
        let height    = self.frame.height;

        self.expect_n(&end_types)?;
        if self.values.len() != height {
            return Err(());
        }

        let result = self.frame;
        self.frame = self.frames.pop().ok_or(())?;
        return Ok(result);
    }
}



#[inline]
pub fn validate_operand<A: Alloc>(
    operand: &Operand,
    locals:  &[ValueType],
    stack:   &mut Stack<A>,
    br_table: Option<&(&[u32], u32)>,
) -> Result<(), ()> {
    debug_assert!(stack.num_frames() > 0);

    let module = stack.module;

    use OperandData::*;
    match operand.data {
        Unreachable => {
            stack.unreachable();
        }

        Nop => {
        }

        Block (ty) => {
            stack.expect_n(ty.begin_types(module))?;
            stack.push_frame(ControlFrameKind::Block, ty);
        }

        Loop (ty) => {
            stack.expect_n(ty.begin_types(module))?;
            stack.push_frame(ControlFrameKind::Loop, ty);
        }

        If (ty) => {
            stack.expect(ValueType::I32)?;
            stack.expect_n(ty.begin_types(module))?;
            stack.push_frame(ControlFrameKind::If, ty);
        }

        Else => {
            let frame = stack.pop_frame().unwrap();
            assert_eq!(frame.kind, ControlFrameKind::If);
            stack.push_frame(ControlFrameKind::Else, frame.ty);
        }

        End => {
            if stack.num_frames() > 1 {
                let frame = stack.pop_frame().unwrap();
                stack.push_n(stack.block_end_types(frame.ty));
            }
            else {
                // implicit return.
                let ty = stack.module.types[stack.ty_idx as usize];
                stack.expect_n(ty.rets)?;
                stack.pop_frame().unwrap();
                debug_assert_eq!(stack.stack_height(), 0);
                debug_assert_eq!(stack.num_frames(),   0);
            }
        }


        Br (label) => {
            let frame = stack.label(label)?;
            stack.expect_n(stack.frame_br_types(&frame))?;
            stack.unreachable();
        }

        BrIf (label) => {
            let frame = stack.label(label)?;
            let tys = stack.frame_br_types(&frame);
            stack.expect(ValueType::I32)?;
            stack.expect_n(tys)?;
            stack.push_n(tys);
        }

        BrTable => {
            let (labels, default) = *br_table.unwrap();

            stack.expect(ValueType::I32)?;

            let frame       = stack.label(default)?;
            let default_tys = stack.frame_br_types(&frame);

            for label in labels.iter().copied() {
                let frame = stack.label(label)?;
                let tys = stack.frame_br_types(&frame);
                if tys.len() != default_tys.len() {
                    return Err(());
                }
                stack.expect_n(tys)?;
                stack.push_n(tys);
            }
            stack.expect_n(default_tys)?;

            stack.unreachable();
        }


        Return => {
            let ty = stack.module.types[stack.ty_idx as usize];
            stack.expect_n(ty.rets)?;
            stack.unreachable();
        }

        Call (func) => {
            let func = func as usize;

            let imports = stack.module.imports.funcs;

            let ty =
                if func < imports.len() {
                    let ty_idx = imports[func];

                    let ty = &module.types[ty_idx as usize];
                    if ty.rets.len() > 1 {
                        unimplemented!()
                    }

                    ty
                }
                else {
                    let func = func - imports.len();
                    let ty_idx = *module.func_types.get(func).unwrap();
                    &module.types[ty_idx as usize]
                };

            stack.expect_n(ty.params)?;
            stack.push_n(ty.rets);
        }

        CallIndirect (tab_idx, ty_idx) => {
            let tab = module.tables.get(tab_idx as usize).unwrap();
            if tab.ty != ValueType::FuncRef {
                unimplemented!()
            }

            let ty = module.types.get(ty_idx as usize).unwrap();
            stack.expect(ValueType::I32)?;
            stack.expect_n(ty.params)?;
            stack.push_n(ty.rets);
        }


        Drop => {
            stack.pop()?;
        }

        Select => {
            stack.expect(ValueType::I32)?;

            if !stack.frame.unreachable {
                let t1 = stack.pop()?.unwrap();
                let t2 = stack.pop()?.unwrap();

                if t1.is_ref() || t2.is_ref() {
                    return Err(());
                }

                if t1 != t2 {
                    return Err(());
                }

                stack.push(t1);
            }
        }


        I32Const (_) => {
            stack.push(ValueType::I32);
        }

        I64Const (_) => {
            stack.push(ValueType::I64);
        }

        F32Const (_) => {
            stack.push(ValueType::F32);
        }

        F64Const (_) => {
            stack.push(ValueType::F64);
        }


        LocalGet (index) => {
            let ty = *locals.get(index as usize).unwrap();
            stack.push(ty);
        }

        LocalSet (index) => {
            let ty = *locals.get(index as usize).unwrap();
            stack.expect(ty)?;
        }

        LocalTee (index) => {
            let ty = *locals.get(index as usize).unwrap();
            stack.expect(ty)?;
            stack.push(ty);
        }


        GlobalGet (index) => {
            let global = *module.globals.get(index as usize).unwrap();
            stack.push(global.ty.ty);
        }

        GlobalSet (index) => {
            let global = *module.globals.get(index as usize).unwrap();
            assert!(global.ty.mutt);
            stack.expect(global.ty.ty)?;
        }
    
        Load { load, align: _, offset: _ } => {
            stack.expect(ValueType::I32)?;
            stack.push(load.ty());
        }

        Store { store, align: _, offset: _ } => {
            stack.expect(store.ty())?;
            stack.expect(ValueType::I32)?;
        }


        TestOp (op) => {
            stack.expect(op.ty())?;
            stack.push(ValueType::I32);
        }

        RelOp (op) => {
            stack.expect(op.ty())?;
            stack.expect(op.ty())?;
            stack.push(ValueType::I32);
        }

        Op1 (op) => {
            stack.expect(op.ty())?;
            stack.push(op.ty());
        }

        Op2 (op) => {
            stack.expect(op.ty())?;
            stack.expect(op.ty())?;
            stack.push(op.ty());
        }

        Convert (cvt) => {
            stack.expect(cvt.from_ty())?;
            stack.push(cvt.to_ty());
        }

        Reinterpret (reterp) => {
            stack.expect(reterp.from_ty())?;
            stack.push(reterp.to_ty());
        }


        MemorySize => {
            stack.push(ValueType::I32);
        }

        MemoryGrow => {
            stack.expect(ValueType::I32)?;
            stack.push(ValueType::I32);
        }

        MemoryCopy => {
            stack.expect(ValueType::I32)?;
            stack.expect(ValueType::I32)?;
            stack.expect(ValueType::I32)?;
        }

        MemoryFill => {
            stack.expect(ValueType::I32)?;
            stack.expect(ValueType::I32)?;
            stack.expect(ValueType::I32)?;
        }
    }

    return Ok(());
}

