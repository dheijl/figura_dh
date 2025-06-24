use figura::{Template, Value};
use std::collections::HashMap;

fn main() {
    let ctx = HashMap::from([
        ("name", Value::String("Gyro".to_string())),
        ("age", Value::Int(25)),
        ("depth", Value::Int(2)),
        ("admin", Value::Bool(true)),
    ]);

    let template = Template::<'{', '}'>::parse("Hello! My name is {name}").unwrap();

    println!("{}", template.format(&ctx).unwrap()); // Output: Hello! My name is Gyro

    let template =
        Template::<'[', ']'>::parse("[ :depth] This will have [[depth]] spaces in the front!")
            .unwrap();

    println!("{}", template.format(&ctx).unwrap()); // Output: "[  2] This will have [depth] spaces in the front!"

    let template = Template::<'{', '}'>::parse("Current user: {admin?Admin:Guest}").unwrap();

    println!("{}", template.format(&ctx).unwrap()); // Output: Current user: Admin
}
