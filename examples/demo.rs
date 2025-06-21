use figura::{DefaultParser, Template, Value};
use std::collections::HashMap;

fn main() {
    let context = HashMap::from([
        ("name", Value::String("Gyro".to_string())),
        ("age", Value::Int(25)),
        ("depth", Value::Int(2)),
    ]);

    if let Ok(t) = Template::<'{', '}'>::parse::<DefaultParser>("Hello! My name is {name}") {
        println!("{}", t.format(&context).unwrap());
    }

    if let Ok(t) = Template::<'[', ']'>::parse::<DefaultParser>(
        "[ :depth] This will have [depth] spaces in the front!",
    ) {
        println!("{}", t.format(&context).unwrap());
    }
}
