use core::cell::UnsafeCell;

use sti::alloc::Alloc;
use sti::boks::Box;
use sti::manual_vec::ManualVec;

use wasm::{ValueType, BlockType, TypeIdx, FuncIdx, TableIdx, MemoryIdx, GlobalIdx, opcode};

/*
- instructions:
    - most instructions are mapped 1:1 for ease of debugging.
    - `if` & `loop` have an additional `br` instruction.
    - all `u32` operands are stored in 4 byte, native endian format.
    - jumps: see below.

- jumps:
    - jumps are relative i32s.
      the jump target is the address of the i32 itself
      plus the value of the i32.
    - when compiling a jump to a known label,
      the relative i32 is computed and written immediately.
    - when compiling a jump to an unknown label,
      a linked list of jumps to that label is maintained
      and patched once the target is known.
        - the head of the list is stored on the frame.
        - the head and each element of the list is either u32::MAX
          (indicating the end of the linked list)
          or the absolute offset of the previous use.
*/


pub(crate) struct Compiler {
    code: ManualVec<u8>,
    frames: ManualVec<Frame>,
    oom: bool,
}

enum Label {
    Known(u32),
    Unknown { last_use: u32 }, // u32::MAX -> None.
}

enum FrameKind {
    Block,
    If { else_use: u32 },
    Else,
    Loop,
}

struct Frame {
    kind: FrameKind,
    label: Label,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            code: ManualVec::new(),
            frames: ManualVec::new(),
            oom: false,
        }
    }

    pub fn begin_func(&mut self) {
        self.code.clear();
        self.frames.clear();
    }

    pub fn code<A: Alloc>(&self, alloc: A) -> Option<Box<UnsafeCell<[u8]>, A>> {
        if self.oom {
            return None;
        }

        let len = self.code.len();

        let ptr = sti::alloc::alloc_array::<u8, _>(&alloc, len)?;
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.code.as_ptr(),
                ptr.as_ptr(),
                len);

            let ptr = core::slice::from_raw_parts_mut(ptr.as_ptr(), len);
            let ptr = core::ptr::NonNull::from(ptr);
            let ptr = core::mem::transmute(ptr);
            return Some(Box::from_raw_parts(ptr, alloc));
        }
    }


    #[inline]
    fn push_byte(&mut self, byte: u8) {
        if 1 > i32::MAX as usize - self.code.len()
        || self.code.push_or_alloc(byte).is_err() {
            self.oom = true;
        }
    }

    #[inline]
    fn push_bytes(&mut self, bytes: &[u8]) {
        if bytes.len() > i32::MAX as usize - self.code.len()
        || self.code.extend_from_slice_or_alloc(bytes).is_err() {
            self.oom = true;
        }
    }

    fn push_frame(&mut self, frame: Frame) {
        if self.frames.push_or_alloc(frame).is_err() {
            self.oom = true;
        }
    }

    fn jump(&mut self, label: u32) {
        let frame = &mut self.frames[label as usize];
        match &mut frame.label {
            Label::Known(dst) => {
                let delta = *dst as i32 - self.code.len() as i32;
                self.push_bytes(&delta.to_ne_bytes());
            }

            Label::Unknown { last_use } => {
                let prev = *last_use;
                *last_use = self.code.len() as u32;
                self.push_bytes(&prev.to_ne_bytes());
            }
        }
    }

    fn patch_jumps(&mut self, last_use: u32, dst: u32) {
        let mut at = last_use;
        while at != u32::MAX {
            let slice = &mut self.code[at as usize .. at as usize + 4];
            let bytes = <&mut [u8; 4]>::try_from(slice).unwrap();
            let next = u32::from_ne_bytes(*bytes);

            let delta = dst as i32 - at as i32;
            *bytes = delta.to_ne_bytes();

            at = next;
        }
    }
}

impl wasm::OperatorVisitor for Compiler {
    type Output = ();

    fn visit_unreachable(&mut self) -> Self::Output {
        self.push_byte(opcode::UNREACHABLE);
    }

    fn visit_nop(&mut self) -> Self::Output {
        self.push_byte(opcode::NOP);
    }

