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

    fn push_args(output: &mut Vec<u8>, args: &[wast::WastArg]) -> Result<(), ()> {
        push_usize(output, args.len());
        for arg in args {
            let wast::WastArg::Core(arg) = arg else { unreachable!() };
            match arg {
                wast::core::WastArgCore::I32(v) => {
                    output.push(0x7f);
                    output.extend_from_slice(&v.to_le_bytes());
                }
                wast::core::WastArgCore::I64(v) => {
                    output.push(0x7e);
                    output.extend_from_slice(&v.to_le_bytes());
                }
                wast::core::WastArgCore::F32(v) => {
                    output.push(0x7d);
                    output.extend_from_slice(&v.bits.to_le_bytes());
                }
                wast::core::WastArgCore::F64(v) => {
                    output.push(0x7c);
                    output.extend_from_slice(&v.bits.to_le_bytes());
                }
                wast::core::WastArgCore::V128(_) => return Err(()),
                wast::core::WastArgCore::RefNull(_) => return Err(()),
                wast::core::WastArgCore::RefExtern(_) => return Err(()),
                wast::core::WastArgCore::RefHost(_) => return Err(()),
            }
        }
        return Ok(());
    }

    fn push_invoke(output: &mut Vec<u8>, invoke: &wast::WastInvoke) -> Result<(), ()> {
        push_bytes(output, invoke.name.as_bytes());
        push_args(output, &invoke.args)
    }

    fn push_rets(output: &mut Vec<u8>, rets: &[wast::WastRet]) -> Result<(), ()> {
        push_usize(output, rets.len());
        for ret in rets {
            let wast::WastRet::Core(ret) = ret else { unreachable!() };
            match ret {
                wast::core::WastRetCore::I32(v) => {
                    output.push(0x7f);
                    output.extend_from_slice(&v.to_le_bytes());
                }
                wast::core::WastRetCore::I64(v) => {
                    output.push(0x7e);
                    output.extend_from_slice(&v.to_le_bytes());
                }
                wast::core::WastRetCore::F32(v) => {
                    output.push(0x7d);
                    let v = match v {
                        wast::core::NanPattern::CanonicalNan => return Err(()),
                        wast::core::NanPattern::ArithmeticNan => return Err(()),
                        wast::core::NanPattern::Value(v) => v,
                    };
                    output.extend_from_slice(&v.bits.to_le_bytes());
                }
                wast::core::WastRetCore::F64(v) => {
                    output.push(0x7c);
                    let v = match v {
                        wast::core::NanPattern::CanonicalNan => return Err(()),
                        wast::core::NanPattern::ArithmeticNan => return Err(()),
                        wast::core::NanPattern::Value(v) => v,
                    };
                    output.extend_from_slice(&v.bits.to_le_bytes());
                }
                wast::core::WastRetCore::V128(_) => return Err(()),
                wast::core::WastRetCore::RefNull(_) => return Err(()),
                wast::core::WastRetCore::RefExtern(_) => return Err(()),
                wast::core::WastRetCore::RefHost(_) => return Err(()),
                wast::core::WastRetCore::RefFunc(_) => return Err(()),
                wast::core::WastRetCore::RefAny => return Err(()),
                wast::core::WastRetCore::RefEq => return Err(()),
                wast::core::WastRetCore::RefArray => return Err(()),
                wast::core::WastRetCore::RefStruct => return Err(()),
                wast::core::WastRetCore::RefI31 => return Err(()),
                wast::core::WastRetCore::Either(_) => return Err(()),
            }
        }
        return Ok(());
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

            WD::Invoke(invoke) => {
                let mut cmd = vec![];
                cmd.push(0x05);
                if push_invoke(&mut cmd, invoke).is_ok() {
                    output.extend_from_slice(&cmd);
                }
            }

            WD::AssertTrap { span: _, exec: _, message: _ } => {
                println!("skip assert trap");
            }

            WD::AssertReturn { span: _, exec, results } => {
                match exec {
                    wast::WastExecute::Invoke(invoke) => {
                        if invoke.module.is_some() {
                            println!("skip assert return (module id)");
                        }
                        else {
                            let mut cmd = vec![];
                            cmd.push(0x07);
                            if push_invoke(&mut cmd, invoke).is_ok()
                            && push_rets(&mut cmd, results).is_ok() {
                                output.extend_from_slice(&cmd);
                            }
                        }
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

