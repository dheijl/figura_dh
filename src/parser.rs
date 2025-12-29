use crate::{
    NoDirective,
    directive::{
        ConditionalDirective, Directive, RepeatDirective, ReplaceDirective, SwitchDirective,
    },
    err::TemplateError,
    lexer::Token,
};
use std::rc::Rc;

pub trait Parser {
    fn parse(input: &[Token]) -> Result<Box<dyn Directive>, TemplateError>;
}

pub struct DefaultParser;

impl DefaultParser {
    fn is_value_token(token: &Token) -> bool {
        matches!(
            token,
            Token::Ident(_) | Token::Literal(_) | Token::Int(_) | Token::Float(_)
        )
    }

    fn parse_switch(tokens: &[Token]) -> Result<Box<dyn Directive>, TemplateError> {
        // value | case1 => result1 | case2 => result2 | _ => default
        let mut iter = tokens.iter();

        let value = match iter.next() {
            Some(t) if Self::is_value_token(t) => t.as_string(),
            _ => {
                return Err(TemplateError::DirectiveParsing(
                    "Switch directive must start with a value".to_string(),
                ));
            }
        };

        let mut cases: Vec<(Rc<str>, Rc<str>)> = Vec::new();
        let mut default: Option<Rc<str>> = None;

        while let Some(token) = iter.next() {
            if *token != Token::Pipe {
                return Err(TemplateError::DirectiveParsing(
                    "Expected '|' in switch directive".to_string(),
                ));
            }

            match iter.next() {
                Some(Token::Underscore) => {
                    // Default case: _ => result
                    if iter.next() != Some(&Token::Arrow) {
                        return Err(TemplateError::DirectiveParsing(
                            "Expected '=>' after '_' in switch".to_string(),
                        ));
                    }
                    match iter.next() {
                        Some(t) if Self::is_value_token(t) => {
                            default = Some(t.as_string());
                        }
                        _ => {
                            return Err(TemplateError::DirectiveParsing(
                                "Expected value after '=>' in default case".to_string(),
                            ));
                        }
                    }
                }
                Some(t) if Self::is_value_token(t) => {
                    let pattern = t.as_string();
                    if iter.next() != Some(&Token::Arrow) {
                        return Err(TemplateError::DirectiveParsing(
                            "Expected '=>' after case pattern".to_string(),
                        ));
                    }
                    match iter.next() {
                        Some(t) if Self::is_value_token(t) => {
                            cases.push((pattern, t.as_string()));
                        }
                        _ => {
                            return Err(TemplateError::DirectiveParsing(
                                "Expected value after '=>'".to_string(),
                            ));
                        }
                    }
                }
                _ => {
                    return Err(TemplateError::DirectiveParsing(
                        "Expected case pattern or '_' after '|'".to_string(),
                    ));
                }
            }
        }

        Ok(Box::new(SwitchDirective::new(value, cases, default)))
    }
}

impl Parser for DefaultParser {
    fn parse(input: &[Token]) -> Result<Box<dyn Directive>, TemplateError> {
        // Empty directive: {}
        if input.is_empty() {
            return Ok(Box::new(NoDirective));
        }

        // Single identifier: {name}
        if let [Token::Ident(ident)] = input {
            return Ok(Box::new(ReplaceDirective(Rc::clone(ident))));
        }

        // Repeat: pattern : count
        if input.len() == 3 && input[1] == Token::Colon {
            if Self::is_value_token(&input[0]) && Self::is_value_token(&input[2]) {
                return Ok(Box::new(RepeatDirective(
                    input[0].as_string(),
                    input[2].as_string(),
                )));
            }
        }

        // Conditional: condition ? then : else
        if input.len() == 5 && input[1] == Token::Question && input[3] == Token::Colon {
            if Self::is_value_token(&input[0])
                && Self::is_value_token(&input[2])
                && Self::is_value_token(&input[4])
            {
                return Ok(Box::new(ConditionalDirective::new(
                    input[0].as_string(),
                    input[2].as_string(),
                    input[4].as_string(),
                )));
            }
        }

        // Switch: value | case => result | ...
        if input.len() >= 4 && input.iter().any(|t| *t == Token::Pipe) {
            return Self::parse_switch(input);
        }

        Err(TemplateError::DirectiveParsing(
            "Unhandled token pattern".to_string(),
        ))
    }
}

#[cfg(test)]
mod parser_tests {
    use crate::{
        Context, Value,
        err::TemplateError,
        lexer::Lexer,
        parser::{DefaultParser, Parser},
    };
    use std::collections::HashMap;

    fn parse_and_execute(input: &str, ctx: &Context) -> Result<String, TemplateError> {
        let tokens = Lexer::tokenize(input);
        let directive = DefaultParser::parse(&tokens)?;
        directive.execute(ctx)
    }

    // ==================== Replace Directive Tests ====================

