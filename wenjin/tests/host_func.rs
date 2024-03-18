use wenjin::Store;


#[test]
fn host_func() {
    let mut store = Store::new();

    let add = store.new_host_func(|a: i32, b: i32| {
        a + b
    }).unwrap();

    assert_eq!(store.call(add, (33, 36)).unwrap(), 69);


    let fib_guest = store.new_func_var::<i32, i32>().unwrap();

    let fib_host = store.new_host_func({ let fib_guest = fib_guest.clone();
        move |store: &mut Store, n: i32| {
            Ok(if n < 2 {
                n
            }
            else {
                store.call(fib_guest, n-2)? + store.call(fib_guest, n-1)?
            })
        }
    }).unwrap();

    let inst = store.new_instance(include_bytes!("host_func.wasm"),
        &[("host", "fib_host", fib_host.into())]).unwrap();

    store.assign_func_var(fib_guest, store.get_export_func(inst, "fib").unwrap()).unwrap();

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

