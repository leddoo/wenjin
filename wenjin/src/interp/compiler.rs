use sti::boks::Box;
use sti::manual_vec::ManualVec;

use wasm::{ValueType, BlockType, TypeIdx, FuncIdx, TableIdx, MemoryIdx, GlobalIdx, opcode, BrTable, ValidatorError};

use crate::{InstanceId, store::InterpFunc};

/*
- instructions:
    - instructions are mapped 1:1 for ease of debugging.
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


pub(crate) struct Compiler<'a> {
    module: &'a wasm::Module<'a>,
    validator: wasm::Validator<'a>,

    code: ManualVec<u8>,
    frames: ManualVec<Frame>,

    num_params: u32,
    begin_stack: u32,
    begin_unreachable: bool,
    max_stack: u32,
    max_align: u32,

    oom: bool,
}

// address of the label.
type Label = u32;

// last use of the label.
// u32::MAX -> none.
type LabelUse = u32;

#[derive(Clone, Debug)]
struct Frame {
    kind: FrameKind,
    height: u32,
    num_params: u32,
    num_rets: u32,
}

#[derive(Clone, Debug)]
enum FrameKind {
    Block { after: LabelUse },
    If { after: LabelUse, else_use: u32 },
    Else { after: LabelUse },
    Loop { head: Label },
}

impl<'a> Compiler<'a> {
    pub fn new(module: &'a wasm::Module<'a>) -> Self {
        Self {
            module,
            validator: wasm::Validator::new(module),
            code: ManualVec::new(),
            frames: ManualVec::new(),
            num_params: 0,
            begin_stack: 0,
            begin_unreachable: false,
            max_stack: 0,
            max_align: 0,
            oom: false,
        }
    }

    pub fn begin_func(&mut self, ty: TypeIdx, locals: &[ValueType]) -> Result<(), ValidatorError> {
        self.validator.begin_func(ty, locals)?;

        self.code.clear();
        self.frames.clear();
        self.num_params = (self.validator.locals().len() - locals.len()) as u32;
        self.begin_stack = 0;
        self.begin_unreachable = false;
        self.max_stack = 0;
        self.max_align = 0;
        self.push_frame_core(Frame {
            kind: FrameKind::Block { after: u32::MAX },
            height: 0,
            num_params: 0,
            num_rets: self.module.types[ty as usize].rets.len() as u32,
        });
        return Ok(());
    }

    pub fn end_func(&mut self, instance: InstanceId) -> Result<InterpFunc, ValidatorError> {
        self.validator.end_func()?;

        debug_assert_eq!(self.frames.len(), 0);
        if self.oom {
            return Err(ValidatorError::OOM);
        }

        let len = self.code.len();

        if 0==1 { crate::interp::dump(&self.code); }

        let alloc = sti::alloc::GlobalAlloc;
        let ptr = sti::alloc::alloc_array::<u8, _>(&alloc, len).ok_or_else(|| ValidatorError::OOM)?;
        let code = unsafe {
            core::ptr::copy_nonoverlapping(
                self.code.as_ptr(),
                ptr.as_ptr(),
                len);

            let ptr = core::slice::from_raw_parts_mut(ptr.as_ptr(), len);
            let ptr = core::ptr::NonNull::from(ptr);
            let ptr = core::mem::transmute(ptr);
            Box::from_raw_parts(ptr, alloc)
        };

        let num_locals = self.validator.num_locals();
        Ok(InterpFunc {
            instance,
            code,
            num_params: self.num_params,
            num_locals,
            stack_size: num_locals + self.max_stack,
        })
    }

    pub fn begin_operator(&mut self, _wasm_position: u32) {
        //println!("{:?}", self.validator.stack());
        self.begin_stack = self.validator.num_stack();
        self.begin_unreachable = self.validator.is_unreachable();
    }

    pub fn end_operator(&mut self) {
        let end_stack = self.validator.num_stack();

        self.max_stack = self.max_stack.max(end_stack);

        for i in self.begin_stack as usize .. end_stack as usize {
            let size = match self.validator.stack()[i] {
                ValueType::I32 => 4,
                ValueType::I64 => 8,
                ValueType::F32 => 4,
                ValueType::F64 => 8,
                ValueType::V128 => 16,
                ValueType::FuncRef => 4,
                ValueType::ExternRef => 4,
            };
            self.max_align = self.max_align.max(size);
        }
    }


    #[inline]
    fn add_byte(&mut self, byte: u8) {
        if 1 > i32::MAX as usize - self.code.len()
        || self.code.push_or_alloc(byte).is_err() {
            self.oom = true;
        }
    }

    #[inline]
    fn add_bytes(&mut self, bytes: &[u8]) {
        if bytes.len() > i32::MAX as usize - self.code.len()
        || self.code.extend_from_slice_or_alloc(bytes).is_err() {
            self.oom = true;
        }
    }

    fn push_frame(&mut self, kind: FrameKind, ty: BlockType, pop: u32) {
        let num_params = ty.begin_types(self.module).len() as u32;
        let num_rets = ty.end_types(self.module).len() as u32;
        let height = self.begin_stack - if !self.begin_unreachable { num_params + pop } else { 0 };
        self.push_frame_core(Frame { kind, height, num_params, num_rets });
    }

    fn push_frame_core(&mut self, frame: Frame) {
        if self.frames.push_or_alloc(frame).is_err() {
            self.oom = true;
        }
    }

    fn pop_frame(&mut self) -> Frame {
        self.frames.pop().unwrap()
    }

    fn jump(&mut self, label: Label) {
        let frame = self.frames.rev_mut(label as usize);
        match &mut frame.kind {
            FrameKind::Block { after } |
            FrameKind::If { after, else_use: _ } |
            FrameKind::Else { after } => {
                let prev = *after;
                *after = self.code.len() as u32;
                self.add_bytes(&prev.to_ne_bytes());
            }

            FrameKind::Loop { head } => {
                let delta = *head as i32 - self.code.len() as i32;
                self.add_bytes(&delta.to_ne_bytes());
            }
        }
    }

    fn br_shift(&mut self, label: Label, pop: u32) {
        let frame = self.frames.rev_mut(label as usize);
        let num = match frame.kind {
            FrameKind::Block {..} |
            FrameKind::If {..} |
            FrameKind::Else {..} => frame.num_rets,
            FrameKind::Loop {..} => frame.num_params,
        };
        let by =
            if !self.begin_unreachable {
                (self.begin_stack - pop - num) - frame.height
            }
            else { 0 };
        self.add_bytes(&num.to_ne_bytes());
        self.add_bytes(&by.to_ne_bytes());
    }

    fn jump_and_shift(&mut self, label: Label, pop: u32) {
        self.jump(label);
        self.br_shift(label, pop);
    }

    fn patch_jumps(&mut self, last_use: LabelUse, dst: Label) {
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

impl<'a> wasm::OperatorVisitor<'a> for Compiler<'a> {
    type Output = Result<(), wasm::ValidatorError>;

    fn visit_unreachable(&mut self) -> Self::Output {
        self.validator.visit_unreachable()?;
        self.add_byte(opcode::UNREACHABLE);
        Ok(())
    }

    fn visit_nop(&mut self) -> Self::Output {
        self.validator.visit_nop()?;
        self.add_byte(opcode::NOP);
        Ok(())
    }

    fn visit_block(&mut self, ty: BlockType) -> Self::Output {
        self.validator.visit_block(ty)?;
        self.add_byte(opcode::BLOCK);
        self.push_frame(FrameKind::Block { after: u32::MAX }, ty, 0);
        Ok(())
    }

    fn visit_loop(&mut self, ty: BlockType) -> Self::Output {
        self.validator.visit_loop(ty)?;
        self.add_byte(opcode::LOOP);
        self.push_frame(FrameKind::Loop { head: self.code.len() as u32 }, ty, 0);
        Ok(())
    }

    fn visit_if(&mut self, ty: BlockType) -> Self::Output {
        self.validator.visit_if(ty)?;
        self.add_byte(opcode::IF);
        let else_use = self.code.len() as u32;
        self.add_bytes(&u32::MAX.to_ne_bytes());
        self.push_frame(FrameKind::If { after: u32::MAX, else_use }, ty, 1);
        Ok(())
    }

    fn visit_else(&mut self) -> Self::Output {
        self.validator.visit_else()?;
        self.add_byte(opcode::ELSE);
        self.jump_and_shift(0, 0);

        let frame = self.pop_frame();

        let FrameKind::If { after, else_use } = frame.kind else { panic!("invalid wasm") };
        //self.push_frame_core(FrameKind::Else { after }, frame.num_params, frame.num_rets, 0);
        _ = self.frames.push(Frame { kind: FrameKind::Else { after }, ..frame });

        let else_offset = self.code.len() as u32;
        self.patch_jumps(else_use, else_offset);
        Ok(())
    }

    fn visit_end(&mut self) -> Self::Output {
        self.validator.visit_end()?;
        let frame = self.pop_frame();

        let offset = self.code.len() as u32;
        match frame.kind {
            FrameKind::Block { after } |
            FrameKind::Else { after } => {
                self.patch_jumps(after, offset);
            }

            FrameKind::If { after, else_use } => {
                self.patch_jumps(after, offset);
                self.patch_jumps(else_use, offset);
            }

            FrameKind::Loop { head: _ } => (),
        }

        self.add_byte(opcode::END);

        if self.frames.len() == 0 {
            self.add_byte(opcode::RETURN);
            self.add_bytes(&frame.num_rets.to_ne_bytes());
        }
        Ok(())
    }

    fn visit_br(&mut self, label: u32) -> Self::Output {
        self.validator.visit_br(label)?;
        self.add_byte(opcode::BR);
        self.jump_and_shift(label, 0);
        Ok(())
    }

    fn visit_br_if(&mut self, label: u32) -> Self::Output {
        self.validator.visit_br_if(label)?;
        self.add_byte(opcode::BR_IF);
        self.jump_and_shift(label, 1);
        Ok(())
    }

    fn visit_br_table(&mut self, table: BrTable<'a>) -> Self::Output {
        self.validator.visit_br_table(table)?;
        self.add_byte(opcode::BR_TABLE);
        self.add_bytes(&table.num_labels.to_ne_bytes());
        for label in table.labels() {
            self.jump_and_shift(label, 1);
        }
        self.jump_and_shift(table.default, 1);
        Ok(())
    }

    fn visit_return(&mut self) -> Self::Output {
        self.validator.visit_return()?;
        self.add_byte(opcode::RETURN);
        self.add_bytes(&self.frames[0].num_rets.to_ne_bytes());
        Ok(())
    }

    fn visit_call(&mut self, func: FuncIdx) -> Self::Output {
        self.validator.visit_call(func)?;
        self.add_byte(opcode::CALL);
        self.add_bytes(&func.to_ne_bytes());
        Ok(())
    }

    fn visit_call_indirect(&mut self, type_idx: TypeIdx, table: TableIdx) -> Self::Output {
        self.validator.visit_call_indirect(type_idx, table)?;
        self.add_byte(opcode::CALL_INDIRECT);
        self.add_bytes(&type_idx.to_ne_bytes());
        self.add_bytes(&table.to_ne_bytes());
        Ok(())
    }

    fn visit_drop(&mut self) -> Self::Output {
        self.validator.visit_drop()?;
        self.add_byte(opcode::DROP);
        Ok(())
    }

    fn visit_select(&mut self) -> Self::Output {
        self.validator.visit_select()?;
        self.add_byte(opcode::SELECT);
        Ok(())
    }

    fn visit_typed_select(&mut self, ty: ValueType) -> Self::Output {
        self.validator.visit_typed_select(ty)?;
        self.add_byte(opcode::SELECT);
        Ok(())
    }

    fn visit_local_get(&mut self, idx: u32) -> Self::Output {
        self.validator.visit_local_get(idx)?;
        self.add_byte(opcode::LOCAL_GET);
        self.add_bytes(&idx.to_ne_bytes());
        Ok(())
    }

    fn visit_local_set(&mut self, idx: u32) -> Self::Output {
        self.validator.visit_local_set(idx)?;
        self.add_byte(opcode::LOCAL_SET);
        self.add_bytes(&idx.to_ne_bytes());
        Ok(())
    }

    fn visit_local_tee(&mut self, idx: u32) -> Self::Output {
        self.validator.visit_local_tee(idx)?;
        self.add_byte(opcode::LOCAL_TEE);
        self.add_bytes(&idx.to_ne_bytes());
        Ok(())
    }

    fn visit_global_get(&mut self, idx: GlobalIdx) -> Self::Output {
        self.validator.visit_global_get(idx)?;
        self.add_byte(opcode::GLOBAL_GET);
        self.add_bytes(&idx.to_ne_bytes());
        Ok(())
    }

    fn visit_global_set(&mut self, idx: GlobalIdx) -> Self::Output {
        self.validator.visit_global_set(idx)?;
        self.add_byte(opcode::GLOBAL_SET);
        self.add_bytes(&idx.to_ne_bytes());
        Ok(())
    }

    fn visit_table_get(&mut self, idx: TableIdx) -> Self::Output {
        self.validator.visit_table_get(idx)?;
        self.add_byte(opcode::TABLE_GET);
        self.add_bytes(&idx.to_ne_bytes());
        Ok(())
    }

    fn visit_table_set(&mut self, idx: TableIdx) -> Self::Output {
        self.validator.visit_table_set(idx)?;
        self.add_byte(opcode::TABLE_SET);
        self.add_bytes(&idx.to_ne_bytes());
        Ok(())
    }

    fn visit_i32_load(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i32_load(align, offset)?;
        self.add_byte(opcode::I32_LOAD);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i64_load(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i64_load(align, offset)?;
        self.add_byte(opcode::I64_LOAD);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_f32_load(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_f32_load(align, offset)?;
        self.add_byte(opcode::F32_LOAD);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_f64_load(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_f64_load(align, offset)?;
        self.add_byte(opcode::F64_LOAD);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i32_load8_s(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i32_load8_s(align, offset)?;
        self.add_byte(opcode::I32_LOAD8_S);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i32_load8_u(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i32_load8_u(align, offset)?;
        self.add_byte(opcode::I32_LOAD8_U);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i32_load16_s(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i32_load16_s(align, offset)?;
        self.add_byte(opcode::I32_LOAD16_S);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i32_load16_u(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i32_load16_u(align, offset)?;
        self.add_byte(opcode::I32_LOAD16_U);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i64_load8_s(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i64_load8_s(align, offset)?;
        self.add_byte(opcode::I64_LOAD8_S);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i64_load8_u(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i64_load8_u(align, offset)?;
        self.add_byte(opcode::I64_LOAD8_U);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i64_load16_s(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i64_load16_s(align, offset)?;
        self.add_byte(opcode::I64_LOAD16_S);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i64_load16_u(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i64_load16_u(align, offset)?;
        self.add_byte(opcode::I64_LOAD16_U);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i64_load32_s(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i64_load32_s(align, offset)?;
        self.add_byte(opcode::I64_LOAD32_S);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i64_load32_u(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i64_load32_u(align, offset)?;
        self.add_byte(opcode::I64_LOAD32_U);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i32_store(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i32_store(align, offset)?;
        self.add_byte(opcode::I32_STORE);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i64_store(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i64_store(align, offset)?;
        self.add_byte(opcode::I64_STORE);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_f32_store(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_f32_store(align, offset)?;
        self.add_byte(opcode::F32_STORE);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_f64_store(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_f64_store(align, offset)?;
        self.add_byte(opcode::F64_STORE);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i32_store8(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i32_store8(align, offset)?;
        self.add_byte(opcode::I32_STORE8);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i32_store16(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i32_store16(align, offset)?;
        self.add_byte(opcode::I32_STORE16);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i64_store8(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i64_store8(align, offset)?;
        self.add_byte(opcode::I64_STORE8);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i64_store16(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i64_store16(align, offset)?;
        self.add_byte(opcode::I64_STORE16);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i64_store32(&mut self, align:u32, offset:u32) -> Self::Output {
        self.validator.visit_i64_store32(align, offset)?;
        self.add_byte(opcode::I64_STORE32);
        self.add_bytes(&offset.to_ne_bytes());
        Ok(())
    }

    fn visit_i32_const(&mut self, value: i32) -> Self::Output {
        self.validator.visit_i32_const(value)?;
        self.add_byte(opcode::I32_CONST);
        self.add_bytes(&value.to_ne_bytes());
        Ok(())
    }

    fn visit_i64_const(&mut self, value: i64) -> Self::Output {
        self.validator.visit_i64_const(value)?;
        self.add_byte(opcode::I64_CONST);
        self.add_bytes(&value.to_ne_bytes());
        Ok(())
    }

    fn visit_f32_const(&mut self, value: f32) -> Self::Output {
        self.validator.visit_f32_const(value)?;
        self.add_byte(opcode::F32_CONST);
        self.add_bytes(&value.to_ne_bytes());
        Ok(())
    }

    fn visit_f64_const(&mut self, value: f64) -> Self::Output {
        self.validator.visit_f64_const(value)?;
        self.add_byte(opcode::F64_CONST);
        self.add_bytes(&value.to_ne_bytes());
        Ok(())
    }

    fn visit_i32_eqz(&mut self) -> Self::Output {
        self.validator.visit_i32_eqz()?;
        self.add_byte(opcode::I32_EQZ);
        Ok(())
    }

    fn visit_i32_eq(&mut self) -> Self::Output {
        self.validator.visit_i32_eq()?;
        self.add_byte(opcode::I32_EQ);
        Ok(())
    }

    fn visit_i32_ne(&mut self) -> Self::Output {
        self.validator.visit_i32_ne()?;
        self.add_byte(opcode::I32_NE);
        Ok(())
    }

    fn visit_i32_lt_s(&mut self) -> Self::Output {
        self.validator.visit_i32_lt_s()?;
        self.add_byte(opcode::I32_LT_S);
        Ok(())
    }

    fn visit_i32_lt_u(&mut self) -> Self::Output {
        self.validator.visit_i32_lt_u()?;
        self.add_byte(opcode::I32_LT_U);
        Ok(())
    }

    fn visit_i32_gt_s(&mut self) -> Self::Output {
        self.validator.visit_i32_gt_s()?;
        self.add_byte(opcode::I32_GT_S);
        Ok(())
    }

    fn visit_i32_gt_u(&mut self) -> Self::Output {
        self.validator.visit_i32_gt_u()?;
        self.add_byte(opcode::I32_GT_U);
        Ok(())
    }

    fn visit_i32_le_s(&mut self) -> Self::Output {
        self.validator.visit_i32_le_s()?;
        self.add_byte(opcode::I32_LE_S);
        Ok(())
    }

    fn visit_i32_le_u(&mut self) -> Self::Output {
        self.validator.visit_i32_le_u()?;
        self.add_byte(opcode::I32_LE_U);
        Ok(())
    }

    fn visit_i32_ge_s(&mut self) -> Self::Output {
        self.validator.visit_i32_ge_s()?;
        self.add_byte(opcode::I32_GE_S);
        Ok(())
    }

    fn visit_i32_ge_u(&mut self) -> Self::Output {
        self.validator.visit_i32_ge_u()?;
        self.add_byte(opcode::I32_GE_U);
        Ok(())
    }

    fn visit_i64_eqz(&mut self) -> Self::Output {
        self.validator.visit_i64_eqz()?;
        self.add_byte(opcode::I64_EQZ);
        Ok(())
    }

    fn visit_i64_eq(&mut self) -> Self::Output {
        self.validator.visit_i64_eq()?;
        self.add_byte(opcode::I64_EQ);
        Ok(())
    }

    fn visit_i64_ne(&mut self) -> Self::Output {
        self.validator.visit_i64_ne()?;
        self.add_byte(opcode::I64_NE);
        Ok(())
    }

    fn visit_i64_lt_s(&mut self) -> Self::Output {
        self.validator.visit_i64_lt_s()?;
        self.add_byte(opcode::I64_LT_S);
        Ok(())
    }

    fn visit_i64_lt_u(&mut self) -> Self::Output {
        self.validator.visit_i64_lt_u()?;
        self.add_byte(opcode::I64_LT_U);
        Ok(())
    }

    fn visit_i64_gt_s(&mut self) -> Self::Output {
        self.validator.visit_i64_gt_s()?;
        self.add_byte(opcode::I64_GT_S);
        Ok(())
    }

    fn visit_i64_gt_u(&mut self) -> Self::Output {
        self.validator.visit_i64_gt_u()?;
        self.add_byte(opcode::I64_GT_U);
        Ok(())
    }

    fn visit_i64_le_s(&mut self) -> Self::Output {
        self.validator.visit_i64_le_s()?;
        self.add_byte(opcode::I64_LE_S);
        Ok(())
    }

    fn visit_i64_le_u(&mut self) -> Self::Output {
        self.validator.visit_i64_le_u()?;
        self.add_byte(opcode::I64_LE_U);
        Ok(())
    }

    fn visit_i64_ge_s(&mut self) -> Self::Output {
        self.validator.visit_i64_ge_s()?;
        self.add_byte(opcode::I64_GE_S);
        Ok(())
    }

    fn visit_i64_ge_u(&mut self) -> Self::Output {
        self.validator.visit_i64_ge_u()?;
        self.add_byte(opcode::I64_GE_U);
        Ok(())
    }

    fn visit_f32_eq(&mut self) -> Self::Output {
        self.validator.visit_f32_eq()?;
        self.add_byte(opcode::F32_EQ);
        Ok(())
    }

    fn visit_f32_ne(&mut self) -> Self::Output {
        self.validator.visit_f32_ne()?;
        self.add_byte(opcode::F32_NE);
        Ok(())
    }

    fn visit_f32_lt(&mut self) -> Self::Output {
        self.validator.visit_f32_lt()?;
        self.add_byte(opcode::F32_LT);
        Ok(())
    }

    fn visit_f32_gt(&mut self) -> Self::Output {
        self.validator.visit_f32_gt()?;
        self.add_byte(opcode::F32_GT);
        Ok(())
    }

    fn visit_f32_le(&mut self) -> Self::Output {
        self.validator.visit_f32_le()?;
        self.add_byte(opcode::F32_LE);
        Ok(())
    }

    fn visit_f32_ge(&mut self) -> Self::Output {
        self.validator.visit_f32_ge()?;
        self.add_byte(opcode::F32_GE);
        Ok(())
    }

    fn visit_f64_eq(&mut self) -> Self::Output {
        self.validator.visit_f64_eq()?;
        self.add_byte(opcode::F64_EQ);
        Ok(())
    }

    fn visit_f64_ne(&mut self) -> Self::Output {
        self.validator.visit_f64_ne()?;
        self.add_byte(opcode::F64_NE);
        Ok(())
    }

    fn visit_f64_lt(&mut self) -> Self::Output {
        self.validator.visit_f64_lt()?;
        self.add_byte(opcode::F64_LT);
        Ok(())
    }

    fn visit_f64_gt(&mut self) -> Self::Output {
        self.validator.visit_f64_gt()?;
        self.add_byte(opcode::F64_GT);
        Ok(())
    }

    fn visit_f64_le(&mut self) -> Self::Output {
        self.validator.visit_f64_le()?;
        self.add_byte(opcode::F64_LE);
        Ok(())
    }

    fn visit_f64_ge(&mut self) -> Self::Output {
        self.validator.visit_f64_ge()?;
        self.add_byte(opcode::F64_GE);
        Ok(())
    }

    fn visit_i32_clz(&mut self) -> Self::Output {
        self.validator.visit_i32_clz()?;
        self.add_byte(opcode::I32_CLZ);
        Ok(())
    }

    fn visit_i32_ctz(&mut self) -> Self::Output {
        self.validator.visit_i32_ctz()?;
        self.add_byte(opcode::I32_CTZ);
        Ok(())
    }

    fn visit_i32_popcnt(&mut self) -> Self::Output {
        self.validator.visit_i32_popcnt()?;
        self.add_byte(opcode::I32_POPCNT);
        Ok(())
    }

    fn visit_i32_add(&mut self) -> Self::Output {
        self.validator.visit_i32_add()?;
        self.add_byte(opcode::I32_ADD);
        Ok(())
    }

    fn visit_i32_sub(&mut self) -> Self::Output {
        self.validator.visit_i32_sub()?;
        self.add_byte(opcode::I32_SUB);
        Ok(())
    }

    fn visit_i32_mul(&mut self) -> Self::Output {
        self.validator.visit_i32_mul()?;
        self.add_byte(opcode::I32_MUL);
        Ok(())
    }

    fn visit_i32_div_s(&mut self) -> Self::Output {
        self.validator.visit_i32_div_s()?;
        self.add_byte(opcode::I32_DIV_S);
        Ok(())
    }

    fn visit_i32_div_u(&mut self) -> Self::Output {
        self.validator.visit_i32_div_u()?;
        self.add_byte(opcode::I32_DIV_U);
        Ok(())
    }

    fn visit_i32_rem_s(&mut self) -> Self::Output {
        self.validator.visit_i32_rem_s()?;
        self.add_byte(opcode::I32_REM_S);
        Ok(())
    }

    fn visit_i32_rem_u(&mut self) -> Self::Output {
        self.validator.visit_i32_rem_u()?;
        self.add_byte(opcode::I32_REM_U);
        Ok(())
    }

    fn visit_i32_and(&mut self) -> Self::Output {
        self.validator.visit_i32_and()?;
        self.add_byte(opcode::I32_AND);
        Ok(())
    }

    fn visit_i32_or(&mut self) -> Self::Output {
        self.validator.visit_i32_or()?;
        self.add_byte(opcode::I32_OR);
        Ok(())
    }

    fn visit_i32_xor(&mut self) -> Self::Output {
        self.validator.visit_i32_xor()?;
        self.add_byte(opcode::I32_XOR);
        Ok(())
    }

    fn visit_i32_shl(&mut self) -> Self::Output {
        self.validator.visit_i32_shl()?;
        self.add_byte(opcode::I32_SHL);
        Ok(())
    }

    fn visit_i32_shr_s(&mut self) -> Self::Output {
        self.validator.visit_i32_shr_s()?;
        self.add_byte(opcode::I32_SHR_S);
        Ok(())
    }

    fn visit_i32_shr_u(&mut self) -> Self::Output {
        self.validator.visit_i32_shr_u()?;
        self.add_byte(opcode::I32_SHR_U);
        Ok(())
    }

    fn visit_i32_rotl(&mut self) -> Self::Output {
        self.validator.visit_i32_rotl()?;
        self.add_byte(opcode::I32_ROTL);
        Ok(())
    }

    fn visit_i32_rotr(&mut self) -> Self::Output {
        self.validator.visit_i32_rotr()?;
        self.add_byte(opcode::I32_ROTR);
        Ok(())
    }

    fn visit_i64_clz(&mut self) -> Self::Output {
        self.validator.visit_i64_clz()?;
        self.add_byte(opcode::I64_CLZ);
        Ok(())
    }

    fn visit_i64_ctz(&mut self) -> Self::Output {
        self.validator.visit_i64_ctz()?;
        self.add_byte(opcode::I64_CTZ);
        Ok(())
    }

    fn visit_i64_popcnt(&mut self) -> Self::Output {
        self.validator.visit_i64_popcnt()?;
        self.add_byte(opcode::I64_POPCNT);
        Ok(())
    }

    fn visit_i64_add(&mut self) -> Self::Output {
        self.validator.visit_i64_add()?;
        self.add_byte(opcode::I64_ADD);
        Ok(())
    }

    fn visit_i64_sub(&mut self) -> Self::Output {
        self.validator.visit_i64_sub()?;
        self.add_byte(opcode::I64_SUB);
        Ok(())
    }

    fn visit_i64_mul(&mut self) -> Self::Output {
        self.validator.visit_i64_mul()?;
        self.add_byte(opcode::I64_MUL);
        Ok(())
    }

    fn visit_i64_div_s(&mut self) -> Self::Output {
        self.validator.visit_i64_div_s()?;
        self.add_byte(opcode::I64_DIV_S);
        Ok(())
    }

    fn visit_i64_div_u(&mut self) -> Self::Output {
        self.validator.visit_i64_div_u()?;
        self.add_byte(opcode::I64_DIV_U);
        Ok(())
    }

    fn visit_i64_rem_s(&mut self) -> Self::Output {
        self.validator.visit_i64_rem_s()?;
        self.add_byte(opcode::I64_REM_S);
        Ok(())
    }

    fn visit_i64_rem_u(&mut self) -> Self::Output {
        self.validator.visit_i64_rem_u()?;
        self.add_byte(opcode::I64_REM_U);
        Ok(())
    }

    fn visit_i64_and(&mut self) -> Self::Output {
        self.validator.visit_i64_and()?;
        self.add_byte(opcode::I64_AND);
        Ok(())
    }

    fn visit_i64_or(&mut self) -> Self::Output {
        self.validator.visit_i64_or()?;
        self.add_byte(opcode::I64_OR);
        Ok(())
    }

    fn visit_i64_xor(&mut self) -> Self::Output {
        self.validator.visit_i64_xor()?;
        self.add_byte(opcode::I64_XOR);
        Ok(())
    }

    fn visit_i64_shl(&mut self) -> Self::Output {
        self.validator.visit_i64_shl()?;
        self.add_byte(opcode::I64_SHL);
        Ok(())
    }

    fn visit_i64_shr_s(&mut self) -> Self::Output {
        self.validator.visit_i64_shr_s()?;
        self.add_byte(opcode::I64_SHR_S);
        Ok(())
    }

    fn visit_i64_shr_u(&mut self) -> Self::Output {
        self.validator.visit_i64_shr_u()?;
        self.add_byte(opcode::I64_SHR_U);
        Ok(())
    }

    fn visit_i64_rotl(&mut self) -> Self::Output {
        self.validator.visit_i64_rotl()?;
        self.add_byte(opcode::I64_ROTL);
        Ok(())
    }

    fn visit_i64_rotr(&mut self) -> Self::Output {
        self.validator.visit_i64_rotr()?;
        self.add_byte(opcode::I64_ROTR);
        Ok(())
    }

    fn visit_f32_abs(&mut self) -> Self::Output {
        self.validator.visit_f32_abs()?;
        self.add_byte(opcode::F32_ABS);
        Ok(())
    }

    fn visit_f32_neg(&mut self) -> Self::Output {
        self.validator.visit_f32_neg()?;
        self.add_byte(opcode::F32_NEG);
        Ok(())
    }

    fn visit_f32_ceil(&mut self) -> Self::Output {
        self.validator.visit_f32_ceil()?;
        self.add_byte(opcode::F32_CEIL);
        Ok(())
    }

    fn visit_f32_floor(&mut self) -> Self::Output {
        self.validator.visit_f32_floor()?;
        self.add_byte(opcode::F32_FLOOR);
        Ok(())
    }

    fn visit_f32_trunc(&mut self) -> Self::Output {
        self.validator.visit_f32_trunc()?;
        self.add_byte(opcode::F32_TRUNC);
        Ok(())
    }

    fn visit_f32_nearest(&mut self) -> Self::Output {
        self.validator.visit_f32_nearest()?;
        self.add_byte(opcode::F32_NEAREST);
        Ok(())
    }

    fn visit_f32_sqrt(&mut self) -> Self::Output {
        self.validator.visit_f32_sqrt()?;
        self.add_byte(opcode::F32_SQRT);
        Ok(())
    }

    fn visit_f32_add(&mut self) -> Self::Output {
        self.validator.visit_f32_add()?;
        self.add_byte(opcode::F32_ADD);
        Ok(())
    }

    fn visit_f32_sub(&mut self) -> Self::Output {
        self.validator.visit_f32_sub()?;
        self.add_byte(opcode::F32_SUB);
        Ok(())
    }

    fn visit_f32_mul(&mut self) -> Self::Output {
        self.validator.visit_f32_mul()?;
        self.add_byte(opcode::F32_MUL);
        Ok(())
    }

    fn visit_f32_div(&mut self) -> Self::Output {
        self.validator.visit_f32_div()?;
        self.add_byte(opcode::F32_DIV);
        Ok(())
    }

    fn visit_f32_min(&mut self) -> Self::Output {
        self.validator.visit_f32_min()?;
        self.add_byte(opcode::F32_MIN);
        Ok(())
    }

    fn visit_f32_max(&mut self) -> Self::Output {
        self.validator.visit_f32_max()?;
        self.add_byte(opcode::F32_MAX);
        Ok(())
    }

    fn visit_f32_copysign(&mut self) -> Self::Output {
        self.validator.visit_f32_copysign()?;
        self.add_byte(opcode::F32_COPYSIGN);
        Ok(())
    }

    fn visit_f64_abs(&mut self) -> Self::Output {
        self.validator.visit_f64_abs()?;
        self.add_byte(opcode::F64_ABS);
        Ok(())
    }

    fn visit_f64_neg(&mut self) -> Self::Output {
        self.validator.visit_f64_neg()?;
        self.add_byte(opcode::F64_NEG);
        Ok(())
    }

    fn visit_f64_ceil(&mut self) -> Self::Output {
        self.validator.visit_f64_ceil()?;
        self.add_byte(opcode::F64_CEIL);
        Ok(())
    }

    fn visit_f64_floor(&mut self) -> Self::Output {
        self.validator.visit_f64_floor()?;
        self.add_byte(opcode::F64_FLOOR);
        Ok(())
    }

    fn visit_f64_trunc(&mut self) -> Self::Output {
        self.validator.visit_f64_trunc()?;
        self.add_byte(opcode::F64_TRUNC);
        Ok(())
    }

    fn visit_f64_nearest(&mut self) -> Self::Output {
        self.validator.visit_f64_nearest()?;
        self.add_byte(opcode::F64_NEAREST);
        Ok(())
    }

    fn visit_f64_sqrt(&mut self) -> Self::Output {
        self.validator.visit_f64_sqrt()?;
        self.add_byte(opcode::F64_SQRT);
        Ok(())
    }

    fn visit_f64_add(&mut self) -> Self::Output {
        self.validator.visit_f64_add()?;
        self.add_byte(opcode::F64_ADD);
        Ok(())
    }

    fn visit_f64_sub(&mut self) -> Self::Output {
        self.validator.visit_f64_sub()?;
        self.add_byte(opcode::F64_SUB);
        Ok(())
    }

    fn visit_f64_mul(&mut self) -> Self::Output {
        self.validator.visit_f64_mul()?;
        self.add_byte(opcode::F64_MUL);
        Ok(())
    }

    fn visit_f64_div(&mut self) -> Self::Output {
        self.validator.visit_f64_div()?;
        self.add_byte(opcode::F64_DIV);
        Ok(())
    }

    fn visit_f64_min(&mut self) -> Self::Output {
        self.validator.visit_f64_min()?;
        self.add_byte(opcode::F64_MIN);
        Ok(())
    }

    fn visit_f64_max(&mut self) -> Self::Output {
        self.validator.visit_f64_max()?;
        self.add_byte(opcode::F64_MAX);
        Ok(())
    }

    fn visit_f64_copysign(&mut self) -> Self::Output {
        self.validator.visit_f64_copysign()?;
        self.add_byte(opcode::F64_COPYSIGN);
        Ok(())
    }

    fn visit_i32_wrap_i64(&mut self) -> Self::Output {
        self.validator.visit_i32_wrap_i64()?;
        self.add_byte(opcode::I32_WRAP_I64);
        Ok(())
    }

    fn visit_i32_trunc_f32_s(&mut self) -> Self::Output {
        self.validator.visit_i32_trunc_f32_s()?;
        self.add_byte(opcode::I32_TRUNC_F32_S);
        Ok(())
    }

    fn visit_i32_trunc_f32_u(&mut self) -> Self::Output {
        self.validator.visit_i32_trunc_f32_u()?;
        self.add_byte(opcode::I32_TRUNC_F32_U);
        Ok(())
    }

    fn visit_i32_trunc_f64_s(&mut self) -> Self::Output {
        self.validator.visit_i32_trunc_f64_s()?;
        self.add_byte(opcode::I32_TRUNC_F64_S);
        Ok(())
    }

    fn visit_i32_trunc_f64_u(&mut self) -> Self::Output {
        self.validator.visit_i32_trunc_f64_u()?;
        self.add_byte(opcode::I32_TRUNC_F64_U);
        Ok(())
    }

    fn visit_i64_extend_i32_s(&mut self) -> Self::Output {
        self.validator.visit_i64_extend_i32_s()?;
        self.add_byte(opcode::I64_EXTEND_I32_S);
        Ok(())
    }

    fn visit_i64_extend_i32_u(&mut self) -> Self::Output {
        self.validator.visit_i64_extend_i32_u()?;
        self.add_byte(opcode::I64_EXTEND_I32_U);
        Ok(())
    }

    fn visit_i64_trunc_f32_s(&mut self) -> Self::Output {
        self.validator.visit_i64_trunc_f32_s()?;
        self.add_byte(opcode::I64_TRUNC_F32_S);
        Ok(())
    }

    fn visit_i64_trunc_f32_u(&mut self) -> Self::Output {
        self.validator.visit_i64_trunc_f32_u()?;
        self.add_byte(opcode::I64_TRUNC_F32_U);
        Ok(())
    }

    fn visit_i64_trunc_f64_s(&mut self) -> Self::Output {
        self.validator.visit_i64_trunc_f64_s()?;
        self.add_byte(opcode::I64_TRUNC_F64_S);
        Ok(())
    }

    fn visit_i64_trunc_f64_u(&mut self) -> Self::Output {
        self.validator.visit_i64_trunc_f64_u()?;
        self.add_byte(opcode::I64_TRUNC_F64_U);
        Ok(())
    }

    fn visit_f32_convert_i32_s(&mut self) -> Self::Output {
        self.validator.visit_f32_convert_i32_s()?;
        self.add_byte(opcode::F32_CONVERT_I32_S);
        Ok(())
    }

    fn visit_f32_convert_i32_u(&mut self) -> Self::Output {
        self.validator.visit_f32_convert_i32_u()?;
        self.add_byte(opcode::F32_CONVERT_I32_U);
        Ok(())
    }

    fn visit_f32_convert_i64_s(&mut self) -> Self::Output {
        self.validator.visit_f32_convert_i64_s()?;
        self.add_byte(opcode::F32_CONVERT_I64_S);
        Ok(())
    }

    fn visit_f32_convert_i64_u(&mut self) -> Self::Output {
        self.validator.visit_f32_convert_i64_u()?;
        self.add_byte(opcode::F32_CONVERT_I64_U);
        Ok(())
    }

    fn visit_f32_demote_f64(&mut self) -> Self::Output {
        self.validator.visit_f32_demote_f64()?;
        self.add_byte(opcode::F32_DEMOTE_F64);
        Ok(())
    }

    fn visit_f64_convert_i32_s(&mut self) -> Self::Output {
        self.validator.visit_f64_convert_i32_s()?;
        self.add_byte(opcode::F64_CONVERT_I32_S);
        Ok(())
    }

    fn visit_f64_convert_i32_u(&mut self) -> Self::Output {
        self.validator.visit_f64_convert_i32_u()?;
        self.add_byte(opcode::F64_CONVERT_I32_U);
        Ok(())
    }

    fn visit_f64_convert_i64_s(&mut self) -> Self::Output {
        self.validator.visit_f64_convert_i64_s()?;
        self.add_byte(opcode::F64_CONVERT_I64_S);
        Ok(())
    }

    fn visit_f64_convert_i64_u(&mut self) -> Self::Output {
        self.validator.visit_f64_convert_i64_u()?;
        self.add_byte(opcode::F64_CONVERT_I64_U);
        Ok(())
    }

    fn visit_f64_promote_f32(&mut self) -> Self::Output {
        self.validator.visit_f64_promote_f32()?;
        self.add_byte(opcode::F64_PROMOTE_F32);
        Ok(())
    }

    fn visit_i32_reinterpret_f32(&mut self) -> Self::Output {
        self.validator.visit_i32_reinterpret_f32()?;
        self.add_byte(opcode::I32_REINTERPRET_F32);
        Ok(())
    }

    fn visit_i64_reinterpret_f64(&mut self) -> Self::Output {
        self.validator.visit_i64_reinterpret_f64()?;
        self.add_byte(opcode::I64_REINTERPRET_F64);
        Ok(())
    }

    fn visit_f32_reinterpret_i32(&mut self) -> Self::Output {
        self.validator.visit_f32_reinterpret_i32()?;
        self.add_byte(opcode::F32_REINTERPRET_I32);
        Ok(())
    }

    fn visit_f64_reinterpret_i64(&mut self) -> Self::Output {
        self.validator.visit_f64_reinterpret_i64()?;
        self.add_byte(opcode::F64_REINTERPRET_I64);
        Ok(())
    }

    fn visit_i32_extend8_s(&mut self) -> Self::Output {
        self.validator.visit_i32_extend8_s()?;
        self.add_byte(opcode::I32_EXTEND8_S);
        Ok(())
    }

    fn visit_i32_extend16_s(&mut self) -> Self::Output {
        self.validator.visit_i32_extend16_s()?;
        self.add_byte(opcode::I32_EXTEND16_S);
        Ok(())
    }

    fn visit_i64_extend8_s(&mut self) -> Self::Output {
        self.validator.visit_i64_extend8_s()?;
        self.add_byte(opcode::I64_EXTEND8_S);
        Ok(())
    }

    fn visit_i64_extend16_s(&mut self) -> Self::Output {
        self.validator.visit_i64_extend16_s()?;
        self.add_byte(opcode::I64_EXTEND16_S);
        Ok(())
    }

    fn visit_i64_extend32_s(&mut self) -> Self::Output {
        self.validator.visit_i64_extend32_s()?;
        self.add_byte(opcode::I64_EXTEND32_S);
        Ok(())
    }

    fn visit_ref_null(&mut self, ty: wasm::RefType) -> Self::Output {
        self.validator.visit_ref_null(ty)?;
        self.add_byte(opcode::REF_NULL);
        Ok(())
    }

    fn visit_ref_is_null(&mut self) -> Self::Output {
        self.validator.visit_ref_is_null()?;
        self.add_byte(opcode::REF_IS_NULL);
        Ok(())
    }

    fn visit_ref_func(&mut self) -> Self::Output {
        self.validator.visit_ref_func()?;
        self.add_byte(opcode::REF_FUNC);
        Ok(())
    }

    fn visit_memory_size(&mut self, mem: MemoryIdx) -> Self::Output {
        self.validator.visit_memory_size(mem)?;
        self.add_byte(opcode::MEMORY_SIZE);
        self.add_bytes(&mem.to_ne_bytes());
        Ok(())
    }

    fn visit_memory_grow(&mut self, mem: MemoryIdx) -> Self::Output {
        self.validator.visit_memory_grow(mem)?;
        self.add_byte(opcode::MEMORY_GROW);
        self.add_bytes(&mem.to_ne_bytes());
        Ok(())
    }

    fn visit_memory_copy(&mut self, dst: MemoryIdx, src: MemoryIdx) -> Self::Output {
        self.validator.visit_memory_copy(dst, src)?;
        self.add_byte(0xfc);
        self.add_bytes(&opcode::xfc::MEMORY_COPY.to_ne_bytes());
        self.add_bytes(&dst.to_ne_bytes());
        self.add_bytes(&src.to_ne_bytes());
        Ok(())
    }

    fn visit_memory_fill(&mut self, mem: MemoryIdx) -> Self::Output {
        self.validator.visit_memory_fill(mem)?;
        self.add_byte(0xfc);
        self.add_bytes(&opcode::xfc::MEMORY_FILL.to_ne_bytes());
        self.add_bytes(&mem.to_ne_bytes());
        Ok(())
    }
}

