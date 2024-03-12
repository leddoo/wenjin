use wenjin::{Store, CType, WasmPtr};


#[test]
fn ctype_padding() {
    let mut store = Store::new();

    let mem_id = store.new_memory(Default::default()).unwrap();
    let mut mem = store.memory(mem_id).unwrap();

    mem.grow(1).unwrap();
    assert_eq!(mem.size_bytes(), wasm::PAGE_SIZE);


    #[derive(Clone, Copy, wenjin_derive::CType)]
    #[repr(C)]
    struct Foo {
        a: u8,
        b: u32,
    }


    let mut foos = [
        Foo { a:  42, b: u32::MAX },
        Foo { a: 123, b: u32::from_ne_bytes([0x12, 0x34, 0x56, 0x78]) },
    ];

    unsafe {
        let ptr = &mut foos as *mut _ as *mut u8;
        ptr.add(1).write(1);
        ptr.add(2).write(2);
        ptr.add(3).write(3);
        ptr.add(9).write(4);
        ptr.add(10).write(5);
        ptr.add(11).write(6);


        let bytes: [u8; 16] = core::mem::transmute(foos);
        assert_eq!(bytes, [
             42, 1, 2, 3, 0xff, 0xff, 0xff, 0xff,
            123, 4, 5, 6, 0x12, 0x34, 0x56, 0x78,
        ]);


        let addr = 1;
        mem.write(WasmPtr::new(addr), foos).unwrap();
        let wasm_bytes = mem.read::<[u8; 16]>(WasmPtr::new(addr)).unwrap();
        assert_eq!(wasm_bytes, [
             42, 0, 0, 0, 0xff, 0xff, 0xff, 0xff,
            123, 0, 0, 0, 0x12, 0x34, 0x56, 0x78,
        ]);

        let mut bytes: [u8; 16] = core::mem::transmute(foos);
        <[Foo; 2]>::clear_padding(&mut bytes);
        assert_eq!(bytes, [
             42, 0, 0, 0, 0xff, 0xff, 0xff, 0xff,
            123, 0, 0, 0, 0x12, 0x34, 0x56, 0x78,
        ]);
    }
}
