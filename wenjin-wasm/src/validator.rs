use sti::traits::{CopyIt, UnwrapDebug};
use sti::manual_vec::ManualVec;

use crate::{ValueType, BlockType, TypeIdx, FuncIdx, TableIdx, MemoryIdx, GlobalIdx, Module, TableType, FuncType, RefType, GlobalType, MemoryType};
use crate::operator::OperatorVisitor;


#[derive(Debug)]
pub enum ValidatorError {
}


pub const DEFAULT_STACK_LIMIT: u32 =  128; // exprs aren't that deep.
pub const DEFAULT_FRAME_LIMIT: u32 = 1024; // jump tables can be pretty large.

pub struct Validator<'a> {
    pub stack_limit: u32,
    pub frame_limit: u32,

    module: &'a Module<'a>,

    locals: ManualVec<ValueType>,
    stack: ManualVec<ValueType>,
    max_stack: u32,
    frames: ManualVec<ControlFrame>,
}

#[derive(Clone, Copy, Debug)]
struct ControlFrame {
    kind:        ControlFrameKind,
    ty:          BlockType,
    height:      u32,
    unreachable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControlFrameKind {
    Block,
    Loop,
    If,
    Else,
}


impl<'a> Validator<'a> {
    pub fn new(module: &'a Module<'a>) -> Self {
        Self {
            stack_limit: DEFAULT_STACK_LIMIT,
            frame_limit: DEFAULT_FRAME_LIMIT,
            module,
            locals: ManualVec::new(),
            stack: ManualVec::new(),
            max_stack: 0,
            frames: ManualVec::new(),
        }
    }

    #[inline(always)]
    pub fn max_stack(&self) -> u32 {
        self.max_stack
    }

    #[inline(always)]
    pub fn num_locals(&self) -> u32 {
        self.locals.len() as u32
    }

    #[inline(always)]
    pub fn stack(&self) -> &[ValueType] {
        &self.stack
    }

    #[inline(always)]
    pub fn num_frames(&self) -> u32 {
        self.frames.len() as u32
    }


    pub fn begin_func(&mut self, ty: TypeIdx, locals: &[ValueType]) -> Result<(), ValidatorError> {
        self.locals.truncate(0);
        let params = self.ty(ty)?.params;
        self.locals.reserve(params.len() + locals.len()).map_err(|_| todo!())?;
        for param in params.copy_it() {
            self.locals.push(param).unwrap_debug();
        }
        for local in locals.copy_it() {
            self.locals.push(local).unwrap_debug();
        }

        self.stack.truncate(0);
        self.max_stack = 0;

        self.frames.truncate(0);
        self.frames.push_or_alloc(ControlFrame {
            kind: ControlFrameKind::Block,
            ty: BlockType::Func(ty),
            height: 0,
            unreachable: false,
        }).map_err(|_| todo!())?;

        return Ok(());
    }

    fn is_unreachable(&self) -> bool {
        self.frames.rev(0).unreachable
    }

    fn unreachable(&mut self) {
        let frame = self.frames.rev_mut(0);
        frame.unreachable = true;
        self.stack.truncate(frame.height as usize);
    }


    fn push(&mut self, ty: ValueType) -> Result<(), ValidatorError> {
        if !self.is_unreachable() {
            if self.stack.len() >= self.stack_limit as usize {
                todo!()
            }

            self.stack.push_or_alloc(ty).map_err(|_| todo!())?;
            self.max_stack = self.max_stack.max(self.stack.len() as u32);
        }
        return Ok(());
    }

    fn push_n(&mut self, tys: &[ValueType]) -> Result<(), ValidatorError> {
        for ty in tys {
            self.push(*ty)?;
        }
        return Ok(());
    }

    fn pop(&mut self) -> Result<ValueType, ValidatorError> {
        debug_assert!(!self.is_unreachable());

        if self.stack.len() <= self.frames.rev(0).height as usize {
            todo!()
        }

        return Ok(self.stack.pop().unwrap());
    }

    fn expect(&mut self, expected_ty: ValueType) -> Result<(), ValidatorError> {
        if !self.is_unreachable() {
            if self.stack.len() <= self.frames.rev(0).height as usize {
                todo!()
            }

            let ty = self.stack.pop().unwrap();
            if ty != expected_ty {
                todo!()
            }
        }
        return Ok(());
    }