    fn visit_block(&mut self, _ty: BlockType) -> Self::Output {
        self.push_byte(opcode::BLOCK);
        self.push_frame(Frame {
            kind: FrameKind::Block,
            label: Label::Unknown { last_use: u32::MAX },
        });
    }

    fn visit_loop(&mut self, _ty: BlockType) -> Self::Output {
        self.push_byte(opcode::LOOP);
        self.push_frame(Frame {
            kind: FrameKind::Loop,
            label: Label::Known(self.code.len() as u32),
        });
    }

    fn visit_if(&mut self, _ty: BlockType) -> Self::Output {
        self.push_byte(opcode::IF);
        let else_use = self.code.len() as u32;
        self.push_bytes(&u32::MAX.to_ne_bytes());
        self.push_frame(Frame {
            kind: FrameKind::If { else_use },
            label: Label::Unknown { last_use: u32::MAX },
        });
    }

    fn visit_else(&mut self) -> Self::Output {
        let frame = self.frames.pop().expect("invalid wasm");
        let FrameKind::If { else_use } = frame.kind else { panic!("invalid wasm") };

        let else_offset = self.code.len() as u32;
        self.patch_jumps(else_use, else_offset);

        self.push_byte(opcode::ELSE);
        self.push_frame(Frame {
            kind: FrameKind::Else,
            label: Label::Unknown { last_use: u32::MAX },
        });
    }

    fn visit_end(&mut self) -> Self::Output {
        let Some(frame) = self.frames.pop() else {
            self.push_byte(opcode::RETURN);
            return;
        };

        let offset = self.code.len() as u32;

        if let FrameKind::If { else_use } = frame.kind {
            self.patch_jumps(else_use, offset)
        }

        if let Label::Unknown { last_use } = frame.label {
            self.patch_jumps(last_use, offset)
        }

        self.push_byte(opcode::END);
    }

    fn visit_br(&mut self, label: u32) -> Self::Output {
        self.push_byte(opcode::BR);
        self.jump(label);
    }

    fn visit_br_if(&mut self, label: u32) -> Self::Output {
        self.push_byte(opcode::BR_IF);
        self.jump(label);
    }

    fn visit_br_table(&mut self, table: ()) -> Self::Output {
        self.push_byte(opcode::UNREACHABLE);
    }

    fn visit_return(&mut self) -> Self::Output {
        self.push_byte(opcode::RETURN);
    }

    fn visit_call(&mut self, func: FuncIdx) -> Self::Output {
        self.push_byte(opcode::CALL);
        self.push_bytes(&func.to_ne_bytes());
    }

    fn visit_call_indirect(&mut self, ty: TypeIdx, table: TableIdx) -> Self::Output {
        self.push_byte(opcode::CALL_INDIRECT);
        self.push_bytes(&ty.to_ne_bytes());
        self.push_bytes(&table.to_ne_bytes());
    }

    fn visit_drop(&mut self) -> Self::Output {
        self.push_byte(opcode::DROP);
    }

    fn visit_select(&mut self) -> Self::Output {
        self.push_byte(opcode::SELECT);
    }

    fn visit_typed_select(&mut self, _ty: ValueType) -> Self::Output {
        self.push_byte(opcode::TYPED_SELECT);
    }

    fn visit_local_get(&mut self, idx: u32) -> Self::Output {
        self.push_byte(opcode::LOCAL_GET);
        self.push_bytes(&idx.to_ne_bytes());
    }

    fn visit_local_set(&mut self, idx: u32) -> Self::Output {
        self.push_byte(opcode::LOCAL_SET);
        self.push_bytes(&idx.to_ne_bytes());
    }

    fn visit_local_tee(&mut self, idx: u32) -> Self::Output {
        self.push_byte(opcode::LOCAL_TEE);
        self.push_bytes(&idx.to_ne_bytes());
    }

    fn visit_global_get(&mut self, idx: GlobalIdx) -> Self::Output {
        self.push_byte(opcode::GLOBAL_GET);
        self.push_bytes(&idx.to_ne_bytes());
    }

