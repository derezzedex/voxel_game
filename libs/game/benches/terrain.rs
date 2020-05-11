use game::terrain::manager::LOAD_DISTANCE;
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use game::terrain::manager::TerrainManager;
use game::registry::Registry;
use std::sync::Arc;
use cgmath::Vector3;

#[derive(Debug, Copy, Clone)]
struct Position{
    x: f32,
    y: f32,
    z: f32
}

impl Position{
    pub fn new(x: f32, y: f32, z: f32) -> Self{
        Self{
            x,
            y,
            z
        }
    }
}

impl std::fmt::Display for Position{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut registry = Registry::new();
    registry.setup();
    let registry = Arc::new(registry);

    let mut terrain_manager = TerrainManager::new(&registry, 1);
    c.bench_function("Chunk generation (distance: 4, not multithreaded)", |b| b.iter(|| terrain_manager.setup()));

    let mut group = c.benchmark_group("Chunk generation (distance: 4, multithreaded)");
    group.sample_size(10);

    let position = Position::new(0., 0., 0.);
    group.bench_with_input(BenchmarkId::new("block read", position), &position, |b, p| {
        b.iter(|| {
            terrain_manager.block_at(p.x, p.y, p.z)
        });
    });

    let position = Position::new(0., 0., 0.);
    group.bench_with_input(BenchmarkId::new("block write", position), &position, |b, p| {
        b.iter(|| {
            terrain_manager.set_block_no_dirty(p.x, p.y, p.z, 0)
        });
    });

    group.finish();
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
