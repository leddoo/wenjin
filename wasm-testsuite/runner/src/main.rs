use sti::reader::Reader;
use wenjin::{Store, Value};


fn main() {
    let tests = [
        ("address.wast", &include_bytes!("../../testsuite-bin/address.wast")[..]),
        ("align.wast", &include_bytes!("../../testsuite-bin/align.wast")[..]),
        //("binary-leb128.wast", &include_bytes!("../../testsuite-bin/binary-leb128.wast")[..]),
        //("binary.wast", &include_bytes!("../../testsuite-bin/binary.wast")[..]),
        ("block.wast", &include_bytes!("../../testsuite-bin/block.wast")[..]),
        ("br.wast", &include_bytes!("../../testsuite-bin/br.wast")[..]),
        ("br_if.wast", &include_bytes!("../../testsuite-bin/br_if.wast")[..]),
        //("br_table.wast", &include_bytes!("../../testsuite-bin/br_table.wast")[..]),
        //("bulk.wast", &include_bytes!("../../testsuite-bin/bulk.wast")[..]),
        ("call.wast", &include_bytes!("../../testsuite-bin/call.wast")[..]),
        //("call_indirect.wast", &include_bytes!("../../testsuite-bin/call_indirect.wast")[..]),
        ("comments.wast", &include_bytes!("../../testsuite-bin/comments.wast")[..]),
        ("const.wast", &include_bytes!("../../testsuite-bin/const.wast")[..]),
        //("conversions.wast", &include_bytes!("../../testsuite-bin/conversions.wast")[..]),
        ("custom.wast", &include_bytes!("../../testsuite-bin/custom.wast")[..]),
        //("data.wast", &include_bytes!("../../testsuite-bin/data.wast")[..]),
        //("elem.wast", &include_bytes!("../../testsuite-bin/elem.wast")[..]),
        ("endianness.wast", &include_bytes!("../../testsuite-bin/endianness.wast")[..]),
        ("exports.wast", &include_bytes!("../../testsuite-bin/exports.wast")[..]),
        ("f32.wast", &include_bytes!("../../testsuite-bin/f32.wast")[..]),
        ("f32_bitwise.wast", &include_bytes!("../../testsuite-bin/f32_bitwise.wast")[..]),
        ("f32_cmp.wast", &include_bytes!("../../testsuite-bin/f32_cmp.wast")[..]),
        ("f64.wast", &include_bytes!("../../testsuite-bin/f64.wast")[..]),
        ("f64_bitwise.wast", &include_bytes!("../../testsuite-bin/f64_bitwise.wast")[..]),
        ("f64_cmp.wast", &include_bytes!("../../testsuite-bin/f64_cmp.wast")[..]),
        ("fac.wast", &include_bytes!("../../testsuite-bin/fac.wast")[..]),
        ("float_exprs.wast", &include_bytes!("../../testsuite-bin/float_exprs.wast")[..]),
        ("float_literals.wast", &include_bytes!("../../testsuite-bin/float_literals.wast")[..]),
        ("float_memory.wast", &include_bytes!("../../testsuite-bin/float_memory.wast")[..]),
        ("float_misc.wast", &include_bytes!("../../testsuite-bin/float_misc.wast")[..]),
        ("forward.wast", &include_bytes!("../../testsuite-bin/forward.wast")[..]),
        ("func.wast", &include_bytes!("../../testsuite-bin/func.wast")[..]),
        //("func_ptrs.wast", &include_bytes!("../../testsuite-bin/func_ptrs.wast")[..]),
        //("global.wast", &include_bytes!("../../testsuite-bin/global.wast")[..]),
        ("i32.wast", &include_bytes!("../../testsuite-bin/i32.wast")[..]),
        ("i64.wast", &include_bytes!("../../testsuite-bin/i64.wast")[..]),
        ("if.wast", &include_bytes!("../../testsuite-bin/if.wast")[..]),
        //("imports.wast", &include_bytes!("../../testsuite-bin/imports.wast")[..]),
        ("inline-module.wast", &include_bytes!("../../testsuite-bin/inline-module.wast")[..]),
        ("int_exprs.wast", &include_bytes!("../../testsuite-bin/int_exprs.wast")[..]),
        ("int_literals.wast", &include_bytes!("../../testsuite-bin/int_literals.wast")[..]),
        ("labels.wast", &include_bytes!("../../testsuite-bin/labels.wast")[..]),
        ("left-to-right.wast", &include_bytes!("../../testsuite-bin/left-to-right.wast")[..]),
        //("linking.wast", &include_bytes!("../../testsuite-bin/linking.wast")[..]),
        ("load.wast", &include_bytes!("../../testsuite-bin/load.wast")[..]),
        ("local_get.wast", &include_bytes!("../../testsuite-bin/local_get.wast")[..]),
        ("local_set.wast", &include_bytes!("../../testsuite-bin/local_set.wast")[..]),
        ("local_tee.wast", &include_bytes!("../../testsuite-bin/local_tee.wast")[..]),
        ("loop.wast", &include_bytes!("../../testsuite-bin/loop.wast")[..]),
        ("memory.wast", &include_bytes!("../../testsuite-bin/memory.wast")[..]),
        ("memory_copy.wast", &include_bytes!("../../testsuite-bin/memory_copy.wast")[..]),
        ("memory_fill.wast", &include_bytes!("../../testsuite-bin/memory_fill.wast")[..]),
        //("memory_grow.wast", &include_bytes!("../../testsuite-bin/memory_grow.wast")[..]),
        //("memory_init.wast", &include_bytes!("../../testsuite-bin/memory_init.wast")[..]),
        //("memory_redundancy.wast", &include_bytes!("../../testsuite-bin/memory_redundancy.wast")[..]),
        //("memory_size.wast", &include_bytes!("../../testsuite-bin/memory_size.wast")[..]),
        //("memory_trap.wast", &include_bytes!("../../testsuite-bin/memory_trap.wast")[..]),
        ("names.wast", &include_bytes!("../../testsuite-bin/names.wast")[..]),
        ("nop.wast", &include_bytes!("../../testsuite-bin/nop.wast")[..]),
        //("obsolete-keywords.wast", &include_bytes!("../../testsuite-bin/obsolete-keywords.wast")[..]),
        //("ref_func.wast", &include_bytes!("../../testsuite-bin/ref_func.wast")[..]),
        //("ref_is_null.wast", &include_bytes!("../../testsuite-bin/ref_is_null.wast")[..]),
        //("ref_null.wast", &include_bytes!("../../testsuite-bin/ref_null.wast")[..]),
        ("return.wast", &include_bytes!("../../testsuite-bin/return.wast")[..]),
        //("select.wast", &include_bytes!("../../testsuite-bin/select.wast")[..]),
        /*
        ("simd_address.wast", &include_bytes!("../../testsuite-bin/simd_address.wast")[..]),
        ("simd_align.wast", &include_bytes!("../../testsuite-bin/simd_align.wast")[..]),
        ("simd_bit_shift.wast", &include_bytes!("../../testsuite-bin/simd_bit_shift.wast")[..]),
        ("simd_bitwise.wast", &include_bytes!("../../testsuite-bin/simd_bitwise.wast")[..]),
        ("simd_boolean.wast", &include_bytes!("../../testsuite-bin/simd_boolean.wast")[..]),
        ("simd_const.wast", &include_bytes!("../../testsuite-bin/simd_const.wast")[..]),
        ("simd_conversions.wast", &include_bytes!("../../testsuite-bin/simd_conversions.wast")[..]),
        ("simd_f32x4.wast", &include_bytes!("../../testsuite-bin/simd_f32x4.wast")[..]),
        ("simd_f32x4_arith.wast", &include_bytes!("../../testsuite-bin/simd_f32x4_arith.wast")[..]),
        ("simd_f32x4_cmp.wast", &include_bytes!("../../testsuite-bin/simd_f32x4_cmp.wast")[..]),
        ("simd_f32x4_pmin_pmax.wast", &include_bytes!("../../testsuite-bin/simd_f32x4_pmin_pmax.wast")[..]),
        ("simd_f32x4_rounding.wast", &include_bytes!("../../testsuite-bin/simd_f32x4_rounding.wast")[..]),
        ("simd_f64x2.wast", &include_bytes!("../../testsuite-bin/simd_f64x2.wast")[..]),
        ("simd_f64x2_arith.wast", &include_bytes!("../../testsuite-bin/simd_f64x2_arith.wast")[..]),
        ("simd_f64x2_cmp.wast", &include_bytes!("../../testsuite-bin/simd_f64x2_cmp.wast")[..]),
        ("simd_f64x2_pmin_pmax.wast", &include_bytes!("../../testsuite-bin/simd_f64x2_pmin_pmax.wast")[..]),
        ("simd_f64x2_rounding.wast", &include_bytes!("../../testsuite-bin/simd_f64x2_rounding.wast")[..]),
        ("simd_i16x8_arith.wast", &include_bytes!("../../testsuite-bin/simd_i16x8_arith.wast")[..]),
        ("simd_i16x8_arith2.wast", &include_bytes!("../../testsuite-bin/simd_i16x8_arith2.wast")[..]),
        ("simd_i16x8_cmp.wast", &include_bytes!("../../testsuite-bin/simd_i16x8_cmp.wast")[..]),
        ("simd_i16x8_extadd_pairwise_i8x16.wast", &include_bytes!("../../testsuite-bin/simd_i16x8_extadd_pairwise_i8x16.wast")[..]),
        ("simd_i16x8_extmul_i8x16.wast", &include_bytes!("../../testsuite-bin/simd_i16x8_extmul_i8x16.wast")[..]),
        ("simd_i16x8_q15mulr_sat_s.wast", &include_bytes!("../../testsuite-bin/simd_i16x8_q15mulr_sat_s.wast")[..]),
        ("simd_i16x8_sat_arith.wast", &include_bytes!("../../testsuite-bin/simd_i16x8_sat_arith.wast")[..]),
        ("simd_i32x4_arith.wast", &include_bytes!("../../testsuite-bin/simd_i32x4_arith.wast")[..]),
        ("simd_i32x4_arith2.wast", &include_bytes!("../../testsuite-bin/simd_i32x4_arith2.wast")[..]),
        ("simd_i32x4_cmp.wast", &include_bytes!("../../testsuite-bin/simd_i32x4_cmp.wast")[..]),
        ("simd_i32x4_dot_i16x8.wast", &include_bytes!("../../testsuite-bin/simd_i32x4_dot_i16x8.wast")[..]),
        ("simd_i32x4_extadd_pairwise_i16x8.wast", &include_bytes!("../../testsuite-bin/simd_i32x4_extadd_pairwise_i16x8.wast")[..]),
        ("simd_i32x4_extmul_i16x8.wast", &include_bytes!("../../testsuite-bin/simd_i32x4_extmul_i16x8.wast")[..]),
        ("simd_i32x4_trunc_sat_f32x4.wast", &include_bytes!("../../testsuite-bin/simd_i32x4_trunc_sat_f32x4.wast")[..]),
        ("simd_i32x4_trunc_sat_f64x2.wast", &include_bytes!("../../testsuite-bin/simd_i32x4_trunc_sat_f64x2.wast")[..]),
        ("simd_i64x2_arith.wast", &include_bytes!("../../testsuite-bin/simd_i64x2_arith.wast")[..]),
        ("simd_i64x2_arith2.wast", &include_bytes!("../../testsuite-bin/simd_i64x2_arith2.wast")[..]),
        ("simd_i64x2_cmp.wast", &include_bytes!("../../testsuite-bin/simd_i64x2_cmp.wast")[..]),
        ("simd_i64x2_extmul_i32x4.wast", &include_bytes!("../../testsuite-bin/simd_i64x2_extmul_i32x4.wast")[..]),
        ("simd_i8x16_arith.wast", &include_bytes!("../../testsuite-bin/simd_i8x16_arith.wast")[..]),
        ("simd_i8x16_arith2.wast", &include_bytes!("../../testsuite-bin/simd_i8x16_arith2.wast")[..]),
        ("simd_i8x16_cmp.wast", &include_bytes!("../../testsuite-bin/simd_i8x16_cmp.wast")[..]),
        ("simd_i8x16_sat_arith.wast", &include_bytes!("../../testsuite-bin/simd_i8x16_sat_arith.wast")[..]),
        ("simd_int_to_int_extend.wast", &include_bytes!("../../testsuite-bin/simd_int_to_int_extend.wast")[..]),
        ("simd_lane.wast", &include_bytes!("../../testsuite-bin/simd_lane.wast")[..]),
        ("simd_linking.wast", &include_bytes!("../../testsuite-bin/simd_linking.wast")[..]),
        ("simd_load.wast", &include_bytes!("../../testsuite-bin/simd_load.wast")[..]),
        ("simd_load16_lane.wast", &include_bytes!("../../testsuite-bin/simd_load16_lane.wast")[..]),
        ("simd_load32_lane.wast", &include_bytes!("../../testsuite-bin/simd_load32_lane.wast")[..]),
        ("simd_load64_lane.wast", &include_bytes!("../../testsuite-bin/simd_load64_lane.wast")[..]),
        ("simd_load8_lane.wast", &include_bytes!("../../testsuite-bin/simd_load8_lane.wast")[..]),
        ("simd_load_extend.wast", &include_bytes!("../../testsuite-bin/simd_load_extend.wast")[..]),
        ("simd_load_splat.wast", &include_bytes!("../../testsuite-bin/simd_load_splat.wast")[..]),
        ("simd_load_zero.wast", &include_bytes!("../../testsuite-bin/simd_load_zero.wast")[..]),
        ("simd_splat.wast", &include_bytes!("../../testsuite-bin/simd_splat.wast")[..]),
        ("simd_store.wast", &include_bytes!("../../testsuite-bin/simd_store.wast")[..]),
        ("simd_store16_lane.wast", &include_bytes!("../../testsuite-bin/simd_store16_lane.wast")[..]),
        ("simd_store32_lane.wast", &include_bytes!("../../testsuite-bin/simd_store32_lane.wast")[..]),
        ("simd_store64_lane.wast", &include_bytes!("../../testsuite-bin/simd_store64_lane.wast")[..]),
        ("simd_store8_lane.wast", &include_bytes!("../../testsuite-bin/simd_store8_lane.wast")[..]),
        */
        //("skip-stack-guard-page.wast", &include_bytes!("../../testsuite-bin/skip-stack-guard-page.wast")[..]),
        ("stack.wast", &include_bytes!("../../testsuite-bin/stack.wast")[..]),
        //("start.wast", &include_bytes!("../../testsuite-bin/start.wast")[..]),
        ("store.wast", &include_bytes!("../../testsuite-bin/store.wast")[..]),
        ("switch.wast", &include_bytes!("../../testsuite-bin/switch.wast")[..]),
        ("table-sub.wast", &include_bytes!("../../testsuite-bin/table-sub.wast")[..]),
        //("table.wast", &include_bytes!("../../testsuite-bin/table.wast")[..]),
        //("table_copy.wast", &include_bytes!("../../testsuite-bin/table_copy.wast")[..]),
        //("table_fill.wast", &include_bytes!("../../testsuite-bin/table_fill.wast")[..]),
        //("table_get.wast", &include_bytes!("../../testsuite-bin/table_get.wast")[..]),
        //("table_grow.wast", &include_bytes!("../../testsuite-bin/table_grow.wast")[..]),
        //("table_init.wast", &include_bytes!("../../testsuite-bin/table_init.wast")[..]),
        //("table_set.wast", &include_bytes!("../../testsuite-bin/table_set.wast")[..]),
        //("table_size.wast", &include_bytes!("../../testsuite-bin/table_size.wast")[..]),
        //("token.wast", &include_bytes!("../../testsuite-bin/token.wast")[..]),
        ("traps.wast", &include_bytes!("../../testsuite-bin/traps.wast")[..]),
        ("type.wast", &include_bytes!("../../testsuite-bin/type.wast")[..]),
        ("unreachable.wast", &include_bytes!("../../testsuite-bin/unreachable.wast")[..]),
        ("unreached-invalid.wast", &include_bytes!("../../testsuite-bin/unreached-invalid.wast")[..]),
        //("unreached-valid.wast", &include_bytes!("../../testsuite-bin/unreached-valid.wast")[..]),
        //("unwind.wast", &include_bytes!("../../testsuite-bin/unwind.wast")[..]),
        ("utf8-custom-section-id.wast", &include_bytes!("../../testsuite-bin/utf8-custom-section-id.wast")[..]),
        ("utf8-import-field.wast", &include_bytes!("../../testsuite-bin/utf8-import-field.wast")[..]),
        ("utf8-import-module.wast", &include_bytes!("../../testsuite-bin/utf8-import-module.wast")[..]),
        ("utf8-invalid-encoding.wast", &include_bytes!("../../testsuite-bin/utf8-invalid-encoding.wast")[..]),
    ];

    let mut num_tests = 0;
    let mut num_successes = 0;

    for (name, bytes) in tests {
        println!("#running {name:?}");

        let mut store = Store::new();

        let mut reader = Reader::new(bytes);

        fn read_usize(reader: &mut Reader<u8>) -> usize {
            u32::from_le_bytes(reader.next_array().unwrap()) as usize
        }

        fn read_bytes<'a>(reader: &mut Reader<'a, u8>) -> &'a [u8] {
            let len = read_usize(reader);
            reader.next_n(len).unwrap()
        }

        fn read_value(reader: &mut Reader<u8>) -> Value {
            match reader.next().unwrap() {
                0x7f => Value::I32(i32::from_le_bytes(reader.next_array().unwrap())),
                0x7e => Value::I64(i64::from_le_bytes(reader.next_array().unwrap())),
                0x7d => Value::F32(f32::from_le_bytes(reader.next_array().unwrap())),
                0x7c => Value::F64(f64::from_le_bytes(reader.next_array().unwrap())),
                _ => unimplemented!()
            }
        }

        let mut module_idx = 0;
        let mut instance = None;

        while let Some(op) = reader.next() {
            match op {
                0x01 => {
                    println!("module {module_idx}");
                    module_idx += 1;

                    let wasm = read_bytes(&mut reader);
                    let module = store.new_module(wasm).unwrap();
                    let inst = store.new_instance(module, &[]).unwrap();
                    instance = Some(inst);
                }

                0x05 => {
                    let name = core::str::from_utf8(read_bytes(&mut reader)).unwrap();

                    let num_args = read_usize(&mut reader);
                    let args = Vec::from_iter((0..num_args).map(|_| { read_value(&mut reader) }));

                    let inst = instance.unwrap();
                    let func = store.get_export_func_dyn(inst, name).unwrap();

                    num_tests += 1;
                    if let Err(e) = store.call_dyn(func, &args, &mut []) {
                        println!("invoke {name}({args:?})");
                        println!(" failed with error {e:?}");
                    }
                    else {
                        num_successes += 1;
                    }
                }

                0x07 => {
                    let name = core::str::from_utf8(read_bytes(&mut reader)).unwrap();

                    let num_args = read_usize(&mut reader);
                    let args = Vec::from_iter((0..num_args).map(|_| { read_value(&mut reader) }));

                    let num_rets = read_usize(&mut reader);
                    let rets = Vec::from_iter((0..num_rets).map(|_| { read_value(&mut reader) }));

                    let mut actual_rets = Vec::from_iter((0..num_rets).map(|_| Value::I32(0)));
                    let inst = instance.unwrap();
                    let func = store.get_export_func_dyn(inst, name).unwrap();

                    num_tests += 1;
                    if let Err(e) = store.call_dyn(func, &args, &mut actual_rets) {
                        println!("expected {name}({args:?}) = {rets:?}");
                        println!(" failed with error {e:?}");
                    }
                    else {
                        let mut equal = true;
                        assert_eq!(rets.len(), actual_rets.len());
                        for i in 0..rets.len() {
                            let ok = match (rets[i], actual_rets[i]) {
                                (Value::I32(a), Value::I32(b)) => a == b,
                                (Value::I64(a), Value::I64(b)) => a == b,
                                (Value::F32(a), Value::F32(b)) => a.to_bits() == b.to_bits(),
                                (Value::F64(a), Value::F64(b)) => a.to_bits() == b.to_bits(),
                                _ => false,
                            };
                            equal = equal && ok;
                        }

                        if equal {
                            num_successes += 1;
                        }
                        else {
                            println!("expected {name}({args:?})\n =   {rets:?}\n got {actual_rets:?}");
                        }
                    }
                }

                _ => unimplemented!()
            }
        }
    }

    println!("ran {num_tests} tests, {num_successes} succeeded, {} failed", num_tests - num_successes);
}

