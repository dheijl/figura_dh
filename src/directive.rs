use crate::{Context, Value, err::TemplateError};
use std::rc::Rc;

pub trait Directive {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError>;
}

pub struct NoDirective;

impl Directive for NoDirective {
    fn execute(&self, _ctx: &Context) -> Result<String, TemplateError> {
        Ok(String::new())
    }
}

/// Holds the name of the variable to replace
pub struct ReplaceDirective(pub Rc<str>);

impl Directive for ReplaceDirective {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError> {
        ctx.get(&*self.0).map(|v| v.to_string()).ok_or_else(|| {
            TemplateError::DirectiveExecution(format!(
                "Trying to use value '{}' which doesn't exist in the context",
                self.0
            ))
        })
    }
}

/// pattern:count
/// Pattern can be anything that resolves to a string.
/// Count must be either a key to a non-negative integer or a non-negative integer literal.
pub struct RepeatDirective(pub Rc<str>, pub Rc<str>);

impl Directive for RepeatDirective {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError> {
        let pattern = match ctx.get(&*self.0) {
            Some(p) => p.to_string(),
            None => self.0.to_string(),
        };

        let count = match ctx.get(&*self.1) {
            Some(c) => match c {
                &Value::Int(i) if i >= 0 => i as usize,
                _ => {
                    return Err(TemplateError::DirectiveExecution(format!(
                        "The value assigned to '{}' must be a non-negative integer",
                        self.1
                    )));
                }
            },
            None => self.1.parse::<usize>().map_err(|_| {
                TemplateError::DirectiveExecution(format!(
                    "Trying to repeat '{}' with a non-integer value",
                    self.0
                ))
            })?,
        };

        Ok(pattern.repeat(count))
    }
}

/// Conditional directive: `condition ? then_value : else_value`
pub struct ConditionalDirective {
    pub condition: Rc<str>,
    pub then_value: Rc<str>,
    pub else_value: Rc<str>,
}

impl ConditionalDirective {
    pub fn new(condition: Rc<str>, then_value: Rc<str>, else_value: Rc<str>) -> Self {
        Self {
            condition,
            then_value,
            else_value,
        }
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::String(s) => !s.is_empty(),
        Value::Str(s) => !s.is_empty(),
        Value::Int(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::Bool(b) => *b,
    }
}

fn resolve_value(key: &str, ctx: &Context) -> String {
    ctx.get(key)
        .map(|v| v.to_string())
        .unwrap_or_else(|| key.to_string())
}

impl Directive for ConditionalDirective {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError> {
        let condition_met = ctx.get(&*self.condition).map(is_truthy).unwrap_or(false);

        let result = if condition_met {
            &self.then_value
        } else {
            &self.else_value
        };

        Ok(resolve_value(result, ctx))
    }
}

/// Switch directive: `value | case1 => result1 | case2 => result2 | _ => default`
pub struct SwitchDirective {
    pub value: Rc<str>,
    pub cases: Vec<(Rc<str>, Rc<str>)>, // (pattern, result)
    pub default: Option<Rc<str>>,
}

impl SwitchDirective {
    pub fn new(value: Rc<str>, cases: Vec<(Rc<str>, Rc<str>)>, default: Option<Rc<str>>) -> Self {
        Self {
            value,
            cases,
            default,
        }
    }
}

impl Directive for SwitchDirective {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError> {
        let value = resolve_value(&self.value, ctx);

        for (pattern, result) in &self.cases {
            let pattern_value = resolve_value(pattern, ctx);
            if value == pattern_value {
                return Ok(resolve_value(result, ctx));
            }
        }

        if let Some(default) = &self.default {
            return Ok(resolve_value(default, ctx));
        }

        Err(TemplateError::DirectiveExecution(format!(
            "No matching case for value '{}' in switch directive",
            value
        )))
    }
}

#[cfg(test)]
mod directive_tests {
    use crate::{
        Value,
        directive::{
            ConditionalDirective, Directive, RepeatDirective, ReplaceDirective, SwitchDirective,
        },
    };
    use std::{collections::HashMap, rc::Rc};

    // ==================== ReplaceDirective Tests ====================

