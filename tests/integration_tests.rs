use figura::{Context, DefaultParser, Template, Value};
use std::borrow::Cow;

#[test]
fn test_template_empty_string() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("").unwrap();
    let ctx = Context::new();
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "");
}

#[test]
fn test_template_no_placeholders() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("Hello World").unwrap();
    let ctx = Context::new();
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Hello World");
}

#[test]
fn test_template_single_placeholder() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("Hello {name}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("Alice")));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Hello Alice");
}

#[test]
fn test_template_multiple_placeholders() {
    let template =
        Template::<'{', '}'>::compile::<DefaultParser>("Hello {name}, you are {age} years old")
            .unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("Bob")));
    ctx.insert("age", Value::Int(25));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Hello Bob, you are 25 years old");
}

#[test]
fn test_template_repeated_placeholder() {
    let template =
        Template::<'{', '}'>::compile::<DefaultParser>("{name} and {name} are friends").unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("Charlie")));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Charlie and Charlie are friends");
}

#[test]
fn test_template_missing_value() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("Hello {name}").unwrap();
    let ctx = Context::new();
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Hello ");
}

#[test]
fn test_template_escaped_delimiter() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("{{name}}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("test")));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "{name}");
}

#[test]
fn test_template_escaped_opening_delimiter() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("Test {{").unwrap();
    let ctx = Context::new();
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Test {");
}

#[test]
fn test_template_mixed_escaped_and_normal() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("{{escaped}} {normal}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("normal", Value::Str(Cow::Borrowed("value")));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "{escaped} value");
}

#[test]
fn test_template_with_newlines() {
    let template =
        Template::<'{', '}'>::compile::<DefaultParser>("Line 1: {a}\nLine 2: {b}\nLine 3: {c}")
            .unwrap();
    let mut ctx = Context::new();
    ctx.insert("a", Value::Int(1));
    ctx.insert("b", Value::Int(2));
    ctx.insert("c", Value::Int(3));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Line 1: 1\nLine 2: 2\nLine 3: 3");
}

#[test]
fn test_template_adjacent_placeholders() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("{a}{b}{c}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("a", Value::Str(Cow::Borrowed("A")));
    ctx.insert("b", Value::Str(Cow::Borrowed("B")));
    ctx.insert("c", Value::Str(Cow::Borrowed("C")));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "ABC");
}

#[test]
fn test_template_placeholder_at_start() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("{name} says hello").unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("Alice")));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Alice says hello");
}

#[test]
fn test_template_placeholder_at_end() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("Hello {name}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("Bob")));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Hello Bob");
}

#[test]
fn test_template_only_placeholder() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("{value}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("value", Value::Str(Cow::Borrowed("test")));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "test");
}

#[test]
fn test_template_empty_placeholder() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("test {}").unwrap();
    let ctx = Context::new();
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "test ");
}

#[test]
fn test_template_whitespace_in_placeholder() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("{ name }").unwrap();
    let mut ctx = Context::new();
    let mut buf = String::new();

    ctx.insert("name", Value::Str(Cow::Owned(String::from("John"))));

    template.format(&ctx, &mut buf).unwrap();

    assert_eq!(buf, "John");
}

#[test]
fn test_template_unclosed_delimiter() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("Hello {name");
    assert!(template.is_err());
    assert!(template.unwrap_err().contains("Unclosed delimiter"));
}

#[test]
fn test_template_nested_delimiters() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("Test {a{b}}").unwrap();
    let ctx = Context::new();
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Test ");
}

#[test]
fn test_template_angle_brackets() {
    let template = Template::<'<', '>'>::compile::<DefaultParser>("Hello <name>").unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("World")));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Hello World");
}

#[test]
fn test_template_square_brackets() {
    let template = Template::<'[', ']'>::compile::<DefaultParser>("Value: [x]").unwrap();
    let mut ctx = Context::new();
    ctx.insert("x", Value::Int(42));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Value: 42");
}

#[test]
fn test_template_dollar_sign() {
    let template = Template::<'$', '$'>::compile::<DefaultParser>("Price: $amount$").unwrap();
    let mut ctx = Context::new();
    ctx.insert("amount", Value::Float(19.99));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Price: 19.99");
}

#[test]
fn test_template_percent_sign() {
    let template = Template::<'%', '%'>::compile::<DefaultParser>("Complete: %percent%").unwrap();
    let mut ctx = Context::new();
    ctx.insert("percent", Value::Int(75));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Complete: 75");
}

#[test]
fn test_template_int_values() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("a={a}, b={b}, c={c}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("a", Value::Int(0));
    ctx.insert("b", Value::Int(-100));
    ctx.insert("c", Value::Int(999999));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "a=0, b=-100, c=999999");
}

#[test]
fn test_template_float_values() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("pi={pi}, e={e}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("pi", Value::Float(3.14159));
    ctx.insert("e", Value::Float(2.71828));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "pi=3.14159, e=2.71828");
}

#[test]
fn test_template_bool_values() {
    let template =
        Template::<'{', '}'>::compile::<DefaultParser>("active={active}, enabled={enabled}")
            .unwrap();
    let mut ctx = Context::new();
    ctx.insert("active", Value::Bool(true));
    ctx.insert("enabled", Value::Bool(false));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "active=true, enabled=false");
}

