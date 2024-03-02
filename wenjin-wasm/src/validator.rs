use sti::manual_vec::ManualVec;

use crate::{ValueType, FuncType, BlockType, TypeIdx, FuncIdx, TableIdx, MemoryIdx, GlobalIdx, Module};
use crate::operator::OperatorVisitor;


pub enum ValidatorError {
}

pub const DEFAULT_STACK_LIMIT: usize = 128;
pub const DEFAULT_FRAME_LIMIT: usize = 1024;

pub struct Validator<'a> {
    pub stack_limit: usize,
    pub frame_limit: usize,

    module: &'a Module<'a>,

    stack: ManualVec<ValueType>,
    frames: ManualVec<ControlFrame>,
    frame: ControlFrame,

    func_ty: TypeIdx,
}

#[derive(Clone, Copy)]
struct ControlFrame {
    kind:        ControlFrameKind,
    ty:          BlockType,
    height:      usize,
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
            func_ty: u32::MAX,
        }
    }

    fn unreachable(&mut self) {
        self.frame.unreachable = true;
        self.stack.truncate(self.frame.height);
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
        todo!()
    }

    fn visit_loop(&mut self, yt: BlockType) -> Self::Output {
        todo!()
    }

    fn visit_if(&mut self, ty: BlockType) -> Self::Output {
        todo!()
    }

    fn visit_else(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_end(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_br(&mut self, label: u32) -> Self::Output {
        todo!()
    }

    fn visit_br_if(&mut self, label: u32) -> Self::Output {
        todo!()
    }

    fn visit_br_table(&mut self, table: ()) -> Self::Output {
        todo!()
    }

    fn visit_return(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_call(&mut self, func: FuncIdx) -> Self::Output {
        todo!()
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
        todo!()
    }

    fn visit_local_set(&mut self, idx: u32) -> Self::Output {
        todo!()
    }

    fn visit_local_tee(&mut self, idx: u32) -> Self::Output {
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


