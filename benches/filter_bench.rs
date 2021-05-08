use bytes::Bytes;
use criterion::{black_box, criterion_main, Criterion};
use statsd_filter_proxy_rs::filter::{filter, filter_2};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Filter benchmark", |b| {
        b.iter_custom(|iters| {
            let block_list = black_box(vec![
                String::from("otherfoo"),
                String::from("otherfoo2"),
                String::from("otherfoo3"),
                String::from("otherfoo4"),
                String::from("otherfoo5"),
                String::from("otherfoo6"),
                String::from("otherfoo7"),
                String::from("otherfoo8"),
                String::from("otherfoo9"),
                String::from("foo"),
            ]);

            let data = black_box(b"notfoo:1|c\nfoo:2|c\nnotfoo:3|c\nnotfoo:1|c\nfoo:2|c\nnotfoo:3|c\nnotfoo:1|c\nfoo:2|c\nnotfoo:3|c\nnotfoo:1|c\nfoo:2|c\nnotfoo:3|c\nnotfoo:1|c\nfoo:2|c\nnotfoo:3|c\n");

            let now = std::time::Instant::now();

            for _ in 0..iters {
                filter(&block_list, data);
            }

            now.elapsed()
        });
    });

    c.bench_function("Filter benchmark 2", |b| {
        b.iter_custom(|iters| {
            let block_list = black_box(vec![
                String::from("otherfoo"),
                String::from("otherfoo2"),
                String::from("otherfoo3"),
                String::from("otherfoo4"),
                String::from("otherfoo5"),
                String::from("otherfoo6"),
                String::from("otherfoo7"),
                String::from("otherfoo8"),
                String::from("otherfoo9"),
                String::from("foo"),
            ]);

            let block_list = block_list.into_iter().map(Bytes::from).collect::<Vec<_>>();

            let data = black_box(b"notfoo:1|c\nfoo:2|c\nnotfoo:3|c\nnotfoo:1|c\nfoo:2|c\nnotfoo:3|c\nnotfoo:1|c\nfoo:2|c\nnotfoo:3|c\nnotfoo:1|c\nfoo:2|c\nnotfoo:3|c\nnotfoo:1|c\nfoo:2|c\nnotfoo:3|c\n");

            let now = std::time::Instant::now();

            for _ in 0..iters {
                filter_2(&block_list, data);
            }

            now.elapsed()
        });
    });
}

fn filter_bench() {
    let mut c = Criterion::default()
        .measurement_time(std::time::Duration::from_secs(10))
        .sample_size(1000);

    criterion_benchmark(&mut c);
}

criterion_main!(filter_bench);
