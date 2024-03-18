use sti::reader::Reader;

pub fn dump(code: &[u8]) {
    let mut reader = Reader::new(code);

    let mut indent = 0;

    loop {
        let offset = reader.offset();
        let Some(op) = reader.next() else { break };

        print!("{:#06x}: ", offset);

        match op {
            wasm::opcode::ELSE |
            wasm::opcode::END => indent -= 1,
            _ => (),
        }
        for _ in 0..indent {
            print!("  ");
        }
        match op {
            wasm::opcode::BLOCK |
            wasm::opcode::IF |
            wasm::opcode::ELSE |
            wasm::opcode::LOOP => indent += 1,
            _ => (),
        }

        print!("{}", wasm::opcode::name(op));

        fn imm_i32(reader: &mut Reader<u8>) {
            let v = i32::from_ne_bytes(reader.next_array::<4>().unwrap());
            print!(" {v}");
        }

        fn imm_i64(reader: &mut Reader<u8>) {
            let v = i64::from_ne_bytes(reader.next_array::<8>().unwrap());
            print!(" {v}");
        }

        fn imm_f32(reader: &mut Reader<u8>) {
            let v = f32::from_ne_bytes(reader.next_array::<4>().unwrap());
            print!(" {v}");
        }

        fn imm_f64(reader: &mut Reader<u8>) {
            let v = f64::from_ne_bytes(reader.next_array::<8>().unwrap());
            print!(" {v}");
        }

        fn jump(reader: &mut Reader<u8>) {
            let offset = reader.offset();
            let delta = i32::from_ne_bytes(reader.next_array::<4>().unwrap());
            print!(" {:#06x}", offset as i32 + delta);
        }

        match op {
            wasm::opcode::UNREACHABLE |
            wasm::opcode::NOP |
            wasm::opcode::BLOCK |
            wasm::opcode::LOOP => (),
            wasm::opcode::IF => { jump(&mut reader); }
            wasm::opcode::ELSE => { jump(&mut reader); imm_i32(&mut reader); imm_i32(&mut reader) }
            wasm::opcode::END => {}
            wasm::opcode::BR => { jump(&mut reader); imm_i32(&mut reader); imm_i32(&mut reader) }
            wasm::opcode::BR_IF => { jump(&mut reader); imm_i32(&mut reader); imm_i32(&mut reader) }
            wasm::opcode::BR_TABLE => {
                let n = u32::from_ne_bytes(reader.next_array::<4>().unwrap());
                print!(" [");
                for _ in 0..n { jump(&mut reader); imm_i32(&mut reader); imm_i32(&mut reader); }
                print!(" ]");
                jump(&mut reader); imm_i32(&mut reader); imm_i32(&mut reader);
            }
            wasm::opcode::RETURN => { imm_i32(&mut reader); }
            wasm::opcode::CALL => { imm_i32(&mut reader); }
            wasm::opcode::CALL_INDIRECT => { imm_i32(&mut reader); imm_i32(&mut reader) }
            wasm::opcode::DROP |
            wasm::opcode::SELECT => (),
            wasm::opcode::TYPED_SELECT => unreachable!(),
            wasm::opcode::LOCAL_GET => { imm_i32(&mut reader); }
            wasm::opcode::LOCAL_SET => { imm_i32(&mut reader); }
            wasm::opcode::LOCAL_TEE => { imm_i32(&mut reader); }
            wasm::opcode::GLOBAL_GET => { imm_i32(&mut reader) }
            wasm::opcode::GLOBAL_SET => { imm_i32(&mut reader) }
            wasm::opcode::TABLE_GET => { todo!() }
            wasm::opcode::TABLE_SET => { todo!() }
            wasm::opcode::I32_LOAD => { imm_i32(&mut reader) }
            wasm::opcode::I64_LOAD => { imm_i32(&mut reader) }
            wasm::opcode::F32_LOAD => { imm_i32(&mut reader) }
            wasm::opcode::F64_LOAD => { imm_i32(&mut reader) }
            wasm::opcode::I32_LOAD8_S => { imm_i32(&mut reader) }
            wasm::opcode::I32_LOAD8_U => { imm_i32(&mut reader) }
            wasm::opcode::I32_LOAD16_S => { imm_i32(&mut reader) }
            wasm::opcode::I32_LOAD16_U => { imm_i32(&mut reader) }
            wasm::opcode::I64_LOAD8_S => { imm_i32(&mut reader) }
            wasm::opcode::I64_LOAD8_U => { imm_i32(&mut reader) }
            wasm::opcode::I64_LOAD16_S => { imm_i32(&mut reader) }
            wasm::opcode::I64_LOAD16_U => { imm_i32(&mut reader) }
            wasm::opcode::I64_LOAD32_S => { imm_i32(&mut reader) }
            wasm::opcode::I64_LOAD32_U => { imm_i32(&mut reader) }
            wasm::opcode::I32_STORE => { imm_i32(&mut reader) }
            wasm::opcode::I64_STORE => { imm_i32(&mut reader) }
            wasm::opcode::F32_STORE => { imm_i32(&mut reader) }
            wasm::opcode::F64_STORE => { imm_i32(&mut reader) }
            wasm::opcode::I32_STORE8 => { imm_i32(&mut reader) }
            wasm::opcode::I32_STORE16 => { imm_i32(&mut reader) }
            wasm::opcode::I64_STORE8 => { imm_i32(&mut reader) }
            wasm::opcode::I64_STORE16 => { imm_i32(&mut reader) }
            wasm::opcode::I64_STORE32 => { imm_i32(&mut reader) }
            wasm::opcode::MEMORY_SIZE => { imm_i32(&mut reader) }
            wasm::opcode::MEMORY_GROW => { imm_i32(&mut reader) }
            wasm::opcode::I32_CONST => { imm_i32(&mut reader); }
            wasm::opcode::I64_CONST => { imm_i64(&mut reader); }
            wasm::opcode::F32_CONST => { imm_f32(&mut reader); }
            wasm::opcode::F64_CONST => { imm_f64(&mut reader); }
            wasm::opcode::I32_EQZ |
            wasm::opcode::I32_EQ |
            wasm::opcode::I32_NE |
            wasm::opcode::I32_LT_S |
            wasm::opcode::I32_LT_U |
            wasm::opcode::I32_GT_S |
            wasm::opcode::I32_GT_U |
            wasm::opcode::I32_LE_S |
            wasm::opcode::I32_LE_U |
            wasm::opcode::I32_GE_S |
            wasm::opcode::I32_GE_U |
            wasm::opcode::I64_EQZ |
            wasm::opcode::I64_EQ |
            wasm::opcode::I64_NE |
            wasm::opcode::I64_LT_S |
            wasm::opcode::I64_LT_U |
            wasm::opcode::I64_GT_S |
            wasm::opcode::I64_GT_U |
            wasm::opcode::I64_LE_S |
            wasm::opcode::I64_LE_U |
            wasm::opcode::I64_GE_S |
            wasm::opcode::I64_GE_U |
            wasm::opcode::F32_EQ |
            wasm::opcode::F32_NE |
            wasm::opcode::F32_LT |
            wasm::opcode::F32_GT |
            wasm::opcode::F32_LE |
            wasm::opcode::F32_GE |
            wasm::opcode::F64_EQ |
            wasm::opcode::F64_NE |
            wasm::opcode::F64_LT |
            wasm::opcode::F64_GT |
            wasm::opcode::F64_LE |
            wasm::opcode::F64_GE |
            wasm::opcode::I32_CLZ |
            wasm::opcode::I32_CTZ |
            wasm::opcode::I32_POPCNT |
            wasm::opcode::I32_ADD |
            wasm::opcode::I32_SUB |
            wasm::opcode::I32_MUL |
            wasm::opcode::I32_DIV_S |
            wasm::opcode::I32_DIV_U |
            wasm::opcode::I32_REM_S |
            wasm::opcode::I32_REM_U |
            wasm::opcode::I32_AND |
            wasm::opcode::I32_OR |
            wasm::opcode::I32_XOR |
            wasm::opcode::I32_SHL |
            wasm::opcode::I32_SHR_S |
            wasm::opcode::I32_SHR_U |
            wasm::opcode::I32_ROTL |
            wasm::opcode::I32_ROTR |
            wasm::opcode::I64_CLZ |
            wasm::opcode::I64_CTZ |
            wasm::opcode::I64_POPCNT |
            wasm::opcode::I64_ADD |
            wasm::opcode::I64_SUB |
            wasm::opcode::I64_MUL |
            wasm::opcode::I64_DIV_S |
            wasm::opcode::I64_DIV_U |
            wasm::opcode::I64_REM_S |
            wasm::opcode::I64_REM_U |
            wasm::opcode::I64_AND |
            wasm::opcode::I64_OR |
            wasm::opcode::I64_XOR |
            wasm::opcode::I64_SHL |
            wasm::opcode::I64_SHR_S |
            wasm::opcode::I64_SHR_U |
            wasm::opcode::I64_ROTL |
            wasm::opcode::I64_ROTR |
            wasm::opcode::F32_ABS |
            wasm::opcode::F32_NEG |
            wasm::opcode::F32_CEIL |
            wasm::opcode::F32_FLOOR |
            wasm::opcode::F32_TRUNC |
            wasm::opcode::F32_NEAREST |
            wasm::opcode::F32_SQRT |
            wasm::opcode::F32_ADD |
            wasm::opcode::F32_SUB |
            wasm::opcode::F32_MUL |
            wasm::opcode::F32_DIV |
            wasm::opcode::F32_MIN |
            wasm::opcode::F32_MAX |
            wasm::opcode::F32_COPYSIGN |
            wasm::opcode::F64_ABS |
            wasm::opcode::F64_NEG |
            wasm::opcode::F64_CEIL |
            wasm::opcode::F64_FLOOR |
            wasm::opcode::F64_TRUNC |
            wasm::opcode::F64_NEAREST |
            wasm::opcode::F64_SQRT |
            wasm::opcode::F64_ADD |
            wasm::opcode::F64_SUB |
            wasm::opcode::F64_MUL |
            wasm::opcode::F64_DIV |
            wasm::opcode::F64_MIN |
            wasm::opcode::F64_MAX |
            wasm::opcode::F64_COPYSIGN |
            wasm::opcode::I32_WRAP_I64 |
            wasm::opcode::I32_TRUNC_F32_S |
            wasm::opcode::I32_TRUNC_F32_U |
            wasm::opcode::I32_TRUNC_F64_S |
            wasm::opcode::I32_TRUNC_F64_U |
            wasm::opcode::I64_EXTEND_I32_S |
            wasm::opcode::I64_EXTEND_I32_U |
            wasm::opcode::I64_TRUNC_F32_S |
            wasm::opcode::I64_TRUNC_F32_U |
            wasm::opcode::I64_TRUNC_F64_S |
            wasm::opcode::I64_TRUNC_F64_U |
            wasm::opcode::F32_CONVERT_I32_S |
            wasm::opcode::F32_CONVERT_I32_U |
            wasm::opcode::F32_CONVERT_I64_S |
            wasm::opcode::F32_CONVERT_I64_U |
            wasm::opcode::F32_DEMOTE_F64 |
            wasm::opcode::F64_CONVERT_I32_S |
            wasm::opcode::F64_CONVERT_I32_U |
            wasm::opcode::F64_CONVERT_I64_S |
            wasm::opcode::F64_CONVERT_I64_U |
            wasm::opcode::F64_PROMOTE_F32 |
            wasm::opcode::I32_REINTERPRET_F32 |
            wasm::opcode::I64_REINTERPRET_F64 |
            wasm::opcode::F32_REINTERPRET_I32 |
            wasm::opcode::F64_REINTERPRET_I64 |
            wasm::opcode::I32_EXTEND8_S |
            wasm::opcode::I32_EXTEND16_S |
            wasm::opcode::I64_EXTEND8_S |
            wasm::opcode::I64_EXTEND16_S |
            wasm::opcode::I64_EXTEND32_S |
            wasm::opcode::REF_NULL |
            wasm::opcode::REF_IS_NULL |
            wasm::opcode::REF_FUNC => (),

            0xfc => {
                let op = u32::from_ne_bytes(reader.next_array::<4>().unwrap());
                match op {
                    wasm::opcode::xfc::MEMORY_COPY => { imm_i32(&mut reader); imm_i32(&mut reader) }
                    wasm::opcode::xfc::MEMORY_FILL => { imm_i32(&mut reader) }

                    _ => unreachable!()
                }
            }

            _ => unreachable!()
        }

        println!();
    }

    println!();
}

