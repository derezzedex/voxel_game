use criterion::{black_box, criterion_group, criterion_main, Criterion};
use game::terrain::chunk::Chunk;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Chunk filler (0)", |b| b.iter(|| Chunk::new(black_box(0))));
    c.bench_function("Chunk filler (usize max)", |b| b.iter(|| Chunk::new(black_box(std::usize::MAX))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
