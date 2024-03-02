use sti::manual_vec::ManualVec;

use crate::{ValueType, BlockType, TypeIdx, FuncIdx, TableIdx, MemoryIdx, GlobalIdx, Module};
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

    stack: ManualVec<ValueType>,
    frames: ManualVec<ControlFrame>,
    frame: ControlFrame,
}

#[derive(Clone, Copy)]
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
            stack: ManualVec::new(),
            frames: ManualVec::new(),
            frame: ControlFrame {
                kind: ControlFrameKind::Block,
                ty: BlockType::Unit,
                height: 0,
                unreachable: false,
            },
        }
    }

    pub fn begin_func(&mut self, ty: TypeIdx, locals: &[ValueType]) {
        self.stack.truncate(0);
        self.frames.truncate(0);
        self.frame = ControlFrame {
            kind: ControlFrameKind::Block,
            ty: BlockType::Func(ty),
            height: 0,
            unreachable: false,
        };
    }

    fn unreachable(&mut self) {
        self.frame.unreachable = true;
        self.stack.truncate(self.frame.height as usize);
    }


    fn push(&mut self, ty: ValueType) -> Result<(), ValidatorError> {
        if !self.frame.unreachable {
            if self.stack.len() >= self.stack_limit as usize {
                todo!()
            }

            self.stack.push_or_alloc(ty).map_err(|_| todo!())?;
        }
        return Ok(());
    }

    fn push_n(&mut self, tys: &[ValueType]) -> Result<(), ValidatorError> {
        for ty in tys {
            self.push(*ty)?;
        }
        return Ok(());
    }

    fn expect(&mut self, expected_ty: ValueType) -> Result<(), ValidatorError> {
        if !self.frame.unreachable {
            if self.stack.len() <= self.frame.height as usize {
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

        self.push_n(self.block_begin_types(ty))?;

        self.frames.push_or_alloc(self.frame).map_err(|_| todo!())?;
        self.frame = ControlFrame {
            kind,
            ty,
            height: self.stack.len() as u32,
            unreachable: false,
        };

        return Ok(());
    }

    // expects the block end types.
    fn pop_frame(&mut self) -> Result<ControlFrame, ValidatorError> {
        if self.frames.len() == 0 {
            todo!()
        }

        let end_types = self.block_end_types(self.frame.ty);
        let height    = self.frame.height;

        self.expect_n(&end_types)?;
        if self.stack.len() != height as usize {
            todo!()
        }

        let result = self.frame;
        self.frame = self.frames.pop().unwrap();
        return Ok(result);
    }


    fn label(&self, idx: u32) -> Result<ControlFrame, ValidatorError> {
        let idx = idx as usize;
        if idx > self.frames.len() {
            todo!()
        }

        if idx == 0 {
            return Ok(self.frame);
        }
        else {
            return Ok(*self.frames.rev(idx - 1));
        }
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
        if self.frames.len() > 0 {
            let frame = self.pop_frame()?;
            self.push_n(self.block_end_types(frame.ty))
        }
        else {
            // implicit return.
            self.expect_n(self.block_end_types(self.frame.ty))?;
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
        self.expect(ValueType::I32)?;
        let frame = self.label(label)?;
        self.expect_n(self.frame_br_types(&frame))?;
        self.push_n(self.frame_br_types(&frame))
    }

    fn visit_br_table(&mut self, table: ()) -> Self::Output {
        todo!()
    }

    fn visit_return(&mut self) -> Self::Output {
        let frame = self.frames.get(0).unwrap_or(&self.frame);
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
        todo!()
    }

    fn visit_drop(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_select(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_select_ex(&mut self, ty: ValueType) -> Self::Output {
        todo!()
    }

    fn visit_local_get(&mut self, idx: u32) -> Self::Output {
        /*
        let Some(ty) = self.locals.get(idx as usize).copied() else {
            todo!()
        };
        self.push(ty)
        */
        todo!()
    }

    fn visit_local_set(&mut self, idx: u32) -> Self::Output {
        /*
        let Some(ty) = self.locals.get(idx as usize).copied() else {
            todo!()
        };
        self.expect(ty)
        */
        todo!()
    }

    fn visit_local_tee(&mut self, idx: u32) -> Self::Output {
        /*
        let Some(ty) = self.locals.get(idx as usize).copied() else {
            todo!()
        };
        self.expect(ty)?;
        self.push(ty)
        */
        todo!()
    }

    fn visit_global_get(&mut self, idx: GlobalIdx) -> Self::Output {
        todo!()
    }

    fn visit_global_set(&mut self, idx: GlobalIdx) -> Self::Output {
        todo!()
    }

    fn visit_table_get(&mut self, idx: TableIdx) -> Self::Output {
        todo!()
    }

    fn visit_table_set(&mut self, idx: TableIdx) -> Self::Output {
        todo!()
    }

    fn visit_i32_load(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i64_load(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_f32_load(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_f64_load(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_load8_s(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_load8_u(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_load16_s(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_load16_u(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i64_load8_s(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i64_load8_u(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i64_load16_s(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i64_load16_u(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i64_load32_s(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i64_load32_u(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_store(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i64_store(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_f32_store(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_f64_store(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_store8(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_store16(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i64_store8(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i64_store16(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i64_store32(&mut self, align:u32, offset:u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_const(&mut self, value:i32) -> Self::Output {
        todo!()
    }

    fn visit_i64_const(&mut self, value:i64) -> Self::Output {
        todo!()
    }

    fn visit_f32_const(&mut self, value:f32) -> Self::Output {
        todo!()
    }

    fn visit_f64_const(&mut self, value:f64) -> Self::Output {
        todo!()
    }

    fn visit_i32_eqz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_lt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_lt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_gt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_gt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_le_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_le_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_ge_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_ge_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_eqz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_lt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_lt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_gt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_gt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_le_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_le_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_ge_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_ge_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_lt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_gt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_le(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_ge(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_lt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_gt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_le(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_ge(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_clz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_ctz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_popcnt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_div_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_div_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_rem_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_rem_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_and(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_or(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_xor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_shl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_shr_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_shr_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_rotl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_rotr(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_clz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_ctz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_popcnt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_div_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_div_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_rem_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_rem_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_and(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_or(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_xor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_shl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_shr_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_shr_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_rotl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_rotr(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_abs(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_neg(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_ceil(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_floor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_trunc(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_nearest(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_sqrt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_div(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_min(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_max(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_copysign(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_abs(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_neg(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_ceil(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_floor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_trunc(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_nearest(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_sqrt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_div(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_min(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_max(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_copysign(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_wrap_i64(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_f32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_f32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_f64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_f64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend_i32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend_i32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_f32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_f32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_f64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_f64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_convert_i32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_convert_i32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_convert_i64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_convert_i64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_demote_f64(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_convert_i32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_convert_i32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_convert_i64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_convert_i64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_promote_f32(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_reinterpret_f32(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_reinterpret_f64(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_reinterpret_i32(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_reinterpret_i64(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_extend8_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_extend16_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend8_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend16_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend32_s(&mut self) -> Self::Output {
        todo!()
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
        todo!()
    }

    fn visit_memory_grow(&mut self, mem: MemoryIdx) -> Self::Output {
        todo!()
    }

    fn visit_memory_copy(&mut self, dst: MemoryIdx, src: MemoryIdx) -> Self::Output {
        todo!()
    }

    fn visit_memory_fill(&mut self, mem: MemoryIdx) -> Self::Output {
        todo!()
    }
}


