// plug name is inferred from the file name as plug4
extern "C" {
    fn plug2(a: i32);
}

#[no_mangle]
pub extern "C" fn deps() -> *const u8 {
    b"plug2\0".as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn plug4(a: i32) {
    plug2(a + 20);
}
