use charmi_macros::charmi_str;

#[test]
fn test() {
    const m: charmi::CharmiStr = charmi_str!("Hello!");
    println!("M - {m:?}");
    panic!("Who am I?");
}
