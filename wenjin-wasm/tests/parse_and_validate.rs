use sti::arena::Arena;

use wenjin_wasm::*;


fn parse_and_validate<'a>(wasm: &'a [u8], alloc: &'a Arena) -> Module<'a> {
    let module = Parser::parse_module(wasm, ModuleLimits::DEFAULT, &alloc).unwrap();

    let mut validator = Validator::new(&module);
    for (i, code) in module.codes.iter().enumerate() {
        let mut p = Parser::from_sub_section(wasm, code.expr);

        //println!("func {}", i + module.imports.funcs.len());
        let mut jumps = Default::default();
        validator.validate_func(&mut p, module.funcs[i], code.locals, Some(&mut jumps)).unwrap();
        //dbg!(&jumps);
    }

    return module;
}

#[test]
fn lua_wasm() {
    let wasm = include_bytes!("lua.wasm");
    let alloc = Arena::new();
    let module = parse_and_validate(wasm, &alloc);
    /*
         Type start=0x0000000b end=0x00000113 (size=0x00000108) count: 38
       Import start=0x00000116 end=0x00000360 (size=0x0000024a) count: 27
     Function start=0x00000363 end=0x00000529 (size=0x000001c6) count: 452
        Table start=0x0000052b end=0x00000532 (size=0x00000007) count: 1
       Memory start=0x00000534 end=0x00000537 (size=0x00000003) count: 1
       Global start=0x00000539 end=0x00000541 (size=0x00000008) count: 1
       Export start=0x00000544 end=0x000005ca (size=0x00000086) count: 11
         Elem start=0x000005cd end=0x000006ea (size=0x0000011d) count: 1
         Code start=0x000006ee end=0x000556f7 (size=0x00055009) count: 452
         Data start=0x000556fa end=0x00059252 (size=0x00003b58) count: 2
       Custom start=0x00059255 end=0x0005ae3a (size=0x00001be5) "name"
       Custom start=0x0005ae3c end=0x0005ae6b (size=0x0000002f) "producers"
       Custom start=0x0005ae6d end=0x0005ae99 (size=0x0000002c) "target_features"
    */
    assert_eq!(module.types.len(), 38);
    assert_eq!(module.imports.imports.len(), 27);
    assert_eq!(module.funcs.len(), 452);
    assert_eq!(module.tables.len(), 1);
    assert_eq!(module.memories.len(), 1);
    assert_eq!(module.globals.len(), 1);
    assert_eq!(module.exports.len(), 11);
    assert_eq!(module.elements.len(), 1);
    assert_eq!(module.codes.len(), 452);
    assert_eq!(module.datas.len(), 2);
    assert_eq!(module.customs.len(), 3);
}


