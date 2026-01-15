use figura::{Context, DefaultParser, Template, Value};

fn main() {
    let mut ctx = Context::new();
    ctx.insert("char", Value::static_str("="));
    ctx.insert("count", Value::Int(50));
    ctx.insert("title", Value::static_str("HEADER"));

    let template =
        Template::<'{', '}'>::compile::<DefaultParser>("{char:count}\n{title}\n{char:count}")
            .unwrap();

    println!("{}", template.format(&ctx).unwrap());

    ctx.insert("pattern", Value::static_str("* "));
    ctx.insert("times", Value::Int(10));

    let template =
        Template::<'{', '}'>::compile::<DefaultParser>("Pattern: {pattern:times}").unwrap();

    println!("{}", template.format(&ctx).unwrap());

    let template =
        Template::<'{', '}'>::compile::<DefaultParser>("{'#':5} Progress Bar {'#':5}").unwrap();

    println!("{}", template.format(&Context::new()).unwrap());
}