#[test]
fn test_template_mixed_value_types() {
    let template =
        Template::<'{', '}'>::compile::<DefaultParser>("str={s}, int={i}, float={f}, bool={b}")
            .unwrap();
    let mut ctx = Context::new();
    ctx.insert("s", Value::Str(Cow::Borrowed("text")));
    ctx.insert("i", Value::Int(42));
    ctx.insert("f", Value::Float(3.14));
    ctx.insert("b", Value::Bool(true));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "str=text, int=42, float=3.14, bool=true");
}

#[test]
fn test_template_reuse() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("Hello {name}").unwrap();

    let mut ctx1 = Context::new();
    ctx1.insert("name", Value::Str(Cow::Borrowed("Alice")));
    let mut buf1 = String::new();
    template.format(&ctx1, &mut buf1).unwrap();
    assert_eq!(buf1, "Hello Alice");

    let mut ctx2 = Context::new();
    ctx2.insert("name", Value::Str(Cow::Borrowed("Bob")));
    let mut buf2 = String::new();
    template.format(&ctx2, &mut buf2).unwrap();
    assert_eq!(buf2, "Hello Bob");
}

#[test]
fn test_template_unicode_text() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("你好 {name}，欢迎！").unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("世界")));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "你好 世界，欢迎！");
}

#[test]
fn test_template_emoji() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("Hello {name} 👋").unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("🌍")));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Hello 🌍 👋");
}

#[test]
fn test_template_complex_document() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>(
        "Name: {name}\nAge: {age}\nEmail: {email}\nActive: {active}",
    )
    .unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("John Doe")));
    ctx.insert("age", Value::Int(30));
    ctx.insert("email", Value::Str(Cow::Borrowed("john@example.com")));
    ctx.insert("active", Value::Bool(true));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(
        buf,
        "Name: John Doe\nAge: 30\nEmail: john@example.com\nActive: true"
    );
}

#[test]
fn test_template_html_like() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>(
        "<div class=\"user\">\n  <h1>{name}</h1>\n  <p>Age: {age}</p>\n</div>",
    )
    .unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("Alice")));
    ctx.insert("age", Value::Int(25));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(
        buf,
        "<div class=\"user\">\n  <h1>Alice</h1>\n  <p>Age: 25</p>\n</div>"
    );
}

#[test]
fn test_template_json_like() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>(
        r#"{{ "name": "{name}", "age": {age}, "active": {active} }}"#,
    )
    .unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("Bob")));
    ctx.insert("age", Value::Int(30));
    ctx.insert("active", Value::Bool(true));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, r#"{ "name": "Bob", "age": 30, "active": true }"#);
}

#[test]
fn test_template_url_like() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>(
        "https://example.com/user/{id}/profile?active={active}",
    )
    .unwrap();
    let mut ctx = Context::new();
    ctx.insert("id", Value::Int(123));
    ctx.insert("active", Value::Bool(true));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "https://example.com/user/123/profile?active=true");
}

#[test]
fn test_template_sql_like() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>(
        "SELECT * FROM users WHERE id = {id} AND active = {active}",
    )
    .unwrap();
    let mut ctx = Context::new();
    ctx.insert("id", Value::Int(42));
    ctx.insert("active", Value::Bool(true));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "SELECT * FROM users WHERE id = 42 AND active = true");
}

#[test]
fn test_template_very_long_text() {
    let long_text = "a".repeat(10000);
    let template = Template::<'{', '}'>::compile::<DefaultParser>(&long_text).unwrap();
    let ctx = Context::new();
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, long_text);
}

#[test]
fn test_template_many_placeholders() {
    let template_str = (0..100)
        .map(|i| format!("{{var{}}}", i))
        .collect::<Vec<_>>()
        .join(" ");
    let template = Template::<'{', '}'>::compile::<DefaultParser>(&template_str).unwrap();

    let mut ctx = Context::new();
    for i in 0..100 {
        ctx.insert(
            Box::leak(format!("var{}", i).into_boxed_str()),
            Value::Int(i),
        );
    }

    let mut buf = String::new();
    template.format(&ctx, &mut buf).unwrap();

    let expected = (0..100)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(buf, expected);
}

#[test]
fn test_template_special_characters_in_text() {
    let template =
        Template::<'{', '}'>::compile::<DefaultParser>("Test: @#$%^&*()_+-=[]\\|;':\",./<>?")
            .unwrap();
    let ctx = Context::new();
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Test: @#$%^&*()_+-=[]\\|;':\",./<>?");
}

#[test]
fn test_template_tabs_and_spaces() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("\t{a}\t\t{b}    {c}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("a", Value::Int(1));
    ctx.insert("b", Value::Int(2));
    ctx.insert("c", Value::Int(3));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "\t1\t\t2    3");
}

#[test]
fn test_template_multiple_consecutive_escapes() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("{{{{{{").unwrap();
    let ctx = Context::new();
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "{{{");
}

#[test]
fn test_template_buffer_reuse() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("Hello {name}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("Alice")));

    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Hello Alice");

    buf.clear();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "Hello Alice");
}

#[test]
fn test_template_partial_context() {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("{a} {b} {c}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("a", Value::Int(1));
    ctx.insert("c", Value::Int(3));
    let mut buf = String::new();

    template.format(&ctx, &mut buf).unwrap();
    assert_eq!(buf, "1  3");
}
