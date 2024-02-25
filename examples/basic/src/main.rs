
fn main() {
    use wenjin::*;


    let mut store = Store::new();

    // host function.
    {
        let mul = store.add_func(|a: i32, b: i32| {
            a * b
        });

        let result = store.call(mul, (23, 3)).unwrap();
        println!("mul(23, 3): {:?}", result);
    }


    // module.
    let inst;
    {
        let lib = store.load_module(include_bytes!("./lib.wasm")).unwrap();
        let lib = store.instantiate_module(lib, &Imports::new()).unwrap();
        inst = lib;

        if let Some(add) = store.exported_func::<(i32, i32), i32>(lib, "add") {
            let result = store.call(add, (33, 36)).unwrap();
            println!("add(33, 36): {:?}", result);
        }

        if let Some(fib) = store.exported_func::<i32, i32>(lib, "fib") {
            for i in 0..15 {
                let result = store.call(fib, i).unwrap();
                println!("fib({i}): {:?}", result);
            }
        }
    }


    // accessing memory.
    {
        let print_bytes = store.add_func(|mem: &mut MemoryView, ptr: WasmPtr<u8>, len: WasmSize| -> Result<(), ()> {
            let bytes = mem.bytes(ptr, len)?;
            println!("{:x?}", bytes);
            Ok(())
        });

        // @todo: WithMemory host functions can't be called directly,
        // not sure why. but that's how you define one.
        _ = print_bytes;
        //store.call(print_bytes, (WasmPtr::new(0), WasmSize(128))).unwrap();
    }



    // structs.
    {
        #[derive(Clone, Copy, Debug, wenjin_derive::CType)]
        #[repr(C)]
        struct Foo {
            a: u32,
            b: WasmPtr<Foo>,
        }

        // @todo: new_memory.
        let memory = store.exported_memory(inst, "memory").unwrap();
        let mut mem = store.memory_view(memory);

        let ptr = WasmPtr::new(128);
        mem.write(ptr, Foo {
            a: 42,
            b: WasmPtr::new(0),
        }).unwrap();

        let foo = mem.read(ptr).unwrap();
        println!("read(write(foo)): {:?}", foo);
    }
}


