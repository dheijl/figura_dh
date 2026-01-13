use criterion::{Criterion, criterion_group, criterion_main};
use figura::{Context, DefaultParser, Template, Value};
use std::borrow::Cow;
use std::collections::HashMap;
use std::hint::black_box;

fn bench_repeat(c: &mut Criterion) {
    let template = Template::<'{', '}'>::compile::<DefaultParser>("{pattern:count}").unwrap();

    let mut ctx: Context = HashMap::new();
    ctx.insert("pattern", Value::Str(Cow::Borrowed("*")));
    ctx.insert("count", Value::Int(50));

    c.bench_function("repeat_directive_50", |b| {
        b.iter(|| {
            let mut buf = String::new();
            template.format(black_box(&ctx), &mut buf).unwrap();
            black_box(buf)
        });
    });
}

criterion_group!(benches, bench_repeat);
criterion_main!(benches);