    #[test]
    fn test_parse_simple_replace() {
        let mut ctx = HashMap::new();
        ctx.insert("name", Value::String("Alice".to_string()));

        let result = parse_and_execute("name", &ctx).unwrap();
        assert_eq!(result, "Alice");
    }

    #[test]
    fn test_parse_replace_int() {
        let mut ctx = HashMap::new();
        ctx.insert("age", Value::Int(25));

        let result = parse_and_execute("age", &ctx).unwrap();
        assert_eq!(result, "25");
    }

    #[test]
    fn test_parse_replace_float() {
        let mut ctx = HashMap::new();
        ctx.insert("price", Value::Float(19.99));

        let result = parse_and_execute("price", &ctx).unwrap();
        assert_eq!(result, "19.99");
    }

    #[test]
    fn test_parse_replace_bool() {
        let mut ctx = HashMap::new();
        ctx.insert("active", Value::Bool(true));

        let result = parse_and_execute("active", &ctx).unwrap();
        assert_eq!(result, "true");
    }

    #[test]
    fn test_parse_replace_missing_key() {
        let ctx = HashMap::new();
        let result = parse_and_execute("missing", &ctx);
        assert!(result.is_err());
    }

    // ==================== Repeat Directive Tests ====================

    #[test]
    fn test_parse_repeat_literal_count() {
        let ctx = HashMap::new();
        let result = parse_and_execute("\"ab\" : 3", &ctx).unwrap();
        assert_eq!(result, "ababab");
    }

