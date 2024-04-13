use arigato::raw::{Dehydrate, FileType, Hydrate, Qid, Stat};
use criterion::{criterion_group, criterion_main, Criterion};
use std::io::{Cursor, Seek, SeekFrom};

pub fn criterion_benchmark(c: &mut Criterion) {
    let stat = Stat::builder("name", Qid::new(FileType::Unknown(3), 4, 5))
        .with_size(1024)
        .with_uid("uid")
        .with_gid("gid")
        .with_muid("muid")
        .with_atime(10)
        .with_mtime(20)
        .with_nuid(500)
        .with_ngid(501)
        .with_nmuid(502)
        .with_extension("something")
        .build();

    let mut group = c.benchmark_group("stat");

    let mut buf = Cursor::new(vec![]);
    group.bench_function("dehydrate", |b| {
        b.iter(|| {
            buf.seek(SeekFrom::Start(0)).unwrap();
            stat.dehydrate(&mut buf).unwrap();
        });
    });

    let mut buf = Cursor::new(vec![]);
    stat.dehydrate(&mut buf).unwrap();
    group.bench_function("hydrate", |b| {
        b.iter(|| {
            buf.seek(SeekFrom::Start(0)).unwrap();
            let _ = Stat::hydrate(&mut buf).unwrap();
        });
    });

    let mut buf = Cursor::new(vec![]);
    group.bench_function("dehydrate-hydrate", |b| {
        b.iter(|| {
            buf.seek(SeekFrom::Start(0)).unwrap();
            stat.dehydrate(&mut buf).unwrap();
            buf.seek(SeekFrom::Start(0)).unwrap();
            let _ = Stat::hydrate(&mut buf).unwrap();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
