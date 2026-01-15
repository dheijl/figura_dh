use figura::{Context, DefaultParser, Template, Value};

fn main() {
    let mut ctx = Context::new();

    ctx.insert("is_admin", Value::Bool(true));
    ctx.insert("score", Value::Int(85));
    ctx.insert("status", Value::static_str("active"));
    ctx.insert("temperature", Value::Float(22.5));

    let template =
        Template::<'{', '}'>::compile::<DefaultParser>("Access: {is_admin ? 'GRANTED' : 'DENIED'}")
            .unwrap();
    println!("{}", template.format(&ctx).unwrap());

    let template =
        Template::<'{', '}'>::compile::<DefaultParser>("Grade: {score >= 90 ? 'A' : 'not-A'}")
            .unwrap();
    println!("{}", template.format(&ctx).unwrap());

    ctx.insert("score", Value::Int(88));
    let template =
        Template::<'{', '}'>::compile::<DefaultParser>("Grade: {score >= 80 ? 'B' : 'below-B'}")
            .unwrap();
    println!("{}", template.format(&ctx).unwrap());

    ctx.insert("score", Value::Int(72));
    let template =
        Template::<'{', '}'>::compile::<DefaultParser>("Grade: {score < 80 ? 'C' : 'above-C'}")
            .unwrap();
    println!("{}", template.format(&ctx).unwrap());

    ctx.insert("score", Value::Int(85));

    let template = Template::<'{', '}'>::compile::<DefaultParser>(
        "Status: {status == 'active' ? 'Online' : 'Offline'}",
    )
    .unwrap();
    println!("{}", template.format(&ctx).unwrap());

    let template = Template::<'{', '}'>::compile::<DefaultParser>(
        "Temp: {temperature > 20.0 ? 'Warm' : 'Cold'}",
    )
    .unwrap();
    println!("{}", template.format(&ctx).unwrap());

    ctx.insert("is_admin", Value::Bool(false));
    let template = Template::<'{', '}'>::compile::<DefaultParser>(
        "Access: {!is_admin ? 'DENIED' : 'GRANTED'}",
    )
    .unwrap();
    println!("{}", template.format(&ctx).unwrap());
}