    #[test]
    fn test_replace_directive_string() {
        let dir = ReplaceDirective(Rc::from("name"));
        let mut ctx = HashMap::new();
        ctx.insert("name", Value::String("World".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "World");
    }

    #[test]
    fn test_replace_directive_str() {
        let dir = ReplaceDirective(Rc::from("greeting"));
        let mut ctx = HashMap::new();
        ctx.insert("greeting", Value::Str("Hello"));
        assert_eq!(dir.execute(&ctx).unwrap(), "Hello");
    }

    #[test]
    fn test_replace_directive_int() {
        let dir = ReplaceDirective(Rc::from("count"));
        let mut ctx = HashMap::new();
        ctx.insert("count", Value::Int(42));
        assert_eq!(dir.execute(&ctx).unwrap(), "42");
    }

    #[test]
    fn test_replace_directive_negative_int() {
        let dir = ReplaceDirective(Rc::from("temp"));
        let mut ctx = HashMap::new();
        ctx.insert("temp", Value::Int(-10));
        assert_eq!(dir.execute(&ctx).unwrap(), "-10");
    }

    #[test]
    fn test_replace_directive_float() {
        let dir = ReplaceDirective(Rc::from("pi"));
        let mut ctx = HashMap::new();
        ctx.insert("pi", Value::Float(3.14159));
        assert_eq!(dir.execute(&ctx).unwrap(), "3.14159");
    }

    #[test]
    fn test_replace_directive_bool_true() {
        let dir = ReplaceDirective(Rc::from("flag"));
        let mut ctx = HashMap::new();
        ctx.insert("flag", Value::Bool(true));
        assert_eq!(dir.execute(&ctx).unwrap(), "true");
    }

    #[test]
    fn test_replace_directive_bool_false() {
        let dir = ReplaceDirective(Rc::from("flag"));
        let mut ctx = HashMap::new();
        ctx.insert("flag", Value::Bool(false));
        assert_eq!(dir.execute(&ctx).unwrap(), "false");
    }

    #[test]
    fn test_replace_directive_missing_key_error() {
        let dir = ReplaceDirective(Rc::from("missing"));
        let ctx = HashMap::new();
        let result = dir.execute(&ctx);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("doesn't exist in the context")
        );
    }

    // ==================== RepeatDirective Tests ====================

    #[test]
    fn test_repeat_directive_literal_pattern_literal_count() {
        let dir = RepeatDirective(Rc::from("ab"), Rc::from("3"));
        let ctx = HashMap::new();
        assert_eq!(dir.execute(&ctx).unwrap(), "ababab");
    }

