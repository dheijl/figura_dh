#[cfg(test)]
mod integration_tests {
    use figura::{Context, Template, Value};
    use std::collections::HashMap;

    type Tpl = Template<'{', '}'>;

    fn format_template(template: &str, ctx: &Context) -> Result<String, figura::TemplateError> {
        let mut tpl = Tpl::parse(template)?;
        tpl.format(ctx)
    }

    // ==================== Basic Formatting Tests ====================

    #[test]
    fn test_format_no_directives() {
        let ctx = HashMap::new();
        let result = format_template("Hello, World!", &ctx).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_format_single_replacement() {
        let mut ctx = HashMap::new();
        ctx.insert("name", Value::String("Alice".to_string()));
        let result = format_template("Hello, {name}!", &ctx).unwrap();
        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_format_multiple_replacements() {
        let mut ctx = HashMap::new();
        ctx.insert("first", Value::String("John".to_string()));
        ctx.insert("last", Value::String("Doe".to_string()));
        let result = format_template("{first} {last}", &ctx).unwrap();
        assert_eq!(result, "John Doe");
    }

    #[test]
    fn test_format_adjacent_directives() {
        let mut ctx = HashMap::new();
        ctx.insert("a", Value::String("X".to_string()));
        ctx.insert("b", Value::String("Y".to_string()));
        ctx.insert("c", Value::String("Z".to_string()));
        let result = format_template("{a}{b}{c}", &ctx).unwrap();
        assert_eq!(result, "XYZ");
    }

    #[test]
    fn test_format_only_directive() {
        let mut ctx = HashMap::new();
        ctx.insert("value", Value::Int(42));
        let result = format_template("{value}", &ctx).unwrap();
        assert_eq!(result, "42");
    }

    // ==================== Repeat Directive Integration Tests ====================

    #[test]
    fn test_format_repeat() {
        let ctx = HashMap::new();
        let result = format_template("Stars: {\"*\" : 5}", &ctx).unwrap();
        assert_eq!(result, "Stars: *****");
    }

    #[test]
    fn test_format_repeat_with_context() {
        let mut ctx = HashMap::new();
        ctx.insert("char", Value::String("-".to_string()));
        ctx.insert("count", Value::Int(3));
        let result = format_template("Line: {char : count}", &ctx).unwrap();
        assert_eq!(result, "Line: ---");
    }

    #[test]
    fn test_format_repeat_in_sentence() {
        let ctx = HashMap::new();
        let result = format_template("Rating: {\"★\" : 4}{\"☆\" : 1}", &ctx).unwrap();
        assert_eq!(result, "Rating: ★★★★☆");
    }

    // ==================== Conditional Directive Integration Tests ====================

    #[test]
    fn test_format_conditional_true() {
        let mut ctx = HashMap::new();
        ctx.insert("premium", Value::Bool(true));
        let result = format_template("Status: {premium ? \"VIP\" : \"Standard\"}", &ctx).unwrap();
        assert_eq!(result, "Status: VIP");
    }

    #[test]
    fn test_format_conditional_false() {
        let mut ctx = HashMap::new();
        ctx.insert("premium", Value::Bool(false));
        let result = format_template("Status: {premium ? \"VIP\" : \"Standard\"}", &ctx).unwrap();
        assert_eq!(result, "Status: Standard");
    }

    #[test]
    fn test_format_conditional_with_values() {
        let mut ctx = HashMap::new();
        ctx.insert("logged_in", Value::Bool(true));
        ctx.insert("username", Value::String("alice".to_string()));
        let result = format_template("Hello, {logged_in ? username : \"Guest\"}!", &ctx).unwrap();
        assert_eq!(result, "Hello, alice!");
    }

    #[test]
    fn test_format_conditional_missing_condition() {
        let ctx = HashMap::new();
        let result = format_template("{exists ? \"yes\" : \"no\"}", &ctx).unwrap();
        assert_eq!(result, "no");
    }

    // ==================== Switch Directive Integration Tests ====================

    #[test]
    fn test_format_switch() {
        let mut ctx = HashMap::new();
        ctx.insert("lang", Value::String("es".to_string()));
        let result = format_template(
            "{lang | \"en\" => \"Hello\" | \"es\" => \"Hola\" | \"fr\" => \"Bonjour\" | _ => \"Hi\"}",
            &ctx,
        )
        .unwrap();
        assert_eq!(result, "Hola");
    }

    #[test]
    fn test_format_switch_default() {
        let mut ctx = HashMap::new();
        ctx.insert("lang", Value::String("de".to_string()));
        let result = format_template(
            "{lang | \"en\" => \"Hello\" | \"es\" => \"Hola\" | _ => \"Greetings\"}",
            &ctx,
        )
        .unwrap();
        assert_eq!(result, "Greetings");
    }

    #[test]
    fn test_format_switch_with_context_results() {
        let mut ctx = HashMap::new();
        ctx.insert("status", Value::String("success".to_string()));
        ctx.insert(
            "success_msg",
            Value::String("Operation completed!".to_string()),
        );
        ctx.insert(
            "error_msg",
            Value::String("Something went wrong.".to_string()),
        );
        let result = format_template(
            "{status | \"success\" => success_msg | \"error\" => error_msg | _ => \"Unknown status\"}",
            &ctx,
        )
        .unwrap();
        assert_eq!(result, "Operation completed!");
    }

    // ==================== Complex Template Tests ====================

    #[test]
    fn test_format_email_template() {
        let mut ctx = HashMap::new();
        ctx.insert("name", Value::String("John".to_string()));
        ctx.insert("order_id", Value::Int(12345));
        ctx.insert("total", Value::Float(99.99));

        let template = "Dear {name},\n\nYour order #{order_id} has been confirmed.\nTotal: ${total}\n\nThank you!";
        let result = format_template(template, &ctx).unwrap();
        assert!(result.contains("Dear John"));
        assert!(result.contains("#12345"));
        assert!(result.contains("$99.99"));
    }

    #[test]
    fn test_format_html_template() {
        let mut ctx = HashMap::new();
        ctx.insert("title", Value::String("Welcome".to_string()));
        ctx.insert("content", Value::String("Hello, World!".to_string()));

        let template = "<html><head><title>{title}</title></head><body>{content}</body></html>";
        let result = format_template(template, &ctx).unwrap();
        assert_eq!(
            result,
            "<html><head><title>Welcome</title></head><body>Hello, World!</body></html>"
        );
    }

    #[test]
    fn test_format_url_template() {
        let mut ctx = HashMap::new();
        ctx.insert("base", Value::String("https://api.example.com".to_string()));
        ctx.insert("user_id", Value::Int(42));
        ctx.insert("resource", Value::String("posts".to_string()));

        let template = "{base}/users/{user_id}/{resource}";
        let result = format_template(template, &ctx).unwrap();
        assert_eq!(result, "https://api.example.com/users/42/posts");
    }

    #[test]
    fn test_format_mixed_directives() {
        let mut ctx = HashMap::new();
        ctx.insert("show_stars", Value::Bool(true));
        ctx.insert("rating", Value::Int(4));
        ctx.insert("name", Value::String("Product".to_string()));

        let template = "{name}: {show_stars ? \"★\" : \"\"}{\"★\" : 3}";
        let result = format_template(template, &ctx).unwrap();
        assert_eq!(result, "Product: ★★★★");
    }

    // ==================== Error Handling Tests ====================

    #[test]
    fn test_format_missing_variable_error() {
        let ctx = HashMap::new();
        let result = format_template("Hello, {missing}!", &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_unmatched_delimiter_error() {
        let ctx = HashMap::new();
        let result = format_template("Hello, {name", &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_extra_closing_delimiter_error() {
        let ctx = HashMap::new();
        let result = format_template("Hello, name}", &ctx);
        assert!(result.is_err());
    }

    // ==================== Escape Sequence Tests ====================

    #[test]
    fn test_format_escaped_delimiters() {
        let ctx = HashMap::new();
        let result = format_template("Use \\{braces\\} for directives", &ctx).unwrap();
        assert_eq!(result, "Use {braces} for directives");
    }

    #[test]
    fn test_format_escaped_with_directive() {
        let mut ctx = HashMap::new();
        ctx.insert("name", Value::String("Test".to_string()));
        let result = format_template("\\{literal\\} and {name}", &ctx).unwrap();
        assert_eq!(result, "{literal} and Test");
    }

    #[test]
    fn test_format_escaped_backslash() {
        let ctx = HashMap::new();
        let result = format_template("path\\\\to\\\\file", &ctx).unwrap();
        assert_eq!(result, "path\\to\\file");
    }

    // ==================== Unicode Tests ====================

    #[test]
    fn test_format_unicode_text() {
        let mut ctx = HashMap::new();
        ctx.insert("greeting", Value::String("你好".to_string()));
        let result = format_template("Greeting: {greeting}!", &ctx).unwrap();
        assert_eq!(result, "Greeting: 你好!");
    }

    #[test]
    fn test_format_emoji() {
        let mut ctx = HashMap::new();
        ctx.insert("mood", Value::String("😀".to_string()));
        let result = format_template("Feeling {mood} today", &ctx).unwrap();
        assert_eq!(result, "Feeling 😀 today");
    }

    #[test]
    fn test_format_cyrillic() {
        let mut ctx = HashMap::new();
        ctx.insert("name", Value::String("Иван".to_string()));
        let result = format_template("Привет, {name}!", &ctx).unwrap();
        assert_eq!(result, "Привет, Иван!");
    }

    // ==================== Custom Delimiter Integration Tests ====================

    #[test]
    fn test_custom_delimiter_angle() {
        type AngleTpl = Template<'<', '>'>;
        let mut ctx = HashMap::new();
        ctx.insert("name", Value::String("World".to_string()));

        let mut tpl = AngleTpl::parse("Hello, <name>!").unwrap();
        let result = tpl.format(&ctx).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_custom_delimiter_square() {
        type SquareTpl = Template<'[', ']'>;
        let mut ctx = HashMap::new();
        ctx.insert("value", Value::Int(42));

        let mut tpl = SquareTpl::parse("Value: [value]").unwrap();
        let result = tpl.format(&ctx).unwrap();
        assert_eq!(result, "Value: 42");
    }

    #[test]
    fn test_custom_delimiter_same_char() {
        type DollarTpl = Template<'$', '$'>;
        let mut ctx = HashMap::new();
        ctx.insert("var", Value::String("content".to_string()));

        let mut tpl = DollarTpl::parse("Start $var$ end").unwrap();
        let result = tpl.format(&ctx).unwrap();
        assert_eq!(result, "Start content end");
    }

    #[test]
    fn test_custom_delimiter_preserves_default() {
        type AngleTpl = Template<'<', '>'>;
        let mut ctx = HashMap::new();
        ctx.insert("name", Value::String("Test".to_string()));

        let mut tpl = AngleTpl::parse("{not_a_directive} <name>").unwrap();
        let result = tpl.format(&ctx).unwrap();
        assert_eq!(result, "{not_a_directive} Test");
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_format_empty_template() {
        let ctx = HashMap::new();
        let result = format_template("", &ctx).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_whitespace_only() {
        let ctx = HashMap::new();
        let result = format_template("   \n\t  ", &ctx).unwrap();
        assert_eq!(result, "   \n\t  ");
    }

    #[test]
    fn test_format_empty_directive() {
        let ctx = HashMap::new();
        let result = format_template("before{}after", &ctx).unwrap();
        assert_eq!(result, "beforeafter");
    }

    #[test]
    fn test_format_whitespace_directive() {
        let ctx = HashMap::new();
        let result = format_template("before{   }after", &ctx).unwrap();
        assert_eq!(result, "beforeafter");
    }

    #[test]
    fn test_format_very_long_template() {
        let mut ctx = HashMap::new();
        ctx.insert("x", Value::String("X".to_string()));

        let template = "a".repeat(10000) + "{x}" + &"b".repeat(10000);
        let result = format_template(&template, &ctx).unwrap();
        assert_eq!(result.len(), 20001);
        assert!(result.starts_with("aaaa"));
        assert!(result.ends_with("bbbb"));
        assert!(result.contains("X"));
    }

    #[test]
    fn test_format_many_directives() {
        let mut ctx = HashMap::new();
        for i in 0..100 {
            ctx.insert(
                Box::leak(format!("v{}", i).into_boxed_str()) as &'static str,
                Value::Int(i),
            );
        }
        let template: String = (0..100).map(|i| format!("{{v{}}}", i)).collect();
        let result = format_template(&template, &ctx).unwrap();
        let expected: String = (0..100).map(|i| i.to_string()).collect();
        assert_eq!(result, expected);
    }

    // ==================== Value Type Tests ====================

    #[test]
    fn test_format_all_value_types() {
        let mut ctx = HashMap::new();
        ctx.insert("string", Value::String("hello".to_string()));
        ctx.insert("str", Value::Str("world"));
        ctx.insert("int", Value::Int(-42));
        ctx.insert("float", Value::Float(3.14));
        ctx.insert("bool", Value::Bool(true));

        let template = "{string} {str} {int} {float} {bool}";
        let result = format_template(template, &ctx).unwrap();
        assert_eq!(result, "hello world -42 3.14 true");
    }

    #[test]
    fn test_format_zero_values() {
        let mut ctx = HashMap::new();
        ctx.insert("zero_int", Value::Int(0));
        ctx.insert("zero_float", Value::Float(0.0));
        ctx.insert("empty_string", Value::String("".to_string()));
        ctx.insert("false_bool", Value::Bool(false));

        let template = "[{zero_int}][{zero_float}][{empty_string}][{false_bool}]";
        let result = format_template(template, &ctx).unwrap();
        assert_eq!(result, "[0][0][][false]");
    }

    #[test]
    fn test_format_large_numbers() {
        let mut ctx = HashMap::new();
        ctx.insert("big_int", Value::Int(i64::MAX));
        ctx.insert("big_float", Value::Float(f64::MAX));

        let template = "{big_int} {big_float}";
        let result = format_template(template, &ctx).unwrap();
        assert!(result.contains("9223372036854775807"));
    }

    #[test]
    fn test_format_negative_float() {
        let mut ctx = HashMap::new();
        ctx.insert("temp", Value::Float(-273.15));

        let result = format_template("Temperature: {temp}°C", &ctx).unwrap();
        assert_eq!(result, "Temperature: -273.15°C");
    }

    // ==================== Multiline Template Tests ====================

    #[test]
    fn test_format_multiline() {
        let mut ctx = HashMap::new();
        ctx.insert("name", Value::String("Alice".to_string()));
        ctx.insert("age", Value::Int(30));

        let template = "Name: {name}\nAge: {age}\nEnd";
        let result = format_template(template, &ctx).unwrap();
        assert_eq!(result, "Name: Alice\nAge: 30\nEnd");
    }

    #[test]
    fn test_format_template_with_tabs() {
        let mut ctx = HashMap::new();
        ctx.insert("col1", Value::String("A".to_string()));
        ctx.insert("col2", Value::String("B".to_string()));

        let template = "{col1}\t{col2}";
        let result = format_template(template, &ctx).unwrap();
        assert_eq!(result, "A\tB");
    }

    // ==================== Real-World Scenario Tests ====================

    #[test]
    fn test_format_invoice() {
        let mut ctx = HashMap::new();
        ctx.insert("company", Value::String("Acme Corp".to_string()));
        ctx.insert("invoice_no", Value::Int(1001));
        ctx.insert("customer", Value::String("John Doe".to_string()));
        ctx.insert("amount", Value::Float(1234.56));
        ctx.insert("paid", Value::Bool(false));

        let template = r#"
    INVOICE #{invoice_no}
    From: {company}
    To: {customer}
    Amount: ${amount}
    Status: {paid ? "PAID" : "PENDING"}
    "#;
        let result = format_template(template, &ctx).unwrap();
        assert!(result.contains("INVOICE #1001"));
        assert!(result.contains("Acme Corp"));
        assert!(result.contains("John Doe"));
        assert!(result.contains("$1234.56"));
        assert!(result.contains("PENDING"));
    }

    #[test]
    fn test_format_config_file() {
        let mut ctx = HashMap::new();
        ctx.insert("host", Value::String("localhost".to_string()));
        ctx.insert("port", Value::Int(8080));
        ctx.insert("debug", Value::Bool(true));

        let template = r#"server:
      host: {host}
      port: {port}
      debug: {debug}"#;
        let result = format_template(template, &ctx).unwrap();
        assert!(result.contains("host: localhost"));
        assert!(result.contains("port: 8080"));
        assert!(result.contains("debug: true"));
    }

    #[test]
    fn test_format_greeting_by_time() {
        let mut ctx = HashMap::new();
        ctx.insert("time_of_day", Value::String("morning".to_string()));
        ctx.insert("name", Value::String("User".to_string()));

        let template = "{time_of_day | \"morning\" => \"Good morning\" | \"afternoon\" => \"Good afternoon\" | \"evening\" => \"Good evening\" | _ => \"Hello\"}, {name}!";
        let result = format_template(template, &ctx).unwrap();
        assert_eq!(result, "Good morning, User!");
    }

    #[test]
    fn test_format_progress_bar() {
        let mut ctx = HashMap::new();
        ctx.insert("filled", Value::Int(7));
        ctx.insert("empty", Value::Int(3));

        // Note: This won't work directly since repeat needs literal count
        // But we can test with literals
        let template = "[{\"█\" : 7}{\"░\" : 3}]";
        let result = format_template(template, &ctx).unwrap();
        assert_eq!(result, "[███████░░░]");
    }

    #[test]
    fn test_format_sql_template() {
        let mut ctx = HashMap::new();
        ctx.insert("table", Value::String("users".to_string()));
        ctx.insert("id", Value::Int(42));

        let template = "SELECT * FROM {table} WHERE id = {id}";
        let result = format_template(template, &ctx).unwrap();
        assert_eq!(result, "SELECT * FROM users WHERE id = 42");
    }

    #[test]
    fn test_format_log_message() {
        let mut ctx = HashMap::new();
        ctx.insert("level", Value::String("ERROR".to_string()));
        ctx.insert("message", Value::String("Connection failed".to_string()));
        ctx.insert("code", Value::Int(500));

        let template = "[{level}] {message} (code: {code})";
        let result = format_template(template, &ctx).unwrap();
        assert_eq!(result, "[ERROR] Connection failed (code: 500)");
    }
}
