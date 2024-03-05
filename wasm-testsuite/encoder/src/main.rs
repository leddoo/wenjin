fn convert(wast: &str) -> (Vec<u8>, Vec<Vec<u8>>) {
    let pb = &wast::parser::ParseBuffer::new(wast).unwrap();
    let Ok(mut wast) = wast::parser::parse::<wast::Wast>(pb) else {
        return (vec![], vec![]);
    };


    let mut output = vec![];
    let mut modules = vec![];

    fn push_usize(output: &mut Vec<u8>, v: usize) {
        output.extend_from_slice(&u32::try_from(v).unwrap().to_le_bytes());
    }

    fn push_bytes(output: &mut Vec<u8>, bytes: &[u8]) {
        push_usize(output, bytes.len());
        output.extend_from_slice(bytes);
    }

    for op in &mut wast.directives {
        use wast::WastDirective as WD;

        match op {
            WD::Wat(wat) => {
                let wasm = wat.encode().unwrap();
                output.push(0x01);
                push_bytes(&mut output, &wasm);
                modules.push(wasm);
            }

            WD::AssertMalformed { span: _, module: _, message: _ } => {
                println!("skip assert malformed");
            }

            WD::AssertInvalid { span: _, module: _, message: _ } => {
                println!("skip assert invalid");
            }

            WD::Register { span: _, name: _, module: _ } => {
                println!("skip register");
            }

            WD::Invoke(_) => {
                println!("skip invoke");
            }

            WD::AssertTrap { span: _, exec: _, message: _ } => {
                println!("skip assert trap");
            }

            WD::AssertReturn { span: _, exec, results } => {
                match exec {
                    wast::WastExecute::Invoke(invoke) => 'err: {
                        if invoke.module.is_some() {
                            println!("skip assert return (module id)");
                            break 'err;
                        }

                        let mut cmd = vec![];

                        cmd.push(0x07);
                        push_bytes(&mut cmd, invoke.name.as_bytes());

                        push_usize(&mut cmd, invoke.args.len());
                        for arg in &invoke.args {
                            let wast::WastArg::Core(arg) = arg else { unreachable!() };
                            match arg {
                                wast::core::WastArgCore::I32(v) => {
                                    cmd.push(0x7f);
                                    cmd.extend_from_slice(&v.to_le_bytes());
                                }
                                wast::core::WastArgCore::I64(v) => {
                                    cmd.push(0x7e);
                                    cmd.extend_from_slice(&v.to_le_bytes());
                                }
                                wast::core::WastArgCore::F32(v) => {
                                    cmd.push(0x7d);
                                    cmd.extend_from_slice(&v.bits.to_le_bytes());
                                }
                                wast::core::WastArgCore::F64(v) => {
                                    cmd.push(0x7c);
                                    cmd.extend_from_slice(&v.bits.to_le_bytes());
                                }
                                wast::core::WastArgCore::V128(_) => break 'err,
                                wast::core::WastArgCore::RefNull(_) => break 'err,
                                wast::core::WastArgCore::RefExtern(_) => break 'err,
                                wast::core::WastArgCore::RefHost(_) => break 'err,
                            }
                        }

                        push_usize(&mut cmd, results.len());
                        for ret in results {
                            let wast::WastRet::Core(ret) = ret else { unreachable!() };
                            match ret {
                                wast::core::WastRetCore::I32(v) => {
                                    cmd.push(0x7f);
                                    cmd.extend_from_slice(&v.to_le_bytes());
                                }
                                wast::core::WastRetCore::I64(v) => {
                                    cmd.push(0x7e);
                                    cmd.extend_from_slice(&v.to_le_bytes());
                                }
                                wast::core::WastRetCore::F32(v) => {
                                    cmd.push(0x7d);
                                    let v = match v {
                                        wast::core::NanPattern::CanonicalNan => break 'err,
                                        wast::core::NanPattern::ArithmeticNan => break 'err,
                                        wast::core::NanPattern::Value(v) => v,
                                    };
                                    cmd.extend_from_slice(&v.bits.to_le_bytes());
                                }
                                wast::core::WastRetCore::F64(v) => {
                                    cmd.push(0x7c);
                                    let v = match v {
                                        wast::core::NanPattern::CanonicalNan => break 'err,
                                        wast::core::NanPattern::ArithmeticNan => break 'err,
                                        wast::core::NanPattern::Value(v) => v,
                                    };
                                    cmd.extend_from_slice(&v.bits.to_le_bytes());
                                }
                                wast::core::WastRetCore::V128(_) => break 'err,
                                wast::core::WastRetCore::RefNull(_) => break 'err,
                                wast::core::WastRetCore::RefExtern(_) => break 'err,
                                wast::core::WastRetCore::RefHost(_) => break 'err,
                                wast::core::WastRetCore::RefFunc(_) => break 'err,
                                wast::core::WastRetCore::RefAny => break 'err,
                                wast::core::WastRetCore::RefEq => break 'err,
                                wast::core::WastRetCore::RefArray => break 'err,
                                wast::core::WastRetCore::RefStruct => break 'err,
                                wast::core::WastRetCore::RefI31 => break 'err,
                                wast::core::WastRetCore::Either(_) => break 'err,
                            }
                        }

                        output.extend_from_slice(&cmd);
                    }

                    wast::WastExecute::Wat(_) => {
                        println!("skip assert return (wat)");
                        continue;
                    }

                    wast::WastExecute::Get { module: _, global: _ } => {
                        println!("skip assert return (global)");
                        continue;
                    }
                }
            }

            WD::AssertExhaustion { span: _, call: _, message: _ } => {
                println!("skip assert exhaustion");
            }

            WD::AssertUnlinkable { span: _, module: _, message: _ } => {
                println!("skip assert unlinkable");
            }

            WD::AssertException { span: _, exec: _ } => {
                println!("skip assert exception");
            }

            WD::Thread(_) => {
                println!("skip thread");
            }

            WD::Wait { span: _, thread: _ } => {
                println!("skip wait");
            }
        }
    }

    return (output, modules);
}

fn main() {
    for path in std::fs::read_dir("./testsuite").unwrap() {
        let path = path.unwrap().path();
        if path.is_dir() { continue }
        let Some(extension) = path.extension() else { continue };
        if extension.to_str().unwrap() != "wast" { continue }
        let wast = std::fs::read_to_string(&path).unwrap();
        let (bin, modules) = convert(&wast);
        std::fs::write(&format!("./testsuite-bin/{}", path.file_name().unwrap().to_str().unwrap()), bin).unwrap();
        for (i, module) in modules.iter().enumerate() {
            std::fs::write(&format!("./testsuite-bin/{}-module-{i}.wasm", path.file_name().unwrap().to_str().unwrap()), module).unwrap();
        }
    }
}

