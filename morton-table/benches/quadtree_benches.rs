use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use morton_table::morton_table::MortonTable;
use morton_table::quadtree::Quadtree;
use morton_table::{Point, Value};
use rand::RngCore;
use rand::{rngs::SmallRng, Rng, SeedableRng};

fn get_rand() -> impl rand::Rng {
    SmallRng::seed_from_u64(0xdeadbeef)
}

fn contains_rand(c: &mut Criterion) {
    let mut group = c.benchmark_group("contains_rand");
    let mut rng = get_rand();
    for size in 8..16 {
        let size = 1 << size;

        let items = (0..size)
            .map(|i| {
                let p = Point::new(rng.gen_range(0, 8000), rng.gen_range(0, 8000));
                (p, Value(i))
            })
            .collect::<Vec<_>>();

        group.bench_with_input(BenchmarkId::new("Morton", size), &size, |b, _| {
            let table = MortonTable::from_iterator(items.iter().cloned());

            b.iter(|| {
                let p = Point::new(rng.gen_range(0, 8000), rng.gen_range(0, 8000));
                table.contains_key(&p)
            })
        });
        group.bench_with_input(BenchmarkId::new("Quadtree", size), &size, |b, _| {
            let table = Quadtree::from_iterator(items.iter().cloned());

            b.iter(|| {
                let p = Point::new(rng.gen_range(0, 8000), rng.gen_range(0, 8000));
                table.contains_key(&p)
            })
        });
    }
    group.finish();
}

fn get_entities_in_range_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_in_range sparse");
    let mut rng = get_rand();
    let radius = 512;
    for size in 8..16 {
        let size = 1 << size;
        let items = (0..size)
            .map(|_| {
                let p = Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2));
                (p, Value(rng.gen()))
            })
            .collect::<Vec<_>>();
        group.bench_with_input(BenchmarkId::new("MortonTable", size), &size, |b, _| {
            let table = MortonTable::from_iterator(items.iter().cloned());

            let mut res = Vec::new();
            b.iter(|| {
                let table = &table;
                let p = Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2));
                table.find_in_range(&p, radius, &mut res);
                black_box(&res);
            });
        });
        group.bench_with_input(BenchmarkId::new("Quadtree", size), &size, |b, _| {
            let table = Quadtree::from_iterator(items.iter().cloned());

            let mut res = Vec::new();
            b.iter(|| {
                let table = &table;
                let p = Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2));
                table.find_in_range(&p, radius, &mut res);
                black_box(&res);
            });
        });
    }
    group.finish();
}

fn get_entities_in_range_dense(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_in_range dense");
    let mut rng = get_rand();
    let radius = 50;
    for size in 8..16 {
        let size = 1 << size;
        let items: Vec<_> = (0..size)
            .map(|_| {
                let p = Point::new(rng.gen_range(0, 200 * 2), rng.gen_range(0, 200 * 2));
                (p, Value(rng.gen()))
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("MortonTable", size), &size, |b, _| {
            let table = MortonTable::from_iterator(items.iter().cloned());

            let mut res = Vec::new();
            b.iter(|| {
                let table = &table;
                let p = Point::new(rng.gen_range(0, 200 * 2), rng.gen_range(0, 200 * 2));
                table.find_in_range(&p, radius, &mut res);
                black_box(&res);
            });
        });
        group.bench_with_input(BenchmarkId::new("Quadtree", size), &size, |b, _| {
            let table = Quadtree::from_iterator(items.iter().cloned());

            let mut res = Vec::new();
            b.iter(|| {
                let table = &table;
                let p = Point::new(rng.gen_range(0, 200 * 2), rng.gen_range(0, 200 * 2));
                table.find_in_range(&p, radius, &mut res);
                black_box(&res);
            });
        });
    }
    group.finish();
}

fn make_table(c: &mut Criterion) {
    let mut group = c.benchmark_group("make_table");
    let mut rng = get_rand();
    for size in 8..16 {
        let size = 1 << size;
        let items: Vec<_> = (0..size)
            .map(|_| {
                (
                    Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2)),
                    Value(rng.next_u32()),
                )
            })
            .collect();
        group.bench_with_input(BenchmarkId::new("MortonTable", size), &size, |b, _| {
            b.iter(|| {
                let table = MortonTable::from_iterator(items.iter().cloned());
                table
            });
        });
        group.bench_with_input(BenchmarkId::new("Quadtree", size), &size, |b, _| {
            b.iter(|| {
                let table = Quadtree::from_iterator(items.iter().cloned());
                table
            });
        });
    }
    group.finish();
}

