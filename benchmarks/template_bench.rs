use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::collections::HashMap;
use std::{borrow::Cow, hint::black_box};

use figura::{Context, DefaultParser, Template, TemplateLexer, Token, Value};

fn bench_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");

    let inputs = [
        ("simple_ident", "foo"),
        ("number", "12345"),
        ("float", "123.456"),
        ("string_literal", r#""hello world""#),
        ("string_escaped", r#""hello\nworld\t!""#),
        ("expression", "foo == bar && baz != 42"),
        ("complex", r#"items:count + 10 >= limit || name == "test""#),
    ];

    for (name, input) in inputs {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("tokenize", name), input, |b, input| {
            b.iter(|| {
                let lexer = TemplateLexer::new(black_box(input));
                let tokens: Vec<Token> = lexer.collect();
                black_box(tokens)
            });
        });
    }

    group.finish();
}

fn bench_template_compile(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile");

    let templates = [
        ("literal_only", "Hello, World!"),
        ("single_var", "Hello, {name}!"),
        (
            "multiple_vars",
            "Hello, {first} {last}! You are {age} years old.",
        ),
        ("escaped_delim", "Use {{braces}} for templates"),
        ("repeat", "Stars: {*:10}"),
        (
            "mixed",
            "Dear {name},\n\nYour order #{order_id} is ready.\nTotal: ${total}\n\nThanks!",
        ),
    ];

    for (name, template) in templates {
        group.throughput(Throughput::Bytes(template.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("compile", name),
            template,
            |b, template| {
                b.iter(|| {
                    Template::<'{', '}'>::compile::<DefaultParser>(black_box(template)).unwrap()
                });
            },
        );
    }

    // Large template
    let large_template = "{greeting} {name}! ".repeat(100);
    group.throughput(Throughput::Bytes(large_template.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("compile", "large_100_vars"),
        &large_template,
        |b, template| {
            b.iter(|| Template::<'{', '}'>::compile::<DefaultParser>(black_box(template)).unwrap());
        },
    );

    group.finish();
}

fn bench_template_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("format");

    // Single variable
    {
        let template = Template::<'{', '}'>::compile::<DefaultParser>("Hello, {name}!").unwrap();
        let mut ctx: Context = HashMap::new();
        ctx.insert("name", Value::Str(Cow::Borrowed("World")));

        group.bench_function("single_var", |b| {
            b.iter(|| {
                let mut buf = String::new();
                template.format(black_box(&ctx), &mut buf).unwrap();
                black_box(buf)
            });
        });

        group.bench_function("single_var_preallocated", |b| {
            b.iter(|| {
                let mut buf = String::with_capacity(64);
                template.format(black_box(&ctx), &mut buf).unwrap();
                black_box(buf)
            });
        });
    }

    // Multiple variables
    {
        let template = Template::<'{', '}'>::compile::<DefaultParser>(
            "Hello, {first} {last}! You are {age} years old and have ${balance}.",
        )
        .unwrap();

        let mut ctx: Context = HashMap::new();
        ctx.insert("first", Value::Str(Cow::Borrowed("John")));
        ctx.insert("last", Value::Str(Cow::Borrowed("Doe")));
        ctx.insert("age", Value::Int(30));
        ctx.insert("balance", Value::Float(1234.56));

        group.bench_function("multiple_vars", |b| {
            b.iter(|| {
                let mut buf = String::new();
                template.format(black_box(&ctx), &mut buf).unwrap();
                black_box(buf)
            });
        });
    }

    // Repeat directive
    {
        let template = Template::<'{', '}'>::compile::<DefaultParser>("{pattern:count}").unwrap();

        let mut ctx: Context = HashMap::new();
        ctx.insert("pattern", Value::Str(Cow::Borrowed("*")));
        ctx.insert("count", Value::Int(50));

        group.bench_function("repeat_directive", |b| {
            b.iter(|| {
                let mut buf = String::new();
                template.format(black_box(&ctx), &mut buf).unwrap();
                black_box(buf)
            });
        });
    }

    // Literal only (no substitution)
    {
        let template = Template::<'{', '}'>::compile::<DefaultParser>(
            "This is a static string with no variables at all.",
        )
        .unwrap();
        let ctx: Context = HashMap::new();

        group.bench_function("literal_only", |b| {
            b.iter(|| {
                let mut buf = String::new();
                template.format(black_box(&ctx), &mut buf).unwrap();
                black_box(buf)
            });
        });
    }

    group.finish();
}

fn bench_lexer_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_throughput");

    for size in [100, 1000, 10000] {
        let input = "identifier ".repeat(size);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("identifiers", size), &input, |b, input| {
            b.iter(|| {
                let lexer = TemplateLexer::new(black_box(input));
                black_box(lexer.count())
            });
        });
    }

    for size in [100, 1000, 10000] {
        let input = "12345 ".repeat(size);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("numbers", size), &input, |b, input| {
            b.iter(|| {
                let lexer = TemplateLexer::new(black_box(input));
                black_box(lexer.count())
            });
        });
    }

    group.finish();
}