    fn expect_n(&mut self, expected_tys: &[ValueType]) -> Result<(), ValidatorError> {
        for ty in expected_tys.iter().rev() {
            self.expect(*ty)?;
        }
        Ok(())
    }


    // pushes the block begin types.
    fn push_frame(&mut self, kind: ControlFrameKind, ty: BlockType) -> Result<(), ValidatorError> {
        if self.frames.len() >= self.frame_limit as usize {
            todo!();
        }

        let height = self.stack.len() as u32;

        self.push_n(self.block_begin_types(ty))?;

        self.frames.push_or_alloc(ControlFrame {
            kind,
            ty,
            height,
            unreachable: self.is_unreachable(),
        }).map_err(|_| todo!())?;

        return Ok(());
    }

    // expects the block end types.
    fn pop_frame(&mut self) -> Result<ControlFrame, ValidatorError> {
        let Some(frame) = self.frames.last().copied() else {
            todo!()
        };

        self.expect_n(self.block_end_types(frame.ty))?;
        if self.stack.len() != frame.height as usize {
            todo!()
        }

        let frame = self.frames.pop().unwrap();
        return Ok(frame);
    }


    fn label(&self, idx: u32) -> Result<ControlFrame, ValidatorError> {
        // @todo: get_rev
        let idx = idx as usize;
        if idx >= self.frames.len() {
            todo!()
        }
        Ok(*self.frames.rev(idx))
    }

    fn ty(&self, idx: TypeIdx) -> Result<FuncType<'a>, ValidatorError> {
        self.module.types.get(idx as usize).copied()
        .ok_or_else(|| todo!())
    }

    fn local(&self, idx: u32) -> Result<ValueType, ValidatorError> {
        self.locals.get(idx as usize).copied()
        .ok_or_else(|| todo!())
    }

    // @todo: move func logic here.

    fn table(&self, idx: TableIdx) -> Result<TableType, ValidatorError> {
        // @todo: imports.
        self.module.tables.get(idx as usize).copied()
        .ok_or_else(|| todo!())
    }

    fn memory(&self, idx: TableIdx) -> Result<MemoryType, ValidatorError> {
        // @todo: imports.
        self.module.memories.get(idx as usize).copied()
        .ok_or_else(|| todo!())
    }

    fn global(&self, idx: u32) -> Result<GlobalType, ValidatorError> {
        // @todo: imports.
        Ok(self.module.globals.get(idx as usize).copied()
        .ok_or_else(|| todo!())?
        .ty)
    }


    #[inline]
    fn load(&mut self, ty: ValueType, _align: u32, _offset: u32) -> Result<(), ValidatorError> {
        // @todo: validate alignment.
        self.expect(ValueType::I32)?;
        self.push(ty)
    }

    #[inline]
    fn store(&mut self, ty: ValueType, _align: u32, _offset: u32) -> Result<(), ValidatorError> {
        // @todo: validate alignment.
        self.expect(ty)?;
        self.expect(ValueType::I32)
    }

    #[inline]
    fn test_op(&mut self, ty: ValueType) -> Result<(), ValidatorError> {
        self.expect(ty)?;
        self.push(ValueType::I32)
    }

    #[inline]
    fn rel_op(&mut self, ty: ValueType) -> Result<(), ValidatorError> {
        self.expect(ty)?;
        self.expect(ty)?;
        self.push(ValueType::I32)
    }

    #[inline]
    fn un_op(&mut self, ty: ValueType) -> Result<(), ValidatorError> {
        self.expect(ty)?;
        self.push(ty)
    }

    #[inline]
    fn bin_op(&mut self, ty: ValueType) -> Result<(), ValidatorError> {
        self.expect(ty)?;
        self.expect(ty)?;
        self.push(ty)
    }

    #[inline]
    fn cvt_op(&mut self, dst_ty: ValueType, src_ty: ValueType) -> Result<(), ValidatorError> {
        self.expect(src_ty)?;
        self.push(dst_ty)
    }


    #[inline]
    fn block_begin_types(&self, ty: BlockType) -> &'a [ValueType] {
        ty.begin_types(self.module)
    }

    #[inline]
    fn block_end_types(&self, ty: BlockType) -> &'a [ValueType] {
        ty.end_types(self.module)
    }

    #[inline]
    fn frame_br_types(&self, frame: &ControlFrame) -> &'a [ValueType] {
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
}

impl<'a> OperatorVisitor for Validator<'a> {
    type Output = Result<(), ValidatorError>;

