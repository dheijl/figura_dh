#[cfg(test)]
mod boundary_tests {
    use figura::{Lexer, Template, Token, Value};
    use std::collections::HashMap;

    type Tpl = Template<'{', '}'>;

    // ==================== Number Boundary Tests ====================

    #[test]
    fn test_lexer_i64_max() {
        let tokens = Lexer::tokenize("9223372036854775807");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Int(i64::MAX)));
    }

    #[test]
    fn test_lexer_i64_min_abs() {
        // i64::MIN as positive would overflow, but we can test a large negative
        let tokens = Lexer::tokenize("-9223372036854775807");
        assert_eq!(tokens.len(), 2); // Minus and Int
    }

    #[test]
    fn test_lexer_float_precision() {
        let tokens = Lexer::tokenize("0.123456789012345");
        assert_eq!(tokens.len(), 1);
    }

    #[test]
    fn test_lexer_very_small_float() {
        let tokens = Lexer::tokenize("0.000000001");
        assert_eq!(tokens.len(), 1);
    }

    // ==================== String Boundary Tests ====================

    #[test]
    fn test_lexer_string_with_all_escapes() {
        let tokens = Lexer::tokenize("\"\\n\\t\\r\\\\\\\"\\0\"");
        assert_eq!(tokens.len(), 1);
        if let Token::Literal(s) = &tokens[0] {
            assert_eq!(&**s, "\n\t\r\\\"\0");
        } else {
            panic!("Expected Literal token");
        }
    }

    #[test]
    fn test_lexer_unclosed_string() {
        // Lexer should handle unclosed string (reaches EOF)
        let tokens = Lexer::tokenize("\"unclosed");
        assert_eq!(tokens.len(), 1);
    }

    // ==================== Template Boundary Tests ====================

    #[test]
    fn test_template_single_char() {
        let tpl = Tpl::parse("a").unwrap();
        assert_eq!(tpl.parts.len(), 1);
    }

    #[test]
    fn test_template_single_directive() {
        let mut ctx = HashMap::new();
        ctx.insert("x", Value::String("y".to_string()));

        let mut tpl = Tpl::parse("{x}").unwrap();
        let result = tpl.format(&ctx).unwrap();
        assert_eq!(result, "y");
    }

    #[test]
    fn test_template_directive_at_start() {
        let mut ctx = HashMap::new();
        ctx.insert("x", Value::String("START".to_string()));

        let mut tpl = Tpl::parse("{x} middle end").unwrap();
        let result = tpl.format(&ctx).unwrap();
        assert_eq!(result, "START middle end");
    }

    #[test]
    fn test_template_directive_at_end() {
        let mut ctx = HashMap::new();
        ctx.insert("x", Value::String("END".to_string()));

        let mut tpl = Tpl::parse("start middle {x}").unwrap();
        let result = tpl.format(&ctx).unwrap();
        assert_eq!(result, "start middle END");
    }

    #[test]
    fn test_template_only_escape() {
        let tpl = Tpl::parse("\\{").unwrap();
        assert_eq!(tpl.parts.len(), 1);
    }

    #[test]
    fn test_template_escape_at_end() {
        let tpl = Tpl::parse("text\\}").unwrap();
        assert_eq!(tpl.parts.len(), 1);
    }
}
