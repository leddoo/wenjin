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
}


