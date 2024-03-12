use wenjin::{Store, WasmSize, WasmPtr, Error};


#[test]
fn malloc() {

    #[derive(Clone, Copy, wenjin_derive::CType)]
    #[repr(C)]
    struct BlockHeader {
        size: WasmSize,
        next: WasmPtr<BlockHeader>,
        free: u8,
        hash: u32,
    }

    const ALIGN: usize = 8;

    impl BlockHeader {
        // @temp
        //const SIZE: u32 = sti::num::ceil_to_multiple_pow2(core::mem::size_of::<BlockHeader>(), ALIGN) as u32;
        const SIZE: u32 = 16;

        fn new(size: WasmSize, next: WasmPtr<BlockHeader>, free: bool) -> Self {
            let free = free as u8;
            let hash = sti::hash::fxhash::fxhash32(&(size, next, free));
            Self { size, next, free, hash }
        }

        fn check_hash(&self) {
            assert_eq!(sti::hash::fxhash::fxhash32(&(self.size, self.next, self.free)), self.hash);
        }
    }

    fn heap_check(mem: wenjin::Memory, heap: WasmPtr<WasmPtr<BlockHeader>>) -> (u32, u32) {
        let mut used = 0;
        let mut free = 0;
        let mut at = mem.read(heap).unwrap();
        while !at.is_null() {
            let h = mem.read(at).unwrap();
            h.check_hash();
            if h.free != 0 {
                free += h.size.0;
            }
            else {
                used += h.size.0;
            }
            at = h.next;
        }
        return (used, free);
    }

    let mut store = Store::new();

    let malloc = store.new_host_func(|store: &mut Store, heap: WasmPtr<WasmPtr<BlockHeader>>, size: WasmSize| {
        let mut mem = store.caller_memory()?;

        if size.0 > u32::MAX / 2 {
            return Err(Error::OutOfMemory);
        }
        let size = wenjin::ceil_to_multiple_pow2(size.usize(), ALIGN) as u32;


        let (old_used, old_free) = heap_check(mem, heap);


        // try to use free block.
        let head = mem.read(heap)?;
        let mut at = head;
        while !at.is_null() {
            let h = mem.read(at).unwrap();
            let body_size = h.size.0 - BlockHeader::SIZE;
            if h.free != 0 && body_size >= size {
                // try split.
                if body_size - size >= BlockHeader::SIZE + ALIGN as u32 {
                    let used_size = BlockHeader::SIZE + size;
                    let used_header = at.byte_add(h.size.0 - used_size);
                    mem.write(used_header, BlockHeader::new(WasmSize(used_size), h.next, false)).unwrap();
                    mem.write(at, BlockHeader::new(WasmSize(h.size.0 - used_size), used_header, true)).unwrap();
                    return Ok(WasmPtr::new(used_header.addr + BlockHeader::SIZE));
                }
                else {
                    mem.write(at, BlockHeader::new(h.size, h.next, false)).unwrap();
                    return Ok(WasmPtr::new(at.addr + BlockHeader::SIZE));
                }
            }
            at = h.next;
        }


        // grow.
        let Some(grow_size) = (BlockHeader::SIZE + wasm::PAGE_SIZE32-1).checked_add(size) else {
            return Err(Error::OutOfMemory);
        };
        let num_pages = grow_size / wasm::PAGE_SIZE as u32;
        let old_pages = mem.grow(num_pages)?;

        let used_header = WasmPtr::new(old_pages*wasm::PAGE_SIZE32);
        let used_size = BlockHeader::SIZE + size;
        let free_header = WasmPtr::new(used_header.addr + used_size);
        let free_size = num_pages*wasm::PAGE_SIZE32 - used_size;
        mem.write(used_header, BlockHeader::new(WasmSize(used_size), head, false)).unwrap();
        mem.write(free_header, BlockHeader::new(WasmSize(free_size), used_header, true)).unwrap();
        mem.write(heap, free_header).unwrap();


        let (new_used, new_free) = heap_check(mem, heap);
        assert_eq!(new_used, old_used + used_size);
        assert_eq!(new_free, old_free + free_size);

        return Ok(<WasmPtr<u8>>::new(used_header.addr + BlockHeader::SIZE));
    }).unwrap();


    let fail = store.new_host_func(|store: &mut Store, msg: WasmPtr<u8>| {
        let mem = store.caller_memory().unwrap();
        let msg = mem.parse_cstr(msg).unwrap();
        let mut buf = sti::vec::Vec::new();
        mem.read_slice_to_vec(msg, &mut buf).unwrap();
        let msg = core::str::from_utf8(&buf).unwrap();
        println!("failed with {:?}", msg);
        assert!(false);
    }).unwrap();

    let module = store.new_module(include_bytes!("malloc.wasm")).unwrap();
    let inst = store.new_instance(module, &[
        ("host", "malloc", malloc.into()),
        ("host", "fail", fail.into()),
    ]).unwrap();

    #[derive(Clone, Copy, wenjin_derive::CType)]
    #[repr(C)]
    struct Tree {
        value: i32,
        left: WasmPtr<Tree>,
        right: WasmPtr<Tree>,
    }

    let run = store.get_export_func::<(), WasmPtr<Tree>>(inst, "run").unwrap();
    let tree = store.call(run, ()).unwrap();

    fn tree_sum(mem: wenjin::Memory, tree: WasmPtr<Tree>) -> i32 {
        if !tree.is_null() {
            let tree = mem.read(tree).unwrap();
            return tree_sum(mem, tree.left) + tree.value + tree_sum(mem, tree.right);
        }
        return 0;
    }

    let mem = store.memory(store.get_export_memory(inst, "memory").unwrap()).unwrap();
    let sum = tree_sum(mem, tree);
    assert_eq!(sum, 1+2+3+4+5+6+7+8);
}