    #[test]
    fn test_parse_repeat_zero() {
        let ctx = HashMap::new();
        let result = parse_and_execute("\"x\" : 0", &ctx).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_parse_repeat_one() {
        let ctx = HashMap::new();
        let result = parse_and_execute("\"hello\" : 1", &ctx).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_parse_repeat_with_context_pattern() {
        let mut ctx = HashMap::new();
        ctx.insert("pattern", Value::String("xy".to_string()));

        let result = parse_and_execute("pattern : 2", &ctx).unwrap();
        assert_eq!(result, "xyxy");
    }

    #[test]
    fn test_parse_repeat_with_context_count() {
        let mut ctx = HashMap::new();
        ctx.insert("count", Value::Int(4));

        let result = parse_and_execute("\"z\" : count", &ctx).unwrap();
        assert_eq!(result, "zzzz");
    }

    #[test]
    fn test_parse_repeat_both_from_context() {
        let mut ctx = HashMap::new();
        ctx.insert("pat", Value::String("->".to_string()));
        ctx.insert("n", Value::Int(3));

        let result = parse_and_execute("pat : n", &ctx).unwrap();
        assert_eq!(result, "->->->");
    }

    #[test]
    fn test_parse_repeat_ident_as_literal_pattern() {
        let ctx = HashMap::new();
        // If pattern is not in context, use it as literal
        let result = parse_and_execute("abc : 2", &ctx).unwrap();
        assert_eq!(result, "abcabc");
    }

    #[test]
    fn test_parse_repeat_negative_count_error() {
        let mut ctx = HashMap::new();
        ctx.insert("count", Value::Int(-5));

        let result = parse_and_execute("\"x\" : count", &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_repeat_non_integer_count_error() {
        let mut ctx = HashMap::new();
        ctx.insert("count", Value::Float(2.5));

        let result = parse_and_execute("\"x\" : count", &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_repeat_invalid_literal_count() {
        let ctx = HashMap::new();
        let result = parse_and_execute("\"x\" : notanumber", &ctx);
        assert!(result.is_err());
    }

    // ==================== Conditional Directive Tests ====================

    #[test]
    fn test_parse_conditional_true() {
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Bool(true));

        let result = parse_and_execute("cond ? yes : no", &ctx).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_parse_conditional_false() {
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Bool(false));

        let result = parse_and_execute("cond ? yes : no", &ctx).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_parse_conditional_missing_is_false() {
        let ctx = HashMap::new();
        let result = parse_and_execute("cond ? yes : no", &ctx).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_parse_conditional_int_nonzero_truthy() {
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Int(42));

        let result = parse_and_execute("cond ? yes : no", &ctx).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_parse_conditional_int_zero_falsy() {
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Int(0));

        let result = parse_and_execute("cond ? yes : no", &ctx).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_parse_conditional_float_nonzero_truthy() {
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Float(0.1));

        let result = parse_and_execute("cond ? yes : no", &ctx).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_parse_conditional_float_zero_falsy() {
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Float(0.0));

        let result = parse_and_execute("cond ? yes : no", &ctx).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_parse_conditional_string_nonempty_truthy() {
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::String("hello".to_string()));

        let result = parse_and_execute("cond ? yes : no", &ctx).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_parse_conditional_string_empty_falsy() {
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::String("".to_string()));

        let result = parse_and_execute("cond ? yes : no", &ctx).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_parse_conditional_str_nonempty_truthy() {
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Str("hi"));

        let result = parse_and_execute("cond ? yes : no", &ctx).unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_parse_conditional_str_empty_falsy() {
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Str(""));

        let result = parse_and_execute("cond ? yes : no", &ctx).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_parse_conditional_resolve_then_value() {
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Bool(true));
        ctx.insert("yes", Value::String("SUCCESS".to_string()));

        let result = parse_and_execute("cond ? yes : no", &ctx).unwrap();
        assert_eq!(result, "SUCCESS");
    }

    #[test]
    fn test_parse_conditional_resolve_else_value() {
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Bool(false));
        ctx.insert("no", Value::String("FAILURE".to_string()));

        let result = parse_and_execute("cond ? yes : no", &ctx).unwrap();
        assert_eq!(result, "FAILURE");
    }

    #[test]
    fn test_parse_conditional_with_literals() {
        let mut ctx = HashMap::new();
        ctx.insert("premium", Value::Bool(true));

        let result = parse_and_execute("premium ? \"VIP\" : \"Standard\"", &ctx).unwrap();
        assert_eq!(result, "VIP");
    }

    #[test]
    fn test_parse_conditional_with_numbers() {
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Bool(true));

        let result = parse_and_execute("cond ? 100 : 0", &ctx).unwrap();
        assert_eq!(result, "100");
    }

    // ==================== Switch Directive Tests ====================

    #[test]
    fn test_parse_switch_first_case() {
        let mut ctx = HashMap::new();
        ctx.insert("status", Value::String("active".to_string()));

        let result = parse_and_execute(
            "status | \"active\" => \"Online\" | \"inactive\" => \"Offline\"",
            &ctx,
        )
        .unwrap();
        assert_eq!(result, "Online");
    }

    #[test]
    fn test_parse_switch_second_case() {
        let mut ctx = HashMap::new();
        ctx.insert("status", Value::String("inactive".to_string()));

        let result = parse_and_execute(
            "status | \"active\" => \"Online\" | \"inactive\" => \"Offline\"",
            &ctx,
        )
        .unwrap();
        assert_eq!(result, "Offline");
    }

    #[test]
    fn test_parse_switch_default() {
        let mut ctx = HashMap::new();
        ctx.insert("status", Value::String("unknown".to_string()));

        let result = parse_and_execute(
            "status | \"active\" => \"On\" | \"inactive\" => \"Off\" | _ => \"Unknown\"",
            &ctx,
        )
        .unwrap();
        assert_eq!(result, "Unknown");
    }

    #[test]
    fn test_parse_switch_no_match_no_default_error() {
        let mut ctx = HashMap::new();
        ctx.insert("status", Value::String("unknown".to_string()));

        let result = parse_and_execute(
            "status | \"active\" => \"On\" | \"inactive\" => \"Off\"",
            &ctx,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_switch_with_idents() {
        let mut ctx = HashMap::new();
        ctx.insert("val", Value::String("a".to_string()));
        ctx.insert("result_a", Value::String("Result A".to_string()));

        let result = parse_and_execute("val | \"a\" => result_a | _ => \"default\"", &ctx).unwrap();
        assert_eq!(result, "Result A");
    }

    #[test]
    fn test_parse_switch_pattern_from_context() {
        let mut ctx = HashMap::new();
        ctx.insert("val", Value::String("x".to_string()));
        ctx.insert("pat", Value::String("x".to_string()));

        let result = parse_and_execute("val | pat => \"matched\" | _ => \"nope\"", &ctx).unwrap();
        assert_eq!(result, "matched");
    }

    #[test]
    fn test_parse_switch_many_cases() {
        let mut ctx = HashMap::new();
        ctx.insert("n", Value::String("3".to_string()));

        let result = parse_and_execute(
            "n | \"1\" => \"one\" | \"2\" => \"two\" | \"3\" => \"three\" | _ => \"many\"",
            &ctx,
        )
        .unwrap();
        assert_eq!(result, "three");
    }

    #[test]
    fn test_parse_switch_only_default() {
        let mut ctx = HashMap::new();
        ctx.insert("x", Value::String("anything".to_string()));

        let result = parse_and_execute("x | _ => \"always\"", &ctx).unwrap();
        assert_eq!(result, "always");
    }

    // ==================== Parser Error Tests ====================

    #[test]
    fn test_parse_unhandled_pattern() {
        let tokens = Lexer::tokenize("a + b");
        let result = DefaultParser::parse(&tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_incomplete_conditional() {
        let tokens = Lexer::tokenize("cond ? yes");
        let result = DefaultParser::parse(&tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_switch_missing_arrow() {
        let tokens = Lexer::tokenize("val | a b");
        let result = DefaultParser::parse(&tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_switch_missing_result() {
        let tokens = Lexer::tokenize("val | a =>");
        let result = DefaultParser::parse(&tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_switch_invalid_start() {
        let tokens = Lexer::tokenize("+ | a => b");
        let result = DefaultParser::parse(&tokens);
        assert!(result.is_err());
    }
}
