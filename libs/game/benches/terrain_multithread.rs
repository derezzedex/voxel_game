use game::terrain::manager::LOAD_DISTANCE;
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use game::terrain::manager::TerrainManager;
use game::registry::Registry;
use std::sync::Arc;

fn criterion_benchmark(c: &mut Criterion) {
    let mut registry = Registry::new();
    registry.setup();
    let registry = Arc::new(registry);

    // let mut group = c.benchmark_group("Chunk generation (distance: 4, multithreaded)");
    // group.sample_size(10);
    //
    // for threads in 1..=num_cpus::get()+1{
    //     let mut terrain_manager = TerrainManager::new(&registry, threads);
    //     group.bench_with_input(BenchmarkId::new("threads", threads), &threads, |b, _| {
    //         b.iter(|| {
    //             terrain_manager.setup_threaded();
    //             while terrain_manager.get_chunks().len() < (LOAD_DISTANCE as usize * 2 + 1).pow(3) { }
    //         });
    //     });
    // }
    // group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
