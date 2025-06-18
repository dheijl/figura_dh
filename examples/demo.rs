use figura::{CurlyBrackets, Template};

fn main() {
    let template = Template::<CurlyBrackets>::parse("Hello, I am {name}!");

    if let Ok(t) = template {
        println!("{:?}", t.parts())
    }
}
