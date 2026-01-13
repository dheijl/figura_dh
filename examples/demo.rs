use figura::{Context, DefaultParser, Template, Value};

type CBTemplate<'a> = Template<'a, '{', '}'>;

fn main() {
    let mut ctx = Context::new();
    let mut output = String::new();
    let template = CBTemplate::compile::<DefaultParser>("Hello, my name is {name}!").unwrap();

    ctx.insert("name", Value::StaticStr("John"));
    ctx.insert("count", Value::Int(4));

    if let Err(e) = template.format(&ctx, &mut output) {
        eprintln!("Error while formatting template: {}", e);
    }

    println!("{}", output);

    let mut output = String::new();
    let template = CBTemplate::compile::<DefaultParser>(
        "This will be repeated {count} times {Abbacchio:count}",
    )
    .unwrap();

    if let Err(e) = template.format(&ctx, &mut output) {
        eprintln!("Error while formatting template: {}", e);
    }

    println!("{}", output);
}
