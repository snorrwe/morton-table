use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use morton_table::quadtree::Quadtree;
use morton_table::{Point, Value};
use rand::RngCore;
use rand::{rngs::SmallRng, Rng, SeedableRng};

fn get_rand() -> impl rand::Rng {
    SmallRng::seed_from_u64(0xdeadbeef)
}

fn contains_rand(c: &mut Criterion) {
    let mut group = c.benchmark_group("Quadtree contains_rand");
    for size in 8..16 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, move |b, &size| {
            let mut rng = get_rand();

            let table = <Quadtree>::from_iterator((0..size).map(|i| {
                let p = Point::new(rng.gen_range(0, 8000), rng.gen_range(0, 8000));
                (p, Value(i))
            }));

            b.iter(|| {
                let p = Point::new(rng.gen_range(0, 8000), rng.gen_range(0, 8000));
                table.contains_key(&p)
            })
        });
    }
    group.finish();
}

fn get_entities_in_range_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("Quadtree find_in_range sparse");
    for size in 8..16 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();

            let table = <Quadtree>::from_iterator((0..size).map(|_| {
                let p = Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2));
                (p, Value(rng.gen()))
            }));

            let radius = 512;
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
    let mut group = c.benchmark_group("Quadtree find_in_range dense");
    for size in 8..16 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();

            let table = <Quadtree>::from_iterator((0..size).map(|_| {
                let p = Point::new(rng.gen_range(0, 200 * 2), rng.gen_range(0, 200 * 2));
                (p, Value(rng.gen()))
            }));

            let radius = 50;
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
    let mut group = c.benchmark_group("Quadtree make_table");
    for size in 8..16 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();

            b.iter(|| {
                let table = <Quadtree>::from_iterator((0..size).map(|_| {
                    (
                        Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2)),
                        Value(rng.next_u32() as i32),
                    )
                }));
                table
            });
        });
    }
    group.finish();
}

fn rebuild_table(c: &mut Criterion) {
    let mut group = c.benchmark_group("Quadtree rebuild_table");
    for size in 8..16 {
        let size = 1 << size;

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();

            let mut table = <Quadtree>::default();

            b.iter(|| {
                table.clear();

                table.extend((0..size).map(|_| {
                    (
                        Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2)),
                        Value(rng.next_u32() as i32),
                    )
                }));
            });
        });
    }
    group.finish();
}

fn get_by_id_rand(c: &mut Criterion) {
    let mut group = c.benchmark_group("Quadtree get_by_id random");
    for size in 8..16 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &len| {
            let mut rng = get_rand();

            let table = <Quadtree>::from_iterator((0..len).map(|_| {
                let pos = Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2));
                (pos, Value(rng.next_u32() as i32))
            }));

            b.iter(|| {
                let pos = Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2));
                table.get_by_id(&pos)
            });
        });
    }
    group.finish();
}

fn get_by_id_in_table_rand(c: &mut Criterion) {
    let mut group = c.benchmark_group("Quadtree get_by_id, all queried elements are in the table");
    for size in 8..16 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &len| {
            let mut rng = get_rand();

            let mut points = Vec::with_capacity(len);
            let table = <Quadtree>::from_iterator((0..len).map(|_| {
                let pos = Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2));
                points.push(pos.clone());
                (pos, Value(rng.next_u32() as i32))
            }));

            b.iter(|| {
                let i = rng.gen_range(0, points.len());
                let pos = &points[i];
                table.get_by_id(pos)
            });
        });
    }
    group.finish();
}

fn random_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("Quadtree random_insert");
    for size in 8..16 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();
            let mut table = <Quadtree>::default();

            for _ in 0..size {
                let x = rng.gen_range(0, 29000);
                let y = rng.gen_range(0, 29000);
                let p = Point::new(x, y);

                table.insert(p, Value(420)).unwrap();
            }

            b.iter(|| {
                let x = rng.gen_range(0, 29000);
                let y = rng.gen_range(0, 29000);
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
