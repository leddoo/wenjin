use wenjin::{Store, Value};


#[test]
fn basic() {
    let wasm = include_bytes!("basic.wasm");

    let mut store = Store::new();

    let module = store.new_module(wasm).unwrap();

    let inst = store.new_instance(module, &[]).unwrap();

    let add = store.get_export_func_dyn(inst, "add").unwrap();
    let mut results = [Value::I32(0)];
    store.call_dyn(add, &[Value::I32(33), Value::I32(36)], &mut results).unwrap();
    assert_eq!(results, [Value::I32(69)]);

    let thing = store.get_export_func_dyn(inst, "thing").unwrap();
    let mut results = [Value::I32(0)];
    store.call_dyn(thing, &[Value::F32(1.0), Value::I32(0)], &mut results).unwrap();
    assert_eq!(results, [Value::F32(1.0)]);
    store.call_dyn(thing, &[Value::F32(1.0), Value::I32(1)], &mut results).unwrap();
    assert_eq!(results, [Value::F32(2.0)]);
    store.call_dyn(thing, &[Value::F32(1.0), Value::I32(2)], &mut results).unwrap();
    assert_eq!(results, [Value::F32(3.0)]);
    store.call_dyn(thing, &[Value::F32(1.0), Value::I32(3)], &mut results).unwrap();
    assert_eq!(results, [Value::F32(6.0)]);


    let fib = store.get_export_func_dyn(inst, "fib").unwrap();
    for i in 0..10 {
        let result = {
            let mut a = 0;
            let mut b = 1;
            for _ in 0..i {
                (a, b) = (b, a+b);
            }
            a
        };

        let mut results = [Value::I32(0)];
        store.call_dyn(fib, &[Value::I32(i)], &mut results).unwrap();
        assert_eq!(results, [Value::I32(result)]);
    }
}


