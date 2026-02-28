use criterion::{Criterion, criterion_group, criterion_main};
use reloaded3_localisation::locale_api::parser::parse_r3locale_bytes;

fn criterion_benchmark(c: &mut Criterion) {
    let original = include_bytes!("../../src/example.r3l");

    c.bench_function("Reloaded 3 Locale File Parser", |b| {
        let mut buffer = original.to_vec();

        b.iter(|| {
            buffer.copy_from_slice(original);
            parse_r3locale_bytes(&mut buffer).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
