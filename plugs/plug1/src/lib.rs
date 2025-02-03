// plug name is inferred from the file name as plug1
extern "C" {
    fn print2(x: i32, y: i32);
}

#[no_mangle]
pub unsafe extern "C" fn plug1(a: i32) {
    print2(a, a);
}

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    let cs = vec![a, b, a];
    let cs = cs.into_iter().sum::<i32>();
    cs
}