fn rebuild_table(c: &mut Criterion) {
    let mut group = c.benchmark_group("rebuild_table");
    let mut rng = get_rand();
    for size in 8..16 {
        let size = 1 << size;

        let items: Vec<_> = (0..size)
            .map(|_| {
                (
                    Point::new(rng.gen_range(0, 3900), rng.gen_range(0, 3900)),
                    Value(rng.next_u32()),
                )
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("MortonTable", size), &size, |b, _| {
            let mut table = MortonTable::default();

            b.iter(|| {
                table.clear();
                table.extend(items.iter().cloned());
            });
        });
        group.bench_with_input(BenchmarkId::new("Quadtree", size), &size, |b, _| {
            let mut table = Quadtree::new(Point::new(0, 0), Point::new(3900, 3900));

            b.iter(|| {
                table.clear();
                table.extend(items.iter().cloned());
            });
        });
    }
    group.finish();
}

fn get_by_id_rand(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_by_id random");
    let mut rng = get_rand();
    for size in 8..16 {
        let size = 1 << size;
        let items = (0..size)
            .map(|_| {
                let pos = Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2));
                (pos, Value(rng.next_u32()))
            })
            .collect::<Vec<_>>();

        group.bench_with_input(BenchmarkId::new("MortonTable", size), &size, |b, _| {
            let table = MortonTable::from_iterator(items.iter().cloned());

            b.iter(|| {
                let pos = Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2));
                table.get_by_id(&pos)
            });
        });
        group.bench_with_input(BenchmarkId::new("Quadtree", size), &size, |b, _| {
            let table = Quadtree::from_iterator(items.iter().cloned());

            b.iter(|| {
                let pos = Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2));
                table.get_by_id(&pos)
            });
        });
    }
    group.finish();
}

fn get_by_id_in_table_rand(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_by_id, all queried elements are in the table");
    let mut rng = get_rand();
    for size in 8..16 {
        let size = 1 << size;
        let points: Vec<_> = (0..size)
            .map(|_| {
                let pos = Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2));
                (pos, Value(rng.next_u32()))
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("MortonTable", size), &size, |b, _| {
            let table = MortonTable::from_iterator(points.iter().cloned());

            b.iter(|| {
                let i = rng.gen_range(0, points.len());
                let pos = &points[i].0;
                table.get_by_id(pos)
            });
        });
        group.bench_with_input(BenchmarkId::new("Quadtree", size), &size, |b, _| {
            let table = Quadtree::from_iterator(points.iter().cloned());

            b.iter(|| {
                let i = rng.gen_range(0, points.len());
                let pos = &points[i].0;
                table.get_by_id(pos)
            });
        });
    }
    group.finish();
}

fn random_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_insert");
    let mut rng = get_rand();
    for size in 8..16 {
        let size = 1 << size;

        let items: Vec<_> = (0..size)
            .map(|_| {
                let x = rng.gen_range(0, 3000);
                let y = rng.gen_range(0, 3000);
                (Point::new(x, y), Value(420))
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("MortonTable", size), &size, |b, _| {
            let mut table = MortonTable::from_iterator(items.iter().cloned());

            b.iter(|| {
                let x = rng.gen_range(0, 3000);
                let y = rng.gen_range(0, 3000);
                let p = Point::new(x, y);

                table.insert(p, Value(420)).unwrap()
            });
        });
        group.bench_with_input(BenchmarkId::new("Quadtree", size), &size, |b, _| {
            let mut table = Quadtree::from_iterator(items.iter().cloned());

            b.iter(|| {
                let x = rng.gen_range(0, 3000);
                let y = rng.gen_range(0, 3000);
                let p = Point::new(x, y);

                table.insert(p, Value(420)).unwrap()
            });
        });
    }
    group.finish();
}

criterion_group!(
    quadtree_benches,
    contains_rand,
    get_entities_in_range_sparse,
    get_entities_in_range_dense,
    make_table,
    random_insert,
    rebuild_table,
    get_by_id_in_table_rand,
    get_by_id_rand,
);

criterion_main!(quadtree_benches);
