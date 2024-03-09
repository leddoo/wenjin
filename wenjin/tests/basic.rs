use wenjin::{Store, Value};


#[test]
fn basic() {
    let wasm = include_bytes!("basic.wasm");

    let mut store = Store::new();

    let module = store.new_module(wasm).unwrap();

    let inst = store.new_instance(module, &[]).unwrap();

    let add = store.get_export_func_dyn(inst, "add").unwrap();
    let add_ty = store.get_export_func::<(i32, i32), i32>(inst, "add").unwrap();
    let mut results = [Value::I32(0)];
    assert_eq!(store.call_dyn(add, &[Value::I32(33), Value::I32(36)], &mut results).unwrap(), [Value::I32(69)]);
    assert_eq!(store.call(add_ty, (33, 36)).unwrap(), 69);

    let thing = store.get_export_func_dyn(inst, "thing").unwrap();
    let thing_ty = store.get_export_func::<(f32, i32), f32>(inst, "thing").unwrap();
    let mut results = [Value::I32(0)];
    assert_eq!(store.call_dyn(thing, &[Value::F32(1.0), Value::I32(0)], &mut results).unwrap(), [Value::F32(1.0)]);
    assert_eq!(store.call(thing_ty, (1.0, 0)).unwrap(), 1.0);
    assert_eq!(store.call_dyn(thing, &[Value::F32(1.0), Value::I32(1)], &mut results).unwrap(), [Value::F32(2.0)]);
    assert_eq!(store.call(thing_ty, (1.0, 1)).unwrap(), 2.0);
    assert_eq!(store.call_dyn(thing, &[Value::F32(1.0), Value::I32(2)], &mut results).unwrap(), [Value::F32(3.0)]);
    assert_eq!(store.call(thing_ty, (1.0, 2)).unwrap(), 3.0);
    assert_eq!(store.call_dyn(thing, &[Value::F32(1.0), Value::I32(3)], &mut results).unwrap(), [Value::F32(6.0)]);
    assert_eq!(store.call(thing_ty, (1.0, 3)).unwrap(), 6.0);


    let fib = store.get_export_func_dyn(inst, "fib").unwrap();
    let fib_ty = store.get_export_func::<i32, i32>(inst, "fib").unwrap();
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
        assert_eq!(store.call_dyn(fib, &[Value::I32(i)], &mut results).unwrap(), [Value::I32(result)]);
        assert_eq!(store.call(fib_ty, i).unwrap(), result);
    }
}


