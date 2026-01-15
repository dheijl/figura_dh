use figura::{Context, DefaultParser, Template, Value};

fn main() {
    let mut ctx = Context::new();
    ctx.insert("title", Value::static_str("Custom Delimiters"));
    ctx.insert("count", Value::Int(5));

    let template =
        Template::<'<', '>'>::compile::<DefaultParser>("Title: <title> | Stars: <'★':count>")
            .unwrap();

    let output = template.format(&ctx).unwrap();
    println!("{}", output);

    let template = Template::<'[', ']'>::compile::<DefaultParser>(
        "Using brackets: [title] with [count] items",
    )
    .unwrap();

    let output = template.format(&ctx).unwrap();
    println!("{}", output);

    let template =
        Template::<'%', '%'>::compile::<DefaultParser>("Percent signs: %title% %%escaped%%")
            .unwrap();

    let output = template.format(&ctx).unwrap();
    println!("{}", output);
}
