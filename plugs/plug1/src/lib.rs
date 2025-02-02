// plug name is inferred from the file name as plug1
extern "C" {
    fn draw(x: usize, y: usize);
    fn get_state() -> i32;
}

#[no_mangle]
pub unsafe extern "C" fn plug1(a: i32) {
    draw(a as usize, a as usize);
}

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    // let cs = vec![a + b + unsafe { get_state() }];
    let cs = vec![a, b, unsafe { get_state() }];
    let cs = cs.into_iter().sum::<i32>();
    cs
}
