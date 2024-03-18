use wenjin::{Store, Error};


#[test]
fn caller_api() {
    let mut store = Store::new();

    let the_inst = std::rc::Rc::new(std::cell::Cell::new(None));

    let host_func = store.new_host_func({ let the_inst = the_inst.clone(); move |store: &mut Store, x: i32| {
        match x {
            0 => {
                assert!(matches!(store.caller_instance().unwrap_err(), Error::CallerNotWasm));
                assert!(matches!(store.caller_memory().unwrap_err(), Error::CallerNotWasm));
            }

            1 => {
                assert_eq!(store.caller_instance().unwrap(), the_inst.get().unwrap());
                assert!(store.caller_memory().is_ok());
            }

            _ => unreachable!(),
        }
    }}).unwrap();

    store.call(host_func, 0).unwrap();

    let inst = store.new_instance(include_bytes!("caller_api.wasm"),
        &[("host", "host_func", host_func.into())]).unwrap();
    the_inst.set(Some(inst));

    let call_host = store.get_export_func::<i32, ()>(inst, "call_host").unwrap();

    store.call(host_func, 0).unwrap();
    store.call(call_host, 1).unwrap();
}