    #[test]
    fn test_repeat_directive_context_pattern() {
        let dir = RepeatDirective(Rc::from("char"), Rc::from("4"));
        let mut ctx = HashMap::new();
        ctx.insert("char", Value::String("X".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "XXXX");
    }

    #[test]
    fn test_repeat_directive_context_count() {
        let dir = RepeatDirective(Rc::from("*"), Rc::from("stars"));
        let mut ctx = HashMap::new();
        ctx.insert("stars", Value::Int(5));
        assert_eq!(dir.execute(&ctx).unwrap(), "*****");
    }

    #[test]
    fn test_repeat_directive_both_from_context() {
        let dir = RepeatDirective(Rc::from("sep"), Rc::from("times"));
        let mut ctx = HashMap::new();
        ctx.insert("sep", Value::String("-".to_string()));
        ctx.insert("times", Value::Int(3));
        assert_eq!(dir.execute(&ctx).unwrap(), "---");
    }

    #[test]
    fn test_repeat_directive_zero_count() {
        let dir = RepeatDirective(Rc::from("x"), Rc::from("0"));
        let ctx = HashMap::new();
        assert_eq!(dir.execute(&ctx).unwrap(), "");
    }

    #[test]
    fn test_repeat_directive_one_count() {
        let dir = RepeatDirective(Rc::from("single"), Rc::from("1"));
        let ctx = HashMap::new();
        assert_eq!(dir.execute(&ctx).unwrap(), "single");
    }

    #[test]
    fn test_repeat_directive_large_count() {
        let dir = RepeatDirective(Rc::from("a"), Rc::from("1000"));
        let ctx = HashMap::new();
        let result = dir.execute(&ctx).unwrap();
        assert_eq!(result.len(), 1000);
        assert!(result.chars().all(|c| c == 'a'));
    }

    #[test]
    fn test_repeat_directive_multichar_pattern() {
        let dir = RepeatDirective(Rc::from("hello "), Rc::from("2"));
        let ctx = HashMap::new();
        assert_eq!(dir.execute(&ctx).unwrap(), "hello hello ");
    }

    #[test]
    fn test_repeat_directive_negative_count_error() {
        let dir = RepeatDirective(Rc::from("x"), Rc::from("n"));
        let mut ctx = HashMap::new();
        ctx.insert("n", Value::Int(-1));
        let result = dir.execute(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_repeat_directive_float_count_error() {
        let dir = RepeatDirective(Rc::from("x"), Rc::from("n"));
        let mut ctx = HashMap::new();
        ctx.insert("n", Value::Float(2.5));
        let result = dir.execute(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_repeat_directive_string_count_error() {
        let dir = RepeatDirective(Rc::from("x"), Rc::from("n"));
        let mut ctx = HashMap::new();
        ctx.insert("n", Value::String("five".to_string()));
        let result = dir.execute(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_repeat_directive_invalid_literal_count_error() {
        let dir = RepeatDirective(Rc::from("x"), Rc::from("not_a_number"));
        let ctx = HashMap::new();
        let result = dir.execute(&ctx);
        assert!(result.is_err());
    }

    // ==================== ConditionalDirective Tests ====================

    #[test]
    fn test_conditional_bool_true() {
        let dir = ConditionalDirective::new(Rc::from("cond"), Rc::from("yes"), Rc::from("no"));
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Bool(true));
        assert_eq!(dir.execute(&ctx).unwrap(), "yes");
    }

    #[test]
    fn test_conditional_bool_false() {
        let dir = ConditionalDirective::new(Rc::from("cond"), Rc::from("yes"), Rc::from("no"));
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Bool(false));
        assert_eq!(dir.execute(&ctx).unwrap(), "no");
    }

    #[test]
    fn test_conditional_int_truthy() {
        let dir = ConditionalDirective::new(Rc::from("val"), Rc::from("nonzero"), Rc::from("zero"));
        let mut ctx = HashMap::new();
        ctx.insert("val", Value::Int(1));
        assert_eq!(dir.execute(&ctx).unwrap(), "nonzero");
    }

    #[test]
    fn test_conditional_int_falsy() {
        let dir = ConditionalDirective::new(Rc::from("val"), Rc::from("nonzero"), Rc::from("zero"));
        let mut ctx = HashMap::new();
        ctx.insert("val", Value::Int(0));
        assert_eq!(dir.execute(&ctx).unwrap(), "zero");
    }

    #[test]
    fn test_conditional_float_truthy() {
        let dir =
            ConditionalDirective::new(Rc::from("f"), Rc::from("has_value"), Rc::from("empty"));
        let mut ctx = HashMap::new();
        ctx.insert("f", Value::Float(0.001));
        assert_eq!(dir.execute(&ctx).unwrap(), "has_value");
    }

    #[test]
    fn test_conditional_float_falsy() {
        let dir =
            ConditionalDirective::new(Rc::from("f"), Rc::from("has_value"), Rc::from("empty"));
        let mut ctx = HashMap::new();
        ctx.insert("f", Value::Float(0.0));
        assert_eq!(dir.execute(&ctx).unwrap(), "empty");
    }

    #[test]
    fn test_conditional_string_truthy() {
        let dir = ConditionalDirective::new(Rc::from("s"), Rc::from("has_text"), Rc::from("empty"));
        let mut ctx = HashMap::new();
        ctx.insert("s", Value::String("hello".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "has_text");
    }

    #[test]
    fn test_conditional_string_falsy() {
        let dir = ConditionalDirective::new(Rc::from("s"), Rc::from("has_text"), Rc::from("empty"));
        let mut ctx = HashMap::new();
        ctx.insert("s", Value::String("".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "empty");
    }

    #[test]
    fn test_conditional_missing_condition_falsy() {
        let dir = ConditionalDirective::new(
            Rc::from("missing"),
            Rc::from("found"),
            Rc::from("not_found"),
        );
        let ctx = HashMap::new();
        assert_eq!(dir.execute(&ctx).unwrap(), "not_found");
    }

    #[test]
    fn test_conditional_resolve_then_from_context() {
        let dir = ConditionalDirective::new(Rc::from("cond"), Rc::from("msg"), Rc::from("other"));
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Bool(true));
        ctx.insert("msg", Value::String("Hello!".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "Hello!");
    }

    #[test]
    fn test_conditional_resolve_else_from_context() {
        let dir =
            ConditionalDirective::new(Rc::from("cond"), Rc::from("msg"), Rc::from("fallback"));
        let mut ctx = HashMap::new();
        ctx.insert("cond", Value::Bool(false));
        ctx.insert("fallback", Value::String("Goodbye!".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "Goodbye!");
    }

    #[test]
    fn test_conditional_literal_result() {
        let dir = ConditionalDirective::new(
            Rc::from("flag"),
            Rc::from("literal_yes"),
            Rc::from("literal_no"),
        );
        let mut ctx = HashMap::new();
        ctx.insert("flag", Value::Bool(true));
        // "literal_yes" not in context, so returned as-is
        assert_eq!(dir.execute(&ctx).unwrap(), "literal_yes");
    }

    // ==================== SwitchDirective Tests ====================

    #[test]
    fn test_switch_first_case_match() {
        let dir = SwitchDirective::new(
            Rc::from("val"),
            vec![
                (Rc::from("a"), Rc::from("result_a")),
                (Rc::from("b"), Rc::from("result_b")),
            ],
            Some(Rc::from("default")),
        );
        let mut ctx = HashMap::new();
        ctx.insert("val", Value::String("a".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "result_a");
    }

    #[test]
    fn test_switch_second_case_match() {
        let dir = SwitchDirective::new(
            Rc::from("val"),
            vec![
                (Rc::from("a"), Rc::from("result_a")),
                (Rc::from("b"), Rc::from("result_b")),
            ],
            Some(Rc::from("default")),
        );
        let mut ctx = HashMap::new();
        ctx.insert("val", Value::String("b".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "result_b");
    }

    #[test]
    fn test_switch_default_case() {
        let dir = SwitchDirective::new(
            Rc::from("val"),
            vec![(Rc::from("a"), Rc::from("result_a"))],
            Some(Rc::from("fallback")),
        );
        let mut ctx = HashMap::new();
        ctx.insert("val", Value::String("x".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "fallback");
    }

    #[test]
    fn test_switch_no_match_no_default_error() {
        let dir = SwitchDirective::new(
            Rc::from("val"),
            vec![(Rc::from("a"), Rc::from("result_a"))],
            None,
        );
        let mut ctx = HashMap::new();
        ctx.insert("val", Value::String("x".to_string()));
        let result = dir.execute(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_switch_resolve_value_from_context() {
        let dir = SwitchDirective::new(
            Rc::from("key"),
            vec![(Rc::from("opt1"), Rc::from("res1"))],
            Some(Rc::from("def")),
        );
        let mut ctx = HashMap::new();
        ctx.insert("key", Value::String("opt1".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "res1");
    }

    #[test]
    fn test_switch_resolve_pattern_from_context() {
        let dir = SwitchDirective::new(
            Rc::from("val"),
            vec![(Rc::from("pat"), Rc::from("matched"))],
            Some(Rc::from("unmatched")),
        );
        let mut ctx = HashMap::new();
        ctx.insert("val", Value::String("hello".to_string()));
        ctx.insert("pat", Value::String("hello".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "matched");
    }

    #[test]
    fn test_switch_resolve_result_from_context() {
        let dir = SwitchDirective::new(
            Rc::from("val"),
            vec![(Rc::from("x"), Rc::from("output"))],
            None,
        );
        let mut ctx = HashMap::new();
        ctx.insert("val", Value::String("x".to_string()));
        ctx.insert("output", Value::String("THE OUTPUT".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "THE OUTPUT");
    }

    #[test]
    fn test_switch_many_cases() {
        let dir = SwitchDirective::new(
            Rc::from("num"),
            vec![
                (Rc::from("1"), Rc::from("one")),
                (Rc::from("2"), Rc::from("two")),
                (Rc::from("3"), Rc::from("three")),
                (Rc::from("4"), Rc::from("four")),
                (Rc::from("5"), Rc::from("five")),
            ],
            Some(Rc::from("many")),
        );
        let mut ctx = HashMap::new();
        ctx.insert("num", Value::String("4".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "four");
    }

    #[test]
    fn test_switch_with_int_value() {
        let dir = SwitchDirective::new(
            Rc::from("n"),
            vec![(Rc::from("42"), Rc::from("answer"))],
            Some(Rc::from("unknown")),
        );
        let mut ctx = HashMap::new();
        ctx.insert("n", Value::Int(42));
        assert_eq!(dir.execute(&ctx).unwrap(), "answer");
    }

    #[test]
    fn test_switch_only_default() {
        let dir = SwitchDirective::new(Rc::from("x"), vec![], Some(Rc::from("always_this")));
        let mut ctx = HashMap::new();
        ctx.insert("x", Value::String("anything".to_string()));
        assert_eq!(dir.execute(&ctx).unwrap(), "always_this");
    }

    #[test]
    fn test_switch_empty_cases_no_default_error() {
        let dir = SwitchDirective::new(Rc::from("x"), vec![], None);
        let mut ctx = HashMap::new();
        ctx.insert("x", Value::String("test".to_string()));
        let result = dir.execute(&ctx);
        assert!(result.is_err());
    }
}
