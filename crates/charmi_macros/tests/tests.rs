use charmi_macros::charmi_str;

#[test]
fn test() {
    const M: charmi::CharmiStr = charmi_str!("Hello!");
    println!("M - {M:?}");
    panic!("Who am I?");
}
