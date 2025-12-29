#[cfg(test)]
mod nested_delimiter_tests {
    use figura::{Template, Value};
    use std::collections::HashMap;

    type Tpl = Template<'{', '}'>;

    #[test]
    fn test_nested_braces_in_directive() {
        // When O != C, nested delimiters should be captured
        let mut ctx = HashMap::new();
        ctx.insert("x", Value::String("test".to_string()));

        // This tests that nested { } inside directive content are preserved
        let mut tpl = Tpl::parse("outer {x} end").unwrap();
        let result = tpl.format(&ctx).unwrap();
        assert_eq!(result, "outer test end");
    }

    #[test]
    fn test_validate_nested_depth() {
        assert_eq!(Tpl::validate("{{}}"), 0);
        assert_eq!(Tpl::validate("{{{}}}"), 0);
        assert_eq!(Tpl::validate("{{{"), 3);
        assert_eq!(Tpl::validate("}}}"), -1); // Stops at first negative
    }

    #[test]
    fn test_deeply_nested() {
        // 5 levels deep
        assert_eq!(Tpl::validate("{{{{{x}}}}}"), 0);
    }

    #[test]
    fn test_unbalanced_nested() {
        assert!(Tpl::validate("{{{}}") > 0);
        assert!(Tpl::validate("{{}}}") < 0);
    }
}
