#[no_mangle]
pub extern "C" fn __name() -> *const u8 {
    b"plug3\0".as_ptr()
}

extern "C" {
    fn print2(x: usize, y: usize);
    fn plug2(a: i32);
    fn print(a: i32);
    fn add(a: i32, b: i32) -> i32;
}

#[no_mangle]
pub extern "C" fn __deps() -> *const u8 {
    b"plug2;plug1\0".as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn plug3(a: i32) {
    plug2(a);
    let n = a * a + 2 * a;
    print(add(a, n));
    print2(n as usize, a as usize);
}
