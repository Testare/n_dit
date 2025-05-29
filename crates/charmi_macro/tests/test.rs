mod tests {
    use charmi_core::CharmiImage;
    use charmi_macro::charmi_toml;

    #[test]
    fn small_test() {
        let gap = '#';
        let attr: Option<String> = None;
        let blue = "blue".to_string();
        let result = charmi_toml![
            r#"
            text="hello#world"
            bg="bbbbb rrrrr"
            values.colors.r="red"
            "#,
            values.gap = gap,
            attr?,
            values.colors.b = blue
        ];
        let expected = CharmiImage::build_dynamic()
            .no_fg()
            .bg("blue")
            .add_text("hello")
            .bg("red")
            .add_gap(1)
            .add_text("world")
            .build();
        assert_eq!(result, expected);
    }
}
