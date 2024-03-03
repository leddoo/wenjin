use wenjin::Store;


#[test]
fn basic() {
    let wasm = include_bytes!("basic.wasm");

    let mut store = Store::new();

    let module = store.new_module(wasm).unwrap();

    let inst = store.new_instance(module, &[]).unwrap();
}


