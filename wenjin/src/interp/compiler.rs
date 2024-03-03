use sti::manual_vec::ManualVec;

use wasm::{ValueType, BlockType, TypeIdx, FuncIdx, TableIdx, MemoryIdx, GlobalIdx, opcode};


struct Compiler {
    pub code_limit: u32,

    code: ManualVec<u8>,
    code_size: u32,
}

impl Compiler {
    #[inline]
    fn push_byte(&mut self, byte: u8) {
        if self.code_size < self.code_limit {
            self.code_size += 1;
            _ = self.code.push_or_alloc(byte);
        }
    }

    #[inline]
    fn push_bytes(&mut self, bytes: &[u8]) {
        if bytes.len() <= (self.code_limit - self.code_size) as usize {
            self.code_size += bytes.len() as u32;
            _ = self.code.extend_from_slice_or_alloc(bytes);
        }
        else {
            self.code_size = self.code_limit;
        }
    }
}

impl wasm::OperatorVisitor for Compiler {
    type Output = ();

    fn visit_unreachable(&mut self) -> Self::Output {
        self.push_byte(opcode::UNREACHABLE);
    }

    fn visit_nop(&mut self) -> Self::Output {
    }

    fn visit_block(&mut self, _ty: BlockType) -> Self::Output {
        //self.expect_n(self.block_begin_types(ty))?;
        //self.push_frame(ControlFrameKind::Block, ty)
        todo!()
    }

    fn visit_loop(&mut self, _ty: BlockType) -> Self::Output {
        //self.expect_n(self.block_begin_types(ty))?;
        //self.push_frame(ControlFrameKind::Loop, ty)
        todo!()
    }

    fn visit_if(&mut self, _ty: BlockType) -> Self::Output {
        //self.expect(ValueType::I32)?;
        //self.expect_n(self.block_begin_types(ty))?;
        //self.push_frame(ControlFrameKind::If, ty)
        todo!()
    }

    fn visit_else(&mut self) -> Self::Output {
        //let frame = self.pop_frame()?;
        //if frame.kind != ControlFrameKind::If {
            //todo!()
        //}
        //self.push_frame(ControlFrameKind::Else, frame.ty)
        todo!()
    }

    fn visit_end(&mut self) -> Self::Output {
        /*
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
        */
        todo!()
    }

    fn visit_br(&mut self, label: u32) -> Self::Output {
        let _ = label;
        /*
        let frame = self.label(label)?;
        self.expect_n(self.frame_br_types(&frame))?;
        self.unreachable();
        return Ok(());
        */
        todo!()
    }

    fn visit_br_if(&mut self, label: u32) -> Self::Output {
        let _ = label;
        /*
        let frame = self.label(label)?;
        self.expect(ValueType::I32)?;
        self.expect_n(self.frame_br_types(&frame))?;
        self.push_n(self.frame_br_types(&frame))
        */
        todo!()
    }

    fn visit_br_table(&mut self, table: ()) -> Self::Output {
        let _ = table;
        /*
        let _ = table;
        self.expect(ValueType::I32)?;
        // @todo: validate br targets.
        self.unreachable();
        return Ok(());
        */
        todo!()
    }

    fn visit_return(&mut self) -> Self::Output {
        /*
        let frame = self.frames[0];
        self.expect_n(self.block_end_types(frame.ty))?;
        self.unreachable();
        return Ok(());
        */
        todo!()
    }

    fn visit_call(&mut self, func: FuncIdx) -> Self::Output {
        let _ = func;
        /*
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
        */
        todo!()
    }

    fn visit_call_indirect(&mut self, ty: TypeIdx, table: TableIdx) -> Self::Output {
        let _ = (ty, table);
        /*
        let table = self.table(table)?;
        if table.ty != RefType::FuncRef {
            todo!()
        }

        let ty = self.ty(ty)?;
        self.expect(ValueType::I32)?;
        self.expect_n(ty.params)?;
        self.push_n(ty.rets)
        */
        todo!()
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

