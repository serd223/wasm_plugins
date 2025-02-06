#[no_mangle]
pub extern "C" fn __name() -> *const u8 {
    b"plug2\0".as_ptr()
}

extern "C" {
    fn print(a: i32);
    fn add(a: i32, b: i32) -> i32;
}

#[no_mangle]
pub extern "C" fn __deps() -> *const u8 {
    b"plug1\0".as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn plug2(a: i32) {
    print(add(a, a));
}

#[no_mangle]
pub unsafe extern "C" fn mul(a: i32, b: i32) -> i32 {
    let res = a * b;
    print(res);
    res
}
