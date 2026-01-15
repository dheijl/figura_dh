# Figura - Lightweight Template Engine for Rust

```
.---------------.
| J             |
|   .      //// |
|  / \    |o o| |
| (_,_)   | < | |
|   |     |___| |
|         /   \ |
|        |     ||
'---------------'
```

## Features

- **Variable Substitution** - Replace placeholders with context values
- **Pattern Repetition** - Repeat strings a specified number of times
- **Conditionals** - Ternary operators with comparison support
- **Custom Delimiters** - Use any characters as template boundaries
- **Extensible Parsers** - Implement custom parsing logic
- **Zero-Copy** - Efficient string handling with `Cow`
- **Escape Sequences** - Support for literal delimiter characters

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
figura = "2.0.0"
```

## Quick Start

```rust
use figura::{Template, DefaultParser, Context, Value};
use std::collections::HashMap;

let mut ctx = Context::new();
ctx.insert("name", Value::static_str("Alice"));
ctx.insert("count", Value::Int(3));

let template = Template::<'{', '}'>::compile::<DefaultParser>(
    "Hello {name}! Stars: {'★':count}"
).unwrap();

let output = template.format(&ctx).unwrap();
assert_eq!(output, "Hello Alice! Stars: ★★★");
```

## Syntax

### Variable Substitution

```rust
let template = Template::<'{', '}'>::compile::<DefaultParser>(
    "User: {username}, Age: {age}"
).unwrap();

ctx.insert("username", Value::static_str("Bob"));
ctx.insert("age", Value::Int(25));
// Output: "User: Bob, Age: 25"
```

### Literals

```rust
let template = Template::<'{', '}'>::compile::<DefaultParser>(
    "Message: {'Hello World'}"
).unwrap();
// Output: "Message: Hello World"
```

### Pattern Repetition

```rust
let template = Template::<'{', '}'>::compile::<DefaultParser>(
    "{'-':50}\n{title}\n{'-':50}"
).unwrap();

ctx.insert("title", Value::static_str("HEADER"));
// Output:
// --------------------------------------------------
// HEADER
// --------------------------------------------------
```

### Conditionals

Simple boolean conditions:

```rust
let template = Template::<'{', '}'>::compile::<DefaultParser>(
    "Status: {active ? 'Online' : 'Offline'}"
).unwrap();

ctx.insert("active", Value::Bool(true));
// Output: "Status: Online"
```

With comparisons:

```rust
let template = Template::<'{', '}'>::compile::<DefaultParser>(
    "Access: {age >= 18 ? 'Granted' : 'Denied'}"
).unwrap();

ctx.insert("age", Value::Int(21));
// Output: "Access: Granted"
```

Supported operators: `==`, `!=`, `>`, `<`, `>=`, `<=`

Logical NOT:

```rust
let template = Template::<'{', '}'>::compile::<DefaultParser>(
    "{!enabled ? 'Disabled' : 'Enabled'}"
).unwrap();
```

### Escaped Delimiters

```rust
let template = Template::<'{', '}'>::compile::<DefaultParser>(
    "Literal braces: {{not a variable}}"
).unwrap();
// Output: "Literal braces: {not a variable}"
```

## Custom Delimiters

Use any characters as delimiters:

```rust
// Angle brackets
let template = Template::<'<', '>'>::compile::<DefaultParser>(
    "Hello <name>!"
).unwrap();

// Square brackets
let template = Template::<'[', ']'>::compile::<DefaultParser>(
    "Value: [count]"
).unwrap();

// Same character for both
let template = Template::<'%', '%'>::compile::<DefaultParser>(
    "Data: %value%"
).unwrap();
```

## Value Types

Figura supports four value types:

```rust
// String (zero-copy when possible)
ctx.insert("name", Value::static_str("Alice"));
ctx.insert("name", Value::owned_str(String::from("Bob")));

// Integer
ctx.insert("count", Value::Int(42));

// Float
ctx.insert("score", Value::Float(95.5));

// Boolean
ctx.insert("active", Value::Bool(true));
```

## Custom Parsers

Implement the `Parser` trait to create custom parsing logic:

```rust
use figura::{Parser, Token, Directive, Argument};

struct MathParser;

impl Parser for MathParser {
    fn parse(tokens: &[Token]) -> Option<Box<dyn Directive>> {
        match tokens {
            [Token::Ident(left), Token::Plus, Token::Ident(right)] => {
                Some(Box::new(AddDirective {
                    left: left.to_string(),
                    right: right.to_string(),
                }))
            }
            _ => Some(Box::new(EmptyDirective)),
        }
    }
}

// Implement custom directive
struct AddDirective {
    left: String,
    right: String,
}

impl Directive for AddDirective {
    fn exec(&self, ctx: &Context) -> Result<Cow<'static, str>, DirectiveError> {
        // Custom execution logic
    }
}

// Use custom parser
let template = Template::<'{', '}'>::compile::<MathParser>(
    "{x + y}"
).unwrap();
```

## API Overview

### Core Types

- `Template<O, C>` - Compiled template with open/close delimiters
- `Value` - Runtime values (String, Int, Float, Bool)
- `Context` - HashMap of variable names to values
- `DefaultParser` - Built-in parser implementation
- `Parser` - Trait for custom parsers
- `Directive` - Trait for executable template components


## License

MIT License - Copyright (c) Saverio Scagnoli

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