fn bench_e2e(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");

    // Realistic email template
    let email_template = r#"Dear {name},

Thank you for your order #{order_id}!

Items ordered: {item_count}
Subtotal: ${subtotal}
Tax: ${tax}
Total: ${total}

Your order will be shipped to:
{address}

Best regards,
The Team"#;

    let mut ctx: Context = HashMap::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("Alice Smith")));
    ctx.insert("order_id", Value::Int(12345));
    ctx.insert("item_count", Value::Int(3));
    ctx.insert("subtotal", Value::Float(99.99));
    ctx.insert("tax", Value::Float(8.50));
    ctx.insert("total", Value::Float(108.49));
    ctx.insert(
        "address",
        Value::Str(Cow::Borrowed("123 Main St, City, ST 12345")),
    );

    group.bench_function("email_compile_and_format", |b| {
        b.iter(|| {
            let template =
                Template::<'{', '}'>::compile::<DefaultParser>(black_box(email_template)).unwrap();
            let mut buf = String::new();
            template.format(&ctx, &mut buf).unwrap();
            black_box(buf)
        });
    });

    let template = Template::<'{', '}'>::compile::<DefaultParser>(email_template).unwrap();

    group.bench_function("email_format_only", |b| {
        b.iter(|| {
            let mut buf = String::new();
            template.format(black_box(&ctx), &mut buf).unwrap();
            black_box(buf)
        });
    });

    group.finish();
}

fn bench_custom_delimiters(c: &mut Criterion) {
    let mut group = c.benchmark_group("custom_delimiters");

    let template_curly = "Hello, {name}!";
    let template_angle = "Hello, <name>";
    let template_percent = "Hello, %name%";

    let mut ctx: Context = HashMap::new();
    ctx.insert("name", Value::Str(Cow::Borrowed("World")));

    group.bench_function("curly_braces", |b| {
        b.iter(|| {
            let t =
                Template::<'{', '}'>::compile::<DefaultParser>(black_box(template_curly)).unwrap();
            let mut buf = String::new();
            t.format(&ctx, &mut buf).unwrap();
            black_box(buf)
        });
    });

    group.bench_function("angle_brackets", |b| {
        b.iter(|| {
            let t =
                Template::<'<', '>'>::compile::<DefaultParser>(black_box(template_angle)).unwrap();
            let mut buf = String::new();
            t.format(&ctx, &mut buf).unwrap();
            black_box(buf)
        });
    });

    group.bench_function("percent_signs", |b| {
        b.iter(|| {
            let t = Template::<'%', '%'>::compile::<DefaultParser>(black_box(template_percent))
                .unwrap();
            let mut buf = String::new();
            t.format(&ctx, &mut buf).unwrap();
            black_box(buf)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_lexer,
    bench_template_compile,
    bench_template_format,
    bench_lexer_throughput,
    bench_e2e,
    bench_custom_delimiters,
);

criterion_main!(benches);