    fn visit_global_set(&mut self, idx: GlobalIdx) -> Self::Output {
        self.push_byte(opcode::GLOBAL_SET);
        self.push_bytes(&idx.to_ne_bytes());
    }

    fn visit_table_get(&mut self, idx: TableIdx) -> Self::Output {
        self.push_byte(opcode::TABLE_GET);
        self.push_bytes(&idx.to_ne_bytes());
    }

    fn visit_table_set(&mut self, idx: TableIdx) -> Self::Output {
        self.push_byte(opcode::TABLE_SET);
        self.push_bytes(&idx.to_ne_bytes());
    }

    fn visit_i32_load(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I32_LOAD);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i64_load(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I64_LOAD);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_f32_load(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::F32_LOAD);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_f64_load(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::F64_LOAD);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i32_load8_s(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I32_LOAD8_S);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i32_load8_u(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I32_LOAD8_U);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i32_load16_s(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I32_LOAD16_S);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i32_load16_u(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I32_LOAD16_U);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i64_load8_s(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I64_LOAD8_S);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i64_load8_u(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I64_LOAD8_U);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i64_load16_s(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I64_LOAD16_S);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i64_load16_u(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I64_LOAD16_U);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i64_load32_s(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I64_LOAD32_S);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i64_load32_u(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I64_LOAD32_U);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i32_store(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I32_STORE);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i64_store(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I64_STORE);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_f32_store(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::F32_STORE);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_f64_store(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::F64_STORE);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i32_store8(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I32_STORE8);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i32_store16(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I32_STORE16);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i64_store8(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I64_STORE8);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i64_store16(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I64_STORE16);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i64_store32(&mut self, _align:u32, offset:u32) -> Self::Output {
        self.push_byte(opcode::I64_STORE32);
        self.push_bytes(&offset.to_ne_bytes());
    }

    fn visit_i32_const(&mut self, value: i32) -> Self::Output {
        self.push_byte(opcode::I32_CONST);
        self.push_bytes(&value.to_ne_bytes());
    }

    fn visit_i64_const(&mut self, value: i64) -> Self::Output {
        self.push_byte(opcode::I64_CONST);
        self.push_bytes(&value.to_ne_bytes());
    }

    fn visit_f32_const(&mut self, value: f32) -> Self::Output {
        self.push_byte(opcode::F32_CONST);
        self.push_bytes(&value.to_ne_bytes());
    }

    fn visit_f64_const(&mut self, value: f64) -> Self::Output {
        self.push_byte(opcode::F64_CONST);
        self.push_bytes(&value.to_ne_bytes());
    }

