use blackscholes::OptionInputs;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let inputs = OptionInputs::new(true, 51.03, 55.0, 0.0, 0.0, 25.0 / 360.0);

    c.bench_function("func", |b| {
        b.iter(|| black_box(inputs.calculate_option_price(0.5)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
