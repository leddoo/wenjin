use wenjin::{Store, TypedFuncId};


#[test]
fn host_func() {
    let mut store = Store::new();

    let add = store.new_host_func(|a: i32, b: i32| {
        a + b
    }).unwrap();

    assert_eq!(store.call(add, (33, 36)).unwrap(), 69);


    let fib_guest: Option<TypedFuncId<i32, i32>> = None;
    let fib_guest = std::rc::Rc::new(core::cell::Cell::new(fib_guest));

    let fib_host = store.new_host_func({ let fib_guest = fib_guest.clone();
        move |store: &mut Store, n: i32| {
            Ok(if n < 2 {
                n
            }
            else {
                let fib_guest = fib_guest.get().unwrap();
                store.call(fib_guest, n-2)? + store.call(fib_guest, n-1)?
            })
        }
    }).unwrap();

    let module = store.new_module(include_bytes!("host_func.wasm")).unwrap();
    let inst = store.new_instance(module, &[("host", "fib_host", fib_host.into())]).unwrap();

    fib_guest.set(Some(store.get_export_func(inst, "fib").unwrap()));
    let fib_guest = fib_guest.get().unwrap();

    for i in 0..10 {
        let result = {
            let mut a = 0;
            let mut b = 1;
            for _ in 0..i {
                (a, b) = (b, a+b);
            }
            a
        };

        assert_eq!(store.call(fib_host, i).unwrap(), result);
        assert_eq!(store.call(fib_guest, i).unwrap(), result);
    }
}