    fn visit_unreachable(&mut self) -> Self::Output {
        self.unreachable();
        Ok(())
    }

    fn visit_nop(&mut self) -> Self::Output {
        Ok(())
    }

    fn visit_block(&mut self, ty: BlockType) -> Self::Output {
        self.expect_n(self.block_begin_types(ty))?;
        self.push_frame(ControlFrameKind::Block, ty)
    }

    fn visit_loop(&mut self, ty: BlockType) -> Self::Output {
        self.expect_n(self.block_begin_types(ty))?;
        self.push_frame(ControlFrameKind::Loop, ty)
    }

    fn visit_if(&mut self, ty: BlockType) -> Self::Output {
        self.expect(ValueType::I32)?;
        self.expect_n(self.block_begin_types(ty))?;
        self.push_frame(ControlFrameKind::If, ty)
    }

    fn visit_else(&mut self) -> Self::Output {
        let frame = self.pop_frame()?;
        if frame.kind != ControlFrameKind::If {
            todo!()
        }
        self.push_frame(ControlFrameKind::Else, frame.ty)
    }

    fn visit_end(&mut self) -> Self::Output {
        if self.frames.len() > 1 {
            let frame = self.pop_frame()?;
            self.push_n(self.block_end_types(frame.ty))
        }
        else {
            // implicit return.
            self.expect_n(self.block_end_types(self.frames[0].ty))?;
            if self.stack.len() != 0 {
                todo!()
            }
            self.unreachable();
            return Ok(());
        }
    }

    fn visit_br(&mut self, label: u32) -> Self::Output {
        let frame = self.label(label)?;
        self.expect_n(self.frame_br_types(&frame))?;
        self.unreachable();
        return Ok(());
    }

    fn visit_br_if(&mut self, label: u32) -> Self::Output {
        let frame = self.label(label)?;
        self.expect(ValueType::I32)?;
        self.expect_n(self.frame_br_types(&frame))?;
        self.push_n(self.frame_br_types(&frame))
    }

    fn visit_br_table(&mut self, default: u32) -> Self::Output {
        // @temp.
        let frame = self.label(default)?;
        self.expect(ValueType::I32)?;
        self.expect_n(self.frame_br_types(&frame))?;
        self.push_n(self.frame_br_types(&frame))?;
        self.unreachable();
        return Ok(());
    }

    fn visit_return(&mut self) -> Self::Output {
        let frame = self.frames[0];
        self.expect_n(self.block_end_types(frame.ty))?;
        self.unreachable();
        return Ok(());
    }

    fn visit_call(&mut self, func: FuncIdx) -> Self::Output {
        let func = func as usize;

        let imports = self.module.imports.funcs;

        let type_idx = match imports.get(func).copied() {
            Some(it) => it,
            None => 
                self.module.funcs.get(func - imports.len()).copied()
                .ok_or_else(|| todo!())?,
        };

        let ty =
            self.module.types.get(type_idx as usize).copied()
            .ok_or_else(|| todo!())?;

        self.expect_n(ty.params)?;
        self.push_n(ty.rets)
    }

    fn visit_call_indirect(&mut self, ty: TypeIdx, table: TableIdx) -> Self::Output {
        let table = self.table(table)?;
        if table.ty != RefType::FuncRef {
            todo!()
        }

        let ty = self.ty(ty)?;
        self.expect(ValueType::I32)?;
        self.expect_n(ty.params)?;
        self.push_n(ty.rets)
    }

    fn visit_drop(&mut self) -> Self::Output {
        if !self.is_unreachable() {
            self.pop()?;
        }
        return Ok(());
    }

    fn visit_select(&mut self) -> Self::Output {
        self.expect(ValueType::I32)?;

        if !self.is_unreachable() {
            let t1 = self.pop()?;
            let t2 = self.pop()?;

            if t1.is_ref() || t2.is_ref() {
                todo!()
            }

            if t1 != t2 {
                todo!()
            }

            self.push(t1)?;
        }

        return Ok(());
    }

    fn visit_typed_select(&mut self, ty: ValueType) -> Self::Output {
        let _ = ty;
        todo!()
    }

    fn visit_local_get(&mut self, idx: u32) -> Self::Output {
        let ty = self.local(idx)?;
        self.push(ty)
    }

    fn visit_local_set(&mut self, idx: u32) -> Self::Output {
        let ty = self.local(idx)?;
        self.expect(ty)
    }