    fn visit_i32_eqz(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_EQZ);
    }

    fn visit_i32_eq(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_EQ);
    }

    fn visit_i32_ne(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_NE);
    }

    fn visit_i32_lt_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_LT_S);
    }

    fn visit_i32_lt_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_LT_U);
    }

    fn visit_i32_gt_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_GT_S);
    }

    fn visit_i32_gt_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_GT_U);
    }

    fn visit_i32_le_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_LE_S);
    }

    fn visit_i32_le_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_LE_U);
    }

    fn visit_i32_ge_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_GE_S);
    }

    fn visit_i32_ge_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_GE_U);
    }

    fn visit_i64_eqz(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_EQZ);
    }

    fn visit_i64_eq(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_EQ);
    }

    fn visit_i64_ne(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_NE);
    }

    fn visit_i64_lt_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_LT_S);
    }

    fn visit_i64_lt_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_LT_U);
    }

    fn visit_i64_gt_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_GT_S);
    }

    fn visit_i64_gt_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_GT_U);
    }

    fn visit_i64_le_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_LE_S);
    }

    fn visit_i64_le_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_LE_U);
    }

    fn visit_i64_ge_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_GE_S);
    }

    fn visit_i64_ge_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_GE_U);
    }

    fn visit_f32_eq(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_EQ);
    }

    fn visit_f32_ne(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_NE);
    }

    fn visit_f32_lt(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_LT);
    }

    fn visit_f32_gt(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_GT);
    }

    fn visit_f32_le(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_LE);
    }

    fn visit_f32_ge(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_GE);
    }

    fn visit_f64_eq(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_EQ);
    }

    fn visit_f64_ne(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_NE);
    }

    fn visit_f64_lt(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_LT);
    }

    fn visit_f64_gt(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_GT);
    }

    fn visit_f64_le(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_LE);
    }

    fn visit_f64_ge(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_GE);
    }

    fn visit_i32_clz(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_CLZ);
    }

    fn visit_i32_ctz(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_CTZ);
    }

    fn visit_i32_popcnt(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_POPCNT);
    }

    fn visit_i32_add(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_ADD);
    }

    fn visit_i32_sub(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_SUB);
    }

    fn visit_i32_mul(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_MUL);
    }

    fn visit_i32_div_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_DIV_S);
    }

    fn visit_i32_div_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_DIV_U);
    }

    fn visit_i32_rem_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_REM_S);
    }

    fn visit_i32_rem_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_REM_U);
    }

    fn visit_i32_and(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_AND);
    }

    fn visit_i32_or(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_OR);
    }

    fn visit_i32_xor(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_XOR);
    }

    fn visit_i32_shl(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_SHL);
    }

    fn visit_i32_shr_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_SHR_S);
    }

    fn visit_i32_shr_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_SHR_U);
    }

    fn visit_i32_rotl(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_ROTL);
    }

    fn visit_i32_rotr(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_ROTR);
    }

    fn visit_i64_clz(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_CLZ);
    }

    fn visit_i64_ctz(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_CTZ);
    }

    fn visit_i64_popcnt(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_POPCNT);
    }

    fn visit_i64_add(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_ADD);
    }

    fn visit_i64_sub(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_SUB);
    }

    fn visit_i64_mul(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_MUL);
    }

    fn visit_i64_div_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_DIV_S);
    }

    fn visit_i64_div_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_DIV_U);
    }

    fn visit_i64_rem_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_REM_S);
    }

    fn visit_i64_rem_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_REM_U);
    }

    fn visit_i64_and(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_AND);
    }

    fn visit_i64_or(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_OR);
    }

    fn visit_i64_xor(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_XOR);
    }

    fn visit_i64_shl(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_SHL);
    }

    fn visit_i64_shr_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_SHR_S);
    }

    fn visit_i64_shr_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_SHR_U);
    }

    fn visit_i64_rotl(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_ROTL);
    }

    fn visit_i64_rotr(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_ROTR);
    }

    fn visit_f32_abs(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_ABS);
    }

    fn visit_f32_neg(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_NEG);
    }

    fn visit_f32_ceil(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_CEIL);
    }

    fn visit_f32_floor(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_FLOOR);
    }

    fn visit_f32_trunc(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_TRUNC);
    }

    fn visit_f32_nearest(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_NEAREST);
    }

    fn visit_f32_sqrt(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_SQRT);
    }

    fn visit_f32_add(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_ADD);
    }

    fn visit_f32_sub(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_SUB);
    }

    fn visit_f32_mul(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_MUL);
    }

    fn visit_f32_div(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_DIV);
    }

    fn visit_f32_min(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_MIN);
    }

    fn visit_f32_max(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_MAX);
    }

    fn visit_f32_copysign(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_COPYSIGN);
    }

    fn visit_f64_abs(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_ABS);
    }

    fn visit_f64_neg(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_NEG);
    }

    fn visit_f64_ceil(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_CEIL);
    }

    fn visit_f64_floor(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_FLOOR);
    }

    fn visit_f64_trunc(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_TRUNC);
    }

    fn visit_f64_nearest(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_NEAREST);
    }

    fn visit_f64_sqrt(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_SQRT);
    }

    fn visit_f64_add(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_ADD);
    }

    fn visit_f64_sub(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_SUB);
    }

    fn visit_f64_mul(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_MUL);
    }

    fn visit_f64_div(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_DIV);
    }

    fn visit_f64_min(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_MIN);
    }

    fn visit_f64_max(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_MAX);
    }

    fn visit_f64_copysign(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_COPYSIGN);
    }

    fn visit_i32_wrap_i64(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_WRAP_I64);
    }

    fn visit_i32_trunc_f32_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_TRUNC_F32_S);
    }

    fn visit_i32_trunc_f32_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_TRUNC_F32_U);
    }

    fn visit_i32_trunc_f64_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_TRUNC_F64_S);
    }

    fn visit_i32_trunc_f64_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_TRUNC_F64_U);
    }

    fn visit_i64_extend_i32_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_EXTEND_I32_S);
    }

    fn visit_i64_extend_i32_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_EXTEND_I32_U);
    }

    fn visit_i64_trunc_f32_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_TRUNC_F32_S);
    }

    fn visit_i64_trunc_f32_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_TRUNC_F32_U);
    }

    fn visit_i64_trunc_f64_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_TRUNC_F64_S);
    }

    fn visit_i64_trunc_f64_u(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_TRUNC_F64_U);
    }

    fn visit_f32_convert_i32_s(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_CONVERT_I32_S);
    }

    fn visit_f32_convert_i32_u(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_CONVERT_I32_U);
    }

    fn visit_f32_convert_i64_s(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_CONVERT_I64_S);
    }

    fn visit_f32_convert_i64_u(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_CONVERT_I64_U);
    }

    fn visit_f32_demote_f64(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_DEMOTE_F64);
    }

    fn visit_f64_convert_i32_s(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_CONVERT_I32_S);
    }

    fn visit_f64_convert_i32_u(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_CONVERT_I32_U);
    }

    fn visit_f64_convert_i64_s(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_CONVERT_I64_S);
    }

    fn visit_f64_convert_i64_u(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_CONVERT_I64_U);
    }

    fn visit_f64_promote_f32(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_PROMOTE_F32);
    }

    fn visit_i32_reinterpret_f32(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_REINTERPRET_F32);
    }

    fn visit_i64_reinterpret_f64(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_REINTERPRET_F64);
    }

    fn visit_f32_reinterpret_i32(&mut self) -> Self::Output {
        self.push_byte(opcode::F32_REINTERPRET_I32);
    }

    fn visit_f64_reinterpret_i64(&mut self) -> Self::Output {
        self.push_byte(opcode::F64_REINTERPRET_I64);
    }

    fn visit_i32_extend8_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_EXTEND8_S);
    }

    fn visit_i32_extend16_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I32_EXTEND16_S);
    }

    fn visit_i64_extend8_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_EXTEND8_S);
    }

    fn visit_i64_extend16_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_EXTEND16_S);
    }

    fn visit_i64_extend32_s(&mut self) -> Self::Output {
        self.push_byte(opcode::I64_EXTEND32_S);
    }

    fn visit_ref_null(&mut self) -> Self::Output {
        self.push_byte(opcode::REF_NULL);
    }

    fn visit_ref_is_null(&mut self) -> Self::Output {
        self.push_byte(opcode::REF_IS_NULL);
    }

    fn visit_ref_func(&mut self) -> Self::Output {
        self.push_byte(opcode::REF_FUNC);
    }

    fn visit_memory_size(&mut self, mem: MemoryIdx) -> Self::Output {
        self.push_byte(opcode::MEMORY_SIZE);
        self.push_bytes(&mem.to_ne_bytes());
    }

    fn visit_memory_grow(&mut self, mem: MemoryIdx) -> Self::Output {
        self.push_byte(opcode::MEMORY_GROW);
        self.push_bytes(&mem.to_ne_bytes());
    }

    fn visit_memory_copy(&mut self, dst: MemoryIdx, src: MemoryIdx) -> Self::Output {
        self.push_byte(0xfc);
        self.push_bytes(&opcode::xfc::MEMORY_COPY.to_ne_bytes());
        self.push_bytes(&dst.to_ne_bytes());
        self.push_bytes(&src.to_ne_bytes());
    }

    fn visit_memory_fill(&mut self, mem: MemoryIdx) -> Self::Output {
        self.push_byte(0xfc);
        self.push_bytes(&opcode::xfc::MEMORY_FILL.to_ne_bytes());
        self.push_bytes(&mem.to_ne_bytes());
    }
}

