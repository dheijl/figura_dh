use std::collections::HashMap;

use figura::{CurlyBrackets, Template, Value};

fn main() {
    let context = HashMap::from([
        ("depth".to_string(), Value::Int(2)),
        ("name".to_string(), Value::String("John".to_string())),
        ("age".to_string(), Value::Int(25)),
    ]);
    let template = Template::<CurlyBrackets>::parse(
        "{ :depth} Hello, I am {name and I am {age} years old!",
    );

    match template {
        Ok(t) => println!("{}", t.format(&context).unwrap()),
        Err(e) => eprintln!("{}", e),
    }
}
