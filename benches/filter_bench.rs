use criterion::{black_box, criterion_main, Criterion};
use statsd_filter_proxy_rs::filter::filter;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Filter benchmark", |b| {
        b.iter_custom(|iters| {
            let block_list = black_box(vec![
                String::from("foo"),
                String::from("otherfoo"),
                String::from("otherfoo2"),
                String::from("otherfoo3"),
                String::from("otherfoo4"),
                String::from("otherfoo5"),
                String::from("otherfoo6"),
                String::from("otherfoo7"),
                String::from("otherfoo8"),
                String::from("otherfoo9"),
            ]);

            let data = black_box(b"notfoo:1|c\nfoo:2|c\nnotfoo:3|c");

            let now = std::time::Instant::now();

            for _ in 0..iters {
                filter(&block_list, data);
            }

            now.elapsed()
        });
    });
}

fn filter_bench() {
    let mut c = Criterion::default()
        .measurement_time(std::time::Duration::from_secs(60))
        .sample_size(1000);

    criterion_benchmark(&mut c);
}

criterion_main!(filter_bench);
