# Figura 🃏

A lightweight, flexible template formatting engine for Rust with support for custom delimiters and extensible directive parsing.

## Features

- **Variable Substitution**: `{name}` → context values
- **Repetition**: `{pattern:count}` → repeat patterns
- **Conditionals**: `{condition?true_part:false_part}` → conditional rendering
- **Custom Delimiters**: Use any characters as template boundaries
- **Extensible Parsers**: Create custom directive handlers
- **Escape Sequences**: `{{` and `}}` for literal delimiters
- **Alignment Support**: `{value<}`, `{value>}`, `{value^}` for formatting hints

## Quick Start

```rust
use figura::{Template, Context, Value};
use std::collections::HashMap;

// Basic variable substitution
let template = Template::parse("Hello, {name}!")?;
let mut ctx = HashMap::new();
ctx.insert("name", Value::String("World".into()));
assert_eq!(template.format(&ctx)?, "Hello, World!");

// Repetition
let template = Template::parse("Echo: {word:3}")?;
let mut ctx = HashMap::new();
ctx.insert("word", Value::String("hi".into()));
assert_eq!(template.format(&ctx)?, "Echo: hihihi");

// Conditionals
let template = Template::parse("{logged_in?Welcome back!:Please log in}")?;
let mut ctx = HashMap::new();
ctx.insert("logged_in", Value::Bool(true));
assert_eq!(template.format(&ctx)?, "Welcome back!");

// Custom delimiters
let template = Template::<'[', ']'>::parse("Hello [name]!")?;
```

## Value Types

The engine supports multiple value types:

```rust
ctx.insert("name", Value::String("Alice".into()));
ctx.insert("age", Value::Int(30));
ctx.insert("score", Value::Float(95.5));
ctx.insert("active", Value::Bool(true));
```

## Custom Parsers

Extend functionality by implementing the `Parser` trait:

```rust
struct CustomParser;

impl Parser for CustomParser {
    fn parse(tokens: &[Token], content: &str) -> Option<Box<dyn Directive>> {
        // Your custom parsing logic
        // Fall back to DefaultParser if needed
        DefaultParser::parse(tokens, content)
    }
}

let template = Template::with_parser::<CustomParser>("Your template")?;
```

## Error Handling

The engine provides detailed error information:

- `MissingDelimiter`: Unmatched template delimiters
- `NoValueFound`: Referenced variable not in context
- `ExecutionError`: Runtime directive execution failures
- `DirectiveParsing`: Parser unable to handle directive

## License

MIT
