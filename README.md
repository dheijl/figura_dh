# Figura - template

A flexible and extensible template formatting engine for Rust that supports custom delimiters, alignment options, and pluggable directive parsers.

## Features

- **Flexible Delimiters**: Use any characters as opening and closing delimiters (default: `{` and `}`)
- **Alignment Support**: Built-in support for left (`<`), right (`>`), and center (`^`) alignment
- **Extensible Parser System**: Create custom directive parsers for domain-specific templating needs
- **Built-in Directives**: Variable replacement and pattern repetition out of the box
- **Escape Sequences**: Support for escaping delimiters when needed
- **Type Safety**: Strong typing with clear error handling

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
figura = "0.0.1"
```

### Basic Usage

```rust
use figura::{Template, Context, Value, DefaultParser};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a template
    let template = Template::<'{', '}'}>::parse::<DefaultParser>("Hello, {name}! You are {age} years old.")?;

    // Create context with values
    let mut context: Context = HashMap::new();
    context.insert("name", Value::String("Alice".to_string()));
    context.insert("age", Value::Int(30));

    // Format the template
    let result = template.format(&context)?;
    println!("{}", result); // Output: Hello, Alice! You are 30 years old.

    Ok(())
}
```

## Core Concepts

### Values

The engine supports three basic value types:

```rust
use figura::Value;

let string_val = Value::String("Hello".to_string());
let int_val = Value::Int(42);
let bool_val = Value::Bool(true);
```

### Context

Context is a key-value map that provides data for template rendering:

```rust
use figura::{Context, Value};
use std::collections::HashMap;

let mut ctx: Context = HashMap::new();
ctx.insert("user", Value::String("John".to_string()));
ctx.insert("score", Value::Int(95));
ctx.insert("active", Value::Bool(true));
```

## Built-in Directives

### Variable Replacement

Replace placeholders with context values:

```rust
let template = Template::<'{', '}'>::parse::<DefaultParser>("Welcome, {username}!")?;
```

### Pattern Repetition

Repeat patterns a specified number of times:

```rust
// Repeat literal pattern
let template = Template::<'{', '}'>::parse::<DefaultParser>("{*:5}")?; // Outputs: *****

// Repeat using context values
let mut ctx = HashMap::new();
ctx.insert("char", Value::String("-".to_string()));
ctx.insert("count", Value::Int(3));
let template = Template::<'{', '}'>::parse::<DefaultParser>("{char:count}")?; // Outputs: ---
```

## Alignment

Control text alignment using alignment specifiers:

```rust
// Left alignment (default)
let template = Template::<'{', '}'>::parse::<DefaultParser>("{name<}")?;

// Right alignment
let template = Template::<'{', '}'>::parse::<DefaultParser>("{name>}")?;

// Center alignment
let template = Template::<'{', '}'>::parse::<DefaultParser>("{name^}")?;

// Check detected alignment
println!("Alignment: {:?}", template.alignment());
```

## Custom Delimiters

Use any characters as delimiters:

```rust
// Square brackets
let template = Template::<'[', ']'>::parse::<DefaultParser>("Hello [name]!")?;

// Same character for both (useful for LaTeX-style)
let template = Template::<'|', '|'>::parse::<DefaultParser>("Value: |variable|")?;
```

## Escape Sequences

Escape delimiters when you need literal characters:

```rust
// Double delimiters for escaping
let template = Template::<'{', '}'>::parse::<DefaultParser>("{{not a directive}} but {this_is}")?;
// Outputs: {not a directive} but [value of this_is]
```

## Custom Parsers

Create your own directive parsers for specialized templating needs:

```rust
use figura::{Parser, Directive, Token, Context, TemplateError};

#[derive(Debug)]
struct UppercaseDirective(String);

impl Directive for UppercaseDirective {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError> {
        if let Some(value) = ctx.get(&self.0) {
            Ok(value.to_string().to_uppercase())
        } else {
            Err(TemplateError::NoValueFound(self.0.clone()))
        }
    }
}

struct CustomParser;

impl Parser for CustomParser {
    fn parse(tokens: &[Token], content: &str) -> Option<Box<dyn Directive>> {
        match tokens {
            // {variable:upper}
            [Token::Literal(var), Token::Symbol(':'), Token::Literal(cmd)]
                if cmd == "upper" => {
                Some(Box::new(UppercaseDirective(var.clone())))
            }

            // Fallback to default parsing
            _ => DefaultParser::parse(tokens, content)
        }
    }
}

// Usage
let template = Template::<'{', '}'>::parse::<CustomParser>("{name:upper}")?;
```

## Advanced Examples

### Complex Template with Multiple Features

```rust
use figura::{Template, Context, Value, DefaultParser};
use std::collections::HashMap;

let template_str = r#"
=== User Report ===
Name: {name^}
Score: {stars:score} ({score}/5)
Status: {status}
{{Note: This is a literal brace}}
"#;

let template = Template::<'{', '}'>::parse::<DefaultParser>(template_str)?;

let mut context = HashMap::new();
context.insert("name", Value::String("Alice Smith".to_string()));
context.insert("stars", Value::String("★".to_string()));
context.insert("score", Value::Int(4));
context.insert("status", Value::String("Active".to_string()));

let result = template.format(&context)?;
println!("{}", result);
```

### Working with Different Data Types

```rust
// The engine automatically converts values to strings
let mut ctx = HashMap::new();
ctx.insert("count", Value::Int(42));
ctx.insert("is_valid", Value::Bool(true));
ctx.insert("message", Value::String("Hello".to_string()));

let template = Template::<'{', '}'>::parse::<DefaultParser>(
    "Count: {count}, Valid: {is_valid}, Message: {message}"
)?;

// Output: Count: 42, Valid: true, Message: Hello
```

## Performance Considerations

- Templates are parsed once and can be reused multiple times with different contexts
- The engine uses efficient string building and minimal allocations during formatting
- Consider caching parsed templates for frequently used patterns

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