    fn visit_local_tee(&mut self, idx: u32) -> Self::Output {
        let ty = self.local(idx)?;
        self.expect(ty)?;
        self.push(ty)
    }

    fn visit_global_get(&mut self, idx: GlobalIdx) -> Self::Output {
        let g = self.global(idx)?;
        self.push(g.ty)
    }

    fn visit_global_set(&mut self, idx: GlobalIdx) -> Self::Output {
        let g = self.global(idx)?;
        if !g.mutt {
            todo!()
        }
        self.expect(g.ty)
    }

    fn visit_table_get(&mut self, idx: TableIdx) -> Self::Output {
        let _ = idx;
        todo!()
    }

    fn visit_table_set(&mut self, idx: TableIdx) -> Self::Output {
        let _ = idx;
        todo!()
    }

    fn visit_i32_load(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::I32, align, offset)
    }

    fn visit_i64_load(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::I64, align, offset)
    }

    fn visit_f32_load(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::F32, align, offset)
    }

    fn visit_f64_load(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::F64, align, offset)
    }

    fn visit_i32_load8_s(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::I32, align, offset)
    }

    fn visit_i32_load8_u(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::I32, align, offset)
    }

    fn visit_i32_load16_s(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::I32, align, offset)
    }

    fn visit_i32_load16_u(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::I32, align, offset)
    }

    fn visit_i64_load8_s(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::I64, align, offset)
    }

    fn visit_i64_load8_u(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::I64, align, offset)
    }

    fn visit_i64_load16_s(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::I64, align, offset)
    }

    fn visit_i64_load16_u(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::I64, align, offset)
    }

    fn visit_i64_load32_s(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::I64, align, offset)
    }

    fn visit_i64_load32_u(&mut self, align:u32, offset:u32) -> Self::Output {
        self.load(ValueType::I64, align, offset)
    }

    fn visit_i32_store(&mut self, align:u32, offset:u32) -> Self::Output {
        self.store(ValueType::I32, align, offset)
    }

    fn visit_i64_store(&mut self, align:u32, offset:u32) -> Self::Output {
        self.store(ValueType::I64, align, offset)
    }

    fn visit_f32_store(&mut self, align:u32, offset:u32) -> Self::Output {
        self.store(ValueType::F32, align, offset)
    }

    fn visit_f64_store(&mut self, align:u32, offset:u32) -> Self::Output {
        self.store(ValueType::F64, align, offset)
    }

    fn visit_i32_store8(&mut self, align:u32, offset:u32) -> Self::Output {
        self.store(ValueType::I32, align, offset)
    }

    fn visit_i32_store16(&mut self, align:u32, offset:u32) -> Self::Output {
        self.store(ValueType::I32, align, offset)
    }

    fn visit_i64_store8(&mut self, align:u32, offset:u32) -> Self::Output {
        self.store(ValueType::I64, align, offset)
    }

    fn visit_i64_store16(&mut self, align:u32, offset:u32) -> Self::Output {
        self.store(ValueType::I64, align, offset)
    }

    fn visit_i64_store32(&mut self, align:u32, offset:u32) -> Self::Output {
        self.store(ValueType::I64, align, offset)
    }

    fn visit_i32_const(&mut self, _value: i32) -> Self::Output {
        self.push(ValueType::I32)
    }

    fn visit_i64_const(&mut self, _value: i64) -> Self::Output {
        self.push(ValueType::I64)
    }

    fn visit_f32_const(&mut self, _value: f32) -> Self::Output {
        self.push(ValueType::F32)
    }

    fn visit_f64_const(&mut self, _value: f64) -> Self::Output {
        self.push(ValueType::F64)
    }

    fn visit_i32_eqz(&mut self) -> Self::Output {
        self.test_op(ValueType::I32)
    }

    fn visit_i32_eq(&mut self) -> Self::Output {
        self.rel_op(ValueType::I32)
    }

    fn visit_i32_ne(&mut self) -> Self::Output {
        self.rel_op(ValueType::I32)
    }

    fn visit_i32_lt_s(&mut self) -> Self::Output {
        self.rel_op(ValueType::I32)
    }

    fn visit_i32_lt_u(&mut self) -> Self::Output {
        self.rel_op(ValueType::I32)
    }

    fn visit_i32_gt_s(&mut self) -> Self::Output {
        self.rel_op(ValueType::I32)
    }

    fn visit_i32_gt_u(&mut self) -> Self::Output {
        self.rel_op(ValueType::I32)
    }

    fn visit_i32_le_s(&mut self) -> Self::Output {
        self.rel_op(ValueType::I32)
    }

    fn visit_i32_le_u(&mut self) -> Self::Output {
        self.rel_op(ValueType::I32)
    }

    fn visit_i32_ge_s(&mut self) -> Self::Output {
        self.rel_op(ValueType::I32)
    }

    fn visit_i32_ge_u(&mut self) -> Self::Output {
        self.rel_op(ValueType::I32)
    }

    fn visit_i64_eqz(&mut self) -> Self::Output {
        self.test_op(ValueType::I64)
    }

    fn visit_i64_eq(&mut self) -> Self::Output {
        self.rel_op(ValueType::I64)
    }

    fn visit_i64_ne(&mut self) -> Self::Output {
        self.rel_op(ValueType::I64)
    }

    fn visit_i64_lt_s(&mut self) -> Self::Output {
        self.rel_op(ValueType::I64)
    }

    fn visit_i64_lt_u(&mut self) -> Self::Output {
        self.rel_op(ValueType::I64)
    }

    fn visit_i64_gt_s(&mut self) -> Self::Output {
        self.rel_op(ValueType::I64)
    }

    fn visit_i64_gt_u(&mut self) -> Self::Output {
        self.rel_op(ValueType::I64)
    }

    fn visit_i64_le_s(&mut self) -> Self::Output {
        self.rel_op(ValueType::I64)
    }

    fn visit_i64_le_u(&mut self) -> Self::Output {
        self.rel_op(ValueType::I64)
    }

    fn visit_i64_ge_s(&mut self) -> Self::Output {
        self.rel_op(ValueType::I64)
    }

    fn visit_i64_ge_u(&mut self) -> Self::Output {
        self.rel_op(ValueType::I64)
    }

    fn visit_f32_eq(&mut self) -> Self::Output {
        self.rel_op(ValueType::F32)
    }

    fn visit_f32_ne(&mut self) -> Self::Output {
        self.rel_op(ValueType::F32)
    }

    fn visit_f32_lt(&mut self) -> Self::Output {
        self.rel_op(ValueType::F32)
    }

    fn visit_f32_gt(&mut self) -> Self::Output {
        self.rel_op(ValueType::F32)
    }

    fn visit_f32_le(&mut self) -> Self::Output {
        self.rel_op(ValueType::F32)
    }

    fn visit_f32_ge(&mut self) -> Self::Output {
        self.rel_op(ValueType::F32)
    }

    fn visit_f64_eq(&mut self) -> Self::Output {
        self.rel_op(ValueType::F64)
    }

    fn visit_f64_ne(&mut self) -> Self::Output {
        self.rel_op(ValueType::F64)
    }

    fn visit_f64_lt(&mut self) -> Self::Output {
        self.rel_op(ValueType::F64)
    }

    fn visit_f64_gt(&mut self) -> Self::Output {
        self.rel_op(ValueType::F64)
    }

    fn visit_f64_le(&mut self) -> Self::Output {
        self.rel_op(ValueType::F64)
    }

    fn visit_f64_ge(&mut self) -> Self::Output {
        self.rel_op(ValueType::F64)
    }

    fn visit_i32_clz(&mut self) -> Self::Output {
        self.un_op(ValueType::I32)
    }

    fn visit_i32_ctz(&mut self) -> Self::Output {
        self.un_op(ValueType::I32)
    }

    fn visit_i32_popcnt(&mut self) -> Self::Output {
        self.un_op(ValueType::I32)
    }

    fn visit_i32_add(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_sub(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_mul(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_div_s(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_div_u(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_rem_s(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_rem_u(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_and(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_or(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_xor(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_shl(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_shr_s(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_shr_u(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_rotl(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i32_rotr(&mut self) -> Self::Output {
        self.bin_op(ValueType::I32)
    }

    fn visit_i64_clz(&mut self) -> Self::Output {
        self.un_op(ValueType::I64)
    }

    fn visit_i64_ctz(&mut self) -> Self::Output {
        self.un_op(ValueType::I64)
    }

    fn visit_i64_popcnt(&mut self) -> Self::Output {
        self.un_op(ValueType::I64)
    }

    fn visit_i64_add(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_sub(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_mul(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_div_s(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_div_u(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_rem_s(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_rem_u(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_and(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_or(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_xor(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_shl(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_shr_s(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_shr_u(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_rotl(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_i64_rotr(&mut self) -> Self::Output {
        self.bin_op(ValueType::I64)
    }

    fn visit_f32_abs(&mut self) -> Self::Output {
        self.un_op(ValueType::F32)
    }

    fn visit_f32_neg(&mut self) -> Self::Output {
        self.un_op(ValueType::F32)
    }

    fn visit_f32_ceil(&mut self) -> Self::Output {
        self.un_op(ValueType::F32)
    }

    fn visit_f32_floor(&mut self) -> Self::Output {
        self.un_op(ValueType::F32)
    }

    fn visit_f32_trunc(&mut self) -> Self::Output {
        self.un_op(ValueType::F32)
    }

    fn visit_f32_nearest(&mut self) -> Self::Output {
        self.un_op(ValueType::F32)
    }

    fn visit_f32_sqrt(&mut self) -> Self::Output {
        self.un_op(ValueType::F32)
    }

    fn visit_f32_add(&mut self) -> Self::Output {
        self.bin_op(ValueType::F32)
    }

    fn visit_f32_sub(&mut self) -> Self::Output {
        self.bin_op(ValueType::F32)
    }

    fn visit_f32_mul(&mut self) -> Self::Output {
        self.bin_op(ValueType::F32)
    }

    fn visit_f32_div(&mut self) -> Self::Output {
        self.bin_op(ValueType::F32)
    }

    fn visit_f32_min(&mut self) -> Self::Output {
        self.bin_op(ValueType::F32)
    }

    fn visit_f32_max(&mut self) -> Self::Output {
        self.bin_op(ValueType::F32)
    }

    fn visit_f32_copysign(&mut self) -> Self::Output {
        self.bin_op(ValueType::F32)
    }

    fn visit_f64_abs(&mut self) -> Self::Output {
        self.un_op(ValueType::F64)
    }

    fn visit_f64_neg(&mut self) -> Self::Output {
        self.un_op(ValueType::F64)
    }

    fn visit_f64_ceil(&mut self) -> Self::Output {
        self.un_op(ValueType::F64)
    }

    fn visit_f64_floor(&mut self) -> Self::Output {
        self.un_op(ValueType::F64)
    }

    fn visit_f64_trunc(&mut self) -> Self::Output {
        self.un_op(ValueType::F64)
    }

    fn visit_f64_nearest(&mut self) -> Self::Output {
        self.un_op(ValueType::F64)
    }

    fn visit_f64_sqrt(&mut self) -> Self::Output {
        self.un_op(ValueType::F64)
    }

    fn visit_f64_add(&mut self) -> Self::Output {
        self.bin_op(ValueType::F64)
    }

    fn visit_f64_sub(&mut self) -> Self::Output {
        self.bin_op(ValueType::F64)
    }

    fn visit_f64_mul(&mut self) -> Self::Output {
        self.bin_op(ValueType::F64)
    }

    fn visit_f64_div(&mut self) -> Self::Output {
        self.bin_op(ValueType::F64)
    }

    fn visit_f64_min(&mut self) -> Self::Output {
        self.bin_op(ValueType::F64)
    }

    fn visit_f64_max(&mut self) -> Self::Output {
        self.bin_op(ValueType::F64)
    }

    fn visit_f64_copysign(&mut self) -> Self::Output {
        self.bin_op(ValueType::F64)
    }

    fn visit_i32_wrap_i64(&mut self) -> Self::Output {
        self.cvt_op(ValueType::I32, ValueType::I64)
    }

    fn visit_i32_trunc_f32_s(&mut self) -> Self::Output {
        self.cvt_op(ValueType::I32, ValueType::F32)
    }

    fn visit_i32_trunc_f32_u(&mut self) -> Self::Output {
        self.cvt_op(ValueType::I32, ValueType::F32)
    }

    fn visit_i32_trunc_f64_s(&mut self) -> Self::Output {
        self.cvt_op(ValueType::I32, ValueType::F64)
    }

    fn visit_i32_trunc_f64_u(&mut self) -> Self::Output {
        self.cvt_op(ValueType::I32, ValueType::F64)
    }

    fn visit_i64_extend_i32_s(&mut self) -> Self::Output {
        self.cvt_op(ValueType::I64, ValueType::I32)
    }

    fn visit_i64_extend_i32_u(&mut self) -> Self::Output {
        self.cvt_op(ValueType::I64, ValueType::I32)
    }

    fn visit_i64_trunc_f32_s(&mut self) -> Self::Output {
        self.cvt_op(ValueType::I64, ValueType::F32)
    }

    fn visit_i64_trunc_f32_u(&mut self) -> Self::Output {
        self.cvt_op(ValueType::I64, ValueType::F32)
    }

    fn visit_i64_trunc_f64_s(&mut self) -> Self::Output {
        self.cvt_op(ValueType::I64, ValueType::F64)
    }

    fn visit_i64_trunc_f64_u(&mut self) -> Self::Output {
        self.cvt_op(ValueType::I64, ValueType::F64)
    }

    fn visit_f32_convert_i32_s(&mut self) -> Self::Output {
        self.cvt_op(ValueType::F32, ValueType::I32)
    }

    fn visit_f32_convert_i32_u(&mut self) -> Self::Output {
        self.cvt_op(ValueType::F32, ValueType::I32)
    }

    fn visit_f32_convert_i64_s(&mut self) -> Self::Output {
        self.cvt_op(ValueType::F32, ValueType::I64)
    }

    fn visit_f32_convert_i64_u(&mut self) -> Self::Output {
        self.cvt_op(ValueType::F32, ValueType::I64)
    }

    fn visit_f32_demote_f64(&mut self) -> Self::Output {
        self.cvt_op(ValueType::F32, ValueType::F64)
    }

    fn visit_f64_convert_i32_s(&mut self) -> Self::Output {
        self.cvt_op(ValueType::F64, ValueType::I32)
    }

    fn visit_f64_convert_i32_u(&mut self) -> Self::Output {
        self.cvt_op(ValueType::F64, ValueType::I32)
    }

    fn visit_f64_convert_i64_s(&mut self) -> Self::Output {
        self.cvt_op(ValueType::F64, ValueType::I64)
    }

    fn visit_f64_convert_i64_u(&mut self) -> Self::Output {
        self.cvt_op(ValueType::F64, ValueType::I64)
    }

    fn visit_f64_promote_f32(&mut self) -> Self::Output {
        self.cvt_op(ValueType::F64, ValueType::F32)
    }

    fn visit_i32_reinterpret_f32(&mut self) -> Self::Output {
        self.cvt_op(ValueType::I32, ValueType::F32)
    }

    fn visit_i64_reinterpret_f64(&mut self) -> Self::Output {
        self.cvt_op(ValueType::I64, ValueType::F64)
    }

    fn visit_f32_reinterpret_i32(&mut self) -> Self::Output {
        self.cvt_op(ValueType::F32, ValueType::I32)
    }

    fn visit_f64_reinterpret_i64(&mut self) -> Self::Output {
        self.cvt_op(ValueType::F64, ValueType::I64)
    }

    fn visit_i32_extend8_s(&mut self) -> Self::Output {
        self.un_op(ValueType::I32)
    }

    fn visit_i32_extend16_s(&mut self) -> Self::Output {
        self.un_op(ValueType::I32)
    }

    fn visit_i64_extend8_s(&mut self) -> Self::Output {
        self.un_op(ValueType::I64)
    }

    fn visit_i64_extend16_s(&mut self) -> Self::Output {
        self.un_op(ValueType::I64)
    }

    fn visit_i64_extend32_s(&mut self) -> Self::Output {
        self.un_op(ValueType::I64)
    }

    fn visit_ref_null(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_ref_is_null(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_ref_func(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_memory_size(&mut self, mem: MemoryIdx) -> Self::Output {
        self.memory(mem)?;
        self.push(ValueType::I32)
    }

    fn visit_memory_grow(&mut self, mem: MemoryIdx) -> Self::Output {
        self.memory(mem)?;
        self.expect(ValueType::I32)?;
        self.push(ValueType::I32)
    }

    fn visit_memory_copy(&mut self, dst: MemoryIdx, src: MemoryIdx) -> Self::Output {
        self.memory(dst)?;
        self.memory(src)?;
        self.expect(ValueType::I32)?;
        self.expect(ValueType::I32)?;
        self.expect(ValueType::I32)
    }

    fn visit_memory_fill(&mut self, mem: MemoryIdx) -> Self::Output {
        self.memory(mem)?;
        self.expect(ValueType::I32)?;
        self.expect(ValueType::I32)?;
        self.expect(ValueType::I32)
    }
}


