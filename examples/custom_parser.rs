use std::collections::HashMap;

use figura::Template;

#[derive(Debug)]
struct ReverseDirective(String);

impl figura::Directive for ReverseDirective {
    fn execute(&self, _: &figura::Context) -> Result<String, figura::TemplateError> {
        Ok(self.0.chars().rev().collect::<String>())
    }
}

struct DirectiveParser;

impl figura::Parser for DirectiveParser {
    fn parse(_tokens: &[figura::Token], content: &str) -> Option<Box<dyn figura::Directive>> {
        Some(Box::new(ReverseDirective(content.to_owned())))
    }
}

fn main() {
    let template = Template::<'<', '>'>::with_parser::<DirectiveParser>("This will be <reversed>");

    if let Ok(t) = template {
        // The output will be "This will be desrever"
        println!("{}", t.format(&HashMap::new()).unwrap());
    }
}
