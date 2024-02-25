
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
    {
        let lib = store.load_module(include_bytes!("./lib.wasm")).unwrap();
        let lib = store.instantiate_module(lib, &Imports::new()).unwrap();

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
}


