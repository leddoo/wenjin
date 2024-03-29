use sti::reader::Reader;
use wenjin::{Store, Value};


fn main() {
    let t0 = std::time::Instant::now();
    let mut input_size = 0;
    let mut module_size = 0;

    let tests = [
        ("address.wast", &include_bytes!("../../testsuite-bin/address.wast")[..]),
        ("align.wast", &include_bytes!("../../testsuite-bin/align.wast")[..]),
        ("binary-leb128.wast", &include_bytes!("../../testsuite-bin/binary-leb128.wast")[..]),
        ("binary.wast", &include_bytes!("../../testsuite-bin/binary.wast")[..]),
        ("block.wast", &include_bytes!("../../testsuite-bin/block.wast")[..]),
        ("br.wast", &include_bytes!("../../testsuite-bin/br.wast")[..]),
        ("br_if.wast", &include_bytes!("../../testsuite-bin/br_if.wast")[..]),
        ("br_table.wast", &include_bytes!("../../testsuite-bin/br_table.wast")[..]),
        //("bulk.wast", &include_bytes!("../../testsuite-bin/bulk.wast")[..]),
        ("call.wast", &include_bytes!("../../testsuite-bin/call.wast")[..]),
        ("call_indirect.wast", &include_bytes!("../../testsuite-bin/call_indirect.wast")[..]),
        ("comments.wast", &include_bytes!("../../testsuite-bin/comments.wast")[..]),
        ("const.wast", &include_bytes!("../../testsuite-bin/const.wast")[..]),
        //("conversions.wast", &include_bytes!("../../testsuite-bin/conversions.wast")[..]),
        ("custom.wast", &include_bytes!("../../testsuite-bin/custom.wast")[..]),
        ("data.wast", &include_bytes!("../../testsuite-bin/data.wast")[..]),
        ("elem.wast", &include_bytes!("../../testsuite-bin/elem.wast")[..]),
        ("endianness.wast", &include_bytes!("../../testsuite-bin/endianness.wast")[..]),
        ("exports.wast", &include_bytes!("../../testsuite-bin/exports.wast")[..]),
        ("extra-loop.wast", &include_bytes!("../../testsuite-bin/extra-loop.wast")[..]),
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
        ("func_ptrs.wast", &include_bytes!("../../testsuite-bin/func_ptrs.wast")[..]),
        ("global.wast", &include_bytes!("../../testsuite-bin/global.wast")[..]),
        ("i32.wast", &include_bytes!("../../testsuite-bin/i32.wast")[..]),
        ("i64.wast", &include_bytes!("../../testsuite-bin/i64.wast")[..]),
        ("if.wast", &include_bytes!("../../testsuite-bin/if.wast")[..]),
        ("imports.wast", &include_bytes!("../../testsuite-bin/imports.wast")[..]),
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
        ("memory_grow.wast", &include_bytes!("../../testsuite-bin/memory_grow.wast")[..]),
        ("memory_init.wast", &include_bytes!("../../testsuite-bin/memory_init.wast")[..]),
        ("memory_redundancy.wast", &include_bytes!("../../testsuite-bin/memory_redundancy.wast")[..]),
        ("memory_size.wast", &include_bytes!("../../testsuite-bin/memory_size.wast")[..]),
        ("memory_trap.wast", &include_bytes!("../../testsuite-bin/memory_trap.wast")[..]),
        ("names.wast", &include_bytes!("../../testsuite-bin/names.wast")[..]),
        ("nop.wast", &include_bytes!("../../testsuite-bin/nop.wast")[..]),
        ("obsolete-keywords.wast", &include_bytes!("../../testsuite-bin/obsolete-keywords.wast")[..]),
        ("ref_func.wast", &include_bytes!("../../testsuite-bin/ref_func.wast")[..]),
        ("ref_is_null.wast", &include_bytes!("../../testsuite-bin/ref_is_null.wast")[..]),
        ("ref_null.wast", &include_bytes!("../../testsuite-bin/ref_null.wast")[..]),
        ("return.wast", &include_bytes!("../../testsuite-bin/return.wast")[..]),
        ("select.wast", &include_bytes!("../../testsuite-bin/select.wast")[..]),
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
        ("table.wast", &include_bytes!("../../testsuite-bin/table.wast")[..]),
        ("table_copy.wast", &include_bytes!("../../testsuite-bin/table_copy.wast")[..]),
        ("table_fill.wast", &include_bytes!("../../testsuite-bin/table_fill.wast")[..]),
        ("table_get.wast", &include_bytes!("../../testsuite-bin/table_get.wast")[..]),
        ("table_grow.wast", &include_bytes!("../../testsuite-bin/table_grow.wast")[..]),
        ("table_init.wast", &include_bytes!("../../testsuite-bin/table_init.wast")[..]),
        ("table_set.wast", &include_bytes!("../../testsuite-bin/table_set.wast")[..]),
        ("table_size.wast", &include_bytes!("../../testsuite-bin/table_size.wast")[..]),
        ("token.wast", &include_bytes!("../../testsuite-bin/token.wast")[..]),
        ("traps.wast", &include_bytes!("../../testsuite-bin/traps.wast")[..]),
        ("type.wast", &include_bytes!("../../testsuite-bin/type.wast")[..]),
        ("unreachable.wast", &include_bytes!("../../testsuite-bin/unreachable.wast")[..]),
        // we don't do stack polymorphism.
        //("unreached-invalid.wast", &include_bytes!("../../testsuite-bin/unreached-invalid.wast")[..]),
        ("unreached-valid.wast", &include_bytes!("../../testsuite-bin/unreached-valid.wast")[..]),
        //("unwind.wast", &include_bytes!("../../testsuite-bin/unwind.wast")[..]),
        ("utf8-custom-section-id.wast", &include_bytes!("../../testsuite-bin/utf8-custom-section-id.wast")[..]),
        ("utf8-import-field.wast", &include_bytes!("../../testsuite-bin/utf8-import-field.wast")[..]),
        ("utf8-import-module.wast", &include_bytes!("../../testsuite-bin/utf8-import-module.wast")[..]),
        ("utf8-invalid-encoding.wast", &include_bytes!("../../testsuite-bin/utf8-invalid-encoding.wast")[..]),
    ];

    let mut num_tests = 0;
    let mut num_successes = 0;
    let mut num_skipped = 0;

    for (name, bytes) in tests {
        println!("#running {name:?}");
        input_size += bytes.len();

        let mut store = Store::new();

        let st_global_i32 = store.new_global(false, Value::I32(666));
        let st_global_i64 = store.new_global(false, Value::I64(666));
        let st_print_i32 = store.new_host_func(|_: i32| ());
        let st_print_i64 = store.new_host_func(|_: i64| ());
        let st_print_f32 = store.new_host_func(|_: f32| ());
        let st_print_f64 = store.new_host_func(|_: f64| ());
        let st_print_i32_f32 = store.new_host_func(|_: i32, _: f32| ());
        let st_print_f64_f64 = store.new_host_func(|_: f64, _: f64| ());

        let imports = &[
            ("spectest", "global_i32", st_global_i32.into()),
            ("spectest", "global_i64", st_global_i64.into()),
            ("spectest", "print_i32", st_print_i32.into()),
            ("spectest", "print_i64", st_print_i64.into()),
            ("spectest", "print_f32", st_print_f32.into()),
            ("spectest", "print_f64", st_print_f64.into()),
            ("spectest", "print_i32_f32", st_print_i32_f32.into()),
            ("spectest", "print_f64_f64", st_print_f64_f64.into()),
        ];

        let mut reader = Reader::new(bytes);

        fn read_usize(reader: &mut Reader<u8>) -> usize {
            u32::from_le_bytes(reader.next_array().unwrap()) as usize
        }

        fn read_bytes<'a>(reader: &mut Reader<'a, u8>) -> &'a [u8] {
            let len = read_usize(reader);
            reader.next_n(len).unwrap()
        }

        fn read_string<'a>(reader: &mut Reader<'a, u8>) -> &'a str {
            core::str::from_utf8(read_bytes(reader)).unwrap()
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

        fn check_error(result: Result<(), wenjin::Error>, message: &str, idx: i32, kind: &str) -> bool {
            let Err(e) = result else {
                println!("failure: module should be {kind} ({idx}) with error {message:?}");
                return false;
            };

            match e {
                wenjin::Error::Wasm(e) => {
                    use wenjin::wasm::ErrorKind as E;
                    match (message, e.kind) {
                        ("i32 constant", E::Leb128Overflow) |
                        ("unexpected end" | "length out of bounds", E::UnexpectedEof) |
                        ("function and code section have inconsistent lengths", E::NumCodesNeNumFuncs) |
                        ("malformed section id", E::InvalidSectionType) |
                        ("malformed mutability", E::InvalidGlobalType) |
                        ("malformed UTF-8 encoding", E::StringNotUtf8) |
                        ("constant expression required", E::InvalidConstExpr) |
                        ("type mismatch", E::InvalidConstExpr) |
                        ("invalid result arity", E::UnsupportedOperator)
                        => {
                            return true;
                        }

                        ("alignment must not be larger than natural", E::AlignTooLarge) |
                        ("type mismatch",
                         E::TypeMismatch { expected: _, found: _ } |
                         E::StackUnderflow |
                         E::FrameExtraStack |
                         E::NonIdIfWithoutElse |
                         E::BrTableInvalidTargetTypes { label: _ } |
                         E::SelectUnexpectedRefType |
                         E::SelectTypeMismatch(_, _) |
                         E::InvalidGlobalInit) |
                        ("unknown label", E::InvalidLabel) |
                        ("unknown type", E::InvalidTypeIdx) |
                        ("unknown function", E::InvalidFuncIdx) |
                        ("unknown table", E::InvalidTableIdx) |
                        ("unknown memory" | "unknown memory 0", E::InvalidMemoryIdx) |
                        ("unknown global", E::InvalidGlobalIdx) |
                        ("unknown local", E::InvalidLocalIdx) |
                        ("global is immutable", E::GlobalNotMutable) |
                        ("constant expression required", E::InvalidGlobalInit)
                        => {
                            return true;
                        }

                        _ => {
                            println!("failure: incorrect error, {kind} {idx}");
                            println!("  {e:?}");
                            println!("  expected {message:?}");
                            return false;
                        }
                    }
                }

                _ => {
                    println!("failure: module should be {kind} ({idx}) with error {message:?}");
                    println!("  but got error {e:?}");
                    return false;
                }
            }
        }

        let mut module_idx = 0;
        let mut malformed_idx = 0;
        let mut invalid_idx = 0;
        let mut instance = None;

        while let Some(op) = reader.next() {
            match op {
                0x01 => {
                    let idx = module_idx;
                    module_idx += 1;
                    println!("module {idx}");

                    let wasm = read_bytes(&mut reader);
                    module_size += wasm.len();

                    num_tests += 1;
                    let inst = store.new_instance(wasm, imports);
                    match inst {
                        Ok(inst) => {
                            num_successes += 1;
                            instance = Some(inst);
                        }
                        Err(e) => {
                            println!("failure: failed to instantiate module {idx}");
                            println!("  error: {e:?}");
                            instance = None;
                        }
                    }
                }

                0x02 => {
                    let idx = malformed_idx;
                    malformed_idx += 1;

                    let wasm = read_bytes(&mut reader);
                    module_size += wasm.len();

                    let message = read_string(&mut reader);

                    if wasm.len() == 0 {
                        println!("skip parse test {idx} ({message:?})");
                        num_skipped += 1;
                        continue;
                    }

                    num_tests += 1;
                    let result = store.new_instance(wasm, imports).map(|_| ());
                    if check_error(result, message, idx, "malformed") {
                        num_successes += 1;
                    }
                }

                0x03 => {
                    let idx = invalid_idx;
                    invalid_idx += 1;

                    let wasm = read_bytes(&mut reader);
                    module_size += wasm.len();

                    let message = read_string(&mut reader);

                    num_tests += 1;
                    let result = store.new_instance(wasm, imports).map(|_| ());
                    if check_error(result, message, idx, "invalid") {
                        num_successes += 1;
                    }
                }

                0x05 => {
                    let name = read_string(&mut reader);

                    let num_args = read_usize(&mut reader);
                    let args = Vec::from_iter((0..num_args).map(|_| { read_value(&mut reader) }));

                    let Some(inst) = instance else {
                        println!("skipping invoke (missing instance)");
                        num_skipped += 1;
                        continue;
                    };
                    let func = store.get_export_func_dyn(inst, name).unwrap();

                    num_tests += 1;
                    if let Err(e) = store.call_dyn(func, &args, &mut []) {
                        println!("failure: invoke {name}({args:?})");
                        println!(" failed with error {e:?}");
                    }
                    else {
                        num_successes += 1;
                    }
                }

                0x06 => {
                    let name = read_string(&mut reader);

                    let num_args = read_usize(&mut reader);
                    let args = Vec::from_iter((0..num_args).map(|_| { read_value(&mut reader) }));

                    let message = read_string(&mut reader);

                    let Some(inst) = instance else {
                        println!("skipping assert_trap (missing instance)");
                        num_skipped += 1;
                        continue;
                    };
                    let func = store.get_export_func_dyn(inst, name).unwrap();

                    num_tests += 1;
                    let result = store.call_dyn_ex(func, &args, &mut [], true);
                    let Err(e) = result else {
                        println!("failure: trap expected for {name}({args:?}) with error {message:?}");
                        continue;
                    };

                    use wenjin::Error as E;
                    match (message, e) {
                        ("unreachable", E::TrapUnreachable) |
                        ("out of bounds memory access", E::TrapMemoryBounds) |
                        ("integer divide by zero", E::TrapDivZero) |
                        ("undefined element", E::TrapTableBounds) |
                        ("indirect call type mismatch", E::TrapCallIndirectTypeMismatch) |
                        ("uninitialized element", E::TrapCallIndirectRefNull)
                        => {
                            num_successes += 1;
                        }

                        _ => {
                            println!("failure: incorrect trap for {name}({args:?})");
                            println!("  {e:?}");
                            println!("  expected {message:?}");
                        }
                    }
                }

                0x07 => {
                    let name = read_string(&mut reader);

                    let num_args = read_usize(&mut reader);
                    let args = Vec::from_iter((0..num_args).map(|_| { read_value(&mut reader) }));

                    let num_rets = read_usize(&mut reader);
                    let rets = Vec::from_iter((0..num_rets).map(|_| { read_value(&mut reader) }));

                    let mut actual_rets = Vec::from_iter((0..num_rets).map(|_| Value::I32(0)));
                    let Some(inst) = instance else {
                        println!("skipping assert_return (missing instance)");
                        num_skipped += 1;
                        continue;
                    };
                    let func = store.get_export_func_dyn(inst, name).unwrap();

                    num_tests += 1;
                    if let Err(e) = store.call_dyn(func, &args, &mut actual_rets) {
                        println!("failure: expected {name}({args:?}) = {rets:?}");
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
                            println!("failure: expected {name}({args:?})");
                            println!(" =   {rets:?}");
                            println!(" got {actual_rets:?}");
                        }
                    }
                }

                _ => unimplemented!()
            }
        }
    }

    let dt = t0.elapsed();
    let num_failed = num_tests - num_successes;
    println!("ran {num_tests} tests, {num_successes} succeeded, {num_failed} failed, {num_skipped} skipped.");
    println!("in {dt:?}, total input size: {input_size}. total size of modules: {module_size}");
}

