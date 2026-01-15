use figura::{Context, DefaultParser, Template, Value};

fn main() {
    let mut ctx = Context::new();
    ctx.insert("name", Value::static_str("Alice"));
    ctx.insert("age", Value::Int(30));
    ctx.insert("city", Value::static_str("Boston"));

    let template = Template::<'{', '}'>::compile::<DefaultParser>(
        "Hello {name}! You are {age} years old and live in {city}.",
    )
    .unwrap();

    let output = template.format(&ctx).unwrap();
    println!("{}", output);
}
