#[cfg(test)]
mod stress_tests {
    use figura::{Lexer, Template, Value};
    use std::collections::HashMap;

    type Tpl = Template<'{', '}'>;

    #[test]
    fn test_lexer_long_identifier() {
        let long_ident = "a".repeat(10000);
        let tokens = Lexer::tokenize(&long_ident);
        assert_eq!(tokens.len(), 1);
    }

    #[test]
    fn test_lexer_long_string() {
        let long_string = format!("\"{}\"", "x".repeat(10000));
        let tokens = Lexer::tokenize(&long_string);
        assert_eq!(tokens.len(), 1);
    }

    #[test]
    fn test_lexer_many_tokens() {
        let input = "a + ".repeat(1000) + "a";
        let tokens = Lexer::tokenize(&input);
        assert_eq!(tokens.len(), 2001); // 1000 pairs + 1 final
    }

    #[test]
    fn test_template_long_text() {
        let long_text = "x".repeat(100000);
        let tpl = Tpl::parse(&long_text).unwrap();
        assert_eq!(tpl.parts.len(), 1);
    }

    #[test]
    fn test_template_many_small_parts() {
        let mut ctx = HashMap::new();
        ctx.insert("x", Value::String("X".to_string()));

        // 500 alternating text and directive parts
        let template: String = (0..500).map(|_| "a{x}").collect();
        let mut tpl = Tpl::parse(&template).unwrap();
        let result = tpl.format(&ctx).unwrap();

        assert_eq!(result.len(), 1000); // 500 'a' + 500 'X'
    }

    #[test]
    fn test_deeply_escaped() {
        let input = "\\".repeat(100);
        let tpl = Tpl::parse(&input).unwrap();
        // Every pair of \\ becomes one \
        assert_eq!(tpl.parts.len(), 1);
    }
}
