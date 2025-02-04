// plug name is inferred from the file name as plug1
extern "C" {
    fn print2(x: i32, y: i32);
}

// Example of state management inside wasm memory
static mut STATE_OFFSET: u32 = 0;

struct MyState {
    ns: Vec<String>,
    x: i32,
    y: u32,
}

#[no_mangle]
pub extern "C" fn __init() {
    let my_state = Box::new(MyState {
        ns: vec!["hey".to_string(), "foo".to_string()],
        x: 10,
        y: 20,
    });
    let my_state = Box::leak(my_state) as *mut MyState;
    unsafe {
        STATE_OFFSET = my_state as u32;
    }
}

#[no_mangle]
pub unsafe extern "C" fn plug1(a: i32) {
    print2(a, a);

    let state = STATE_OFFSET as *mut MyState;

    let ns_len = (*state).ns.len() as i32;
    print2(ns_len, ns_len);

    let x = (*state).x;
    let y = (*state).y as i32;
    print2(x, y);
}

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    let cs = vec![a, b, a];
    let cs = cs.into_iter().sum::<i32>();
    cs
}
