use bytes::{Bytes, BytesMut};
use criterion::{black_box, criterion_main, Criterion};
use statsd_filter_proxy_rs::filtered_codec::FilteredCodec;
use tokio_util::codec::Decoder;

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

            let mut filter = FilteredCodec {
                block_list: block_list.into_iter().map(Bytes::from).collect(),
            };

            let data = black_box(b"notfoo:1|c\nfoo:2|c\nnotfoo:3|c\n");

            let mut duration = std::time::Duration::from_secs(0);

            for _ in 0..iters {
                let mut src = BytesMut::from(&data[..]);
                let now = std::time::Instant::now();

                let _ = filter.decode(&mut src);

                duration += now.elapsed();
            }

            duration
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
