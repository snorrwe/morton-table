use super::*;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

#[test]
fn insertions() {
    let mut table = MortonTable::new();

    table.insert(Point::new(16, 32), Value(123)).unwrap();
}

#[test]
fn test_range_query_all() {
    for _ in 0..16 {
        test_range_query_all_impl();
    }
}

fn test_range_query_all_impl() {
    let mut rng = rand::thread_rng();

    let mut table = MortonTable::new();
    let mut positions = HashSet::new();

    for i in 0..256 {
        let mut p = Point::new(rng.gen_range(0, 128), rng.gen_range(0, 128));
        while positions.contains(&p) {
            p = Point::new(rng.gen_range(0, 128), rng.gen_range(0, 128));
        }
        table.insert(p, Value(i)).unwrap();
        positions.insert(p);
    }

    assert_eq!(positions.len(), 256, "the test input is faulty");

    let mut res = Vec::new();
    // sqrt a^2 + b^2 = 90; where a = 64 and b = 64
    let radius = 91;
    table.find_in_range(&Point::new(64, 64), radius, &mut res);

    let res_positions = res.iter().map(|(k, _)| *k).collect::<HashSet<_>>();

    assert_eq!(
        res_positions,
        positions,
        "\nDifference: {:?}",
        positions.symmetric_difference(&res_positions)
    );
}

#[test]
fn test_range_query_partial_1() {
    let mut table = MortonTable::new();

    for (i, p) in [
        Point::new(8, 6),
        Point::new(9, 10),
        Point::new(11, 8),
        Point::new(6, 8),
        // lets put some outside the query range
        Point::new(16, 8),
        Point::new(12, 11),
        Point::new(0, 0),
        Point::new(15, 20),
    ]
    .iter()
    .enumerate()
    {
        table.insert(*p, Value(i as u32)).unwrap();
    }

    let mut res = Vec::new();
    table.find_in_range(&Point::new(8, 8), 4, &mut res);

    let res_positions = res.iter().map(|(k, _)| *k).collect::<HashSet<_>>();

    assert_eq!(
        res.len(),
        res_positions.len(),
        "there were duplicated in the output"
    );

    assert_eq!(res.len(), 4);
}

#[test]
fn get_by_id() {
    let mut rng = rand::thread_rng();

    let mut table = MortonTable::new();

    let mut points = HashSet::with_capacity(64);

    for _ in 0..64 {
        let p = Point::new(rng.gen_range(0, 128), rng.gen_range(0, 128));
        let [x, y] = p.0;
        let i = 1000 * x + y;
        points.insert((p, Value(i)));
    }

    for (p, e) in points.iter() {
        table.insert(p.clone(), *e).unwrap();
    }

    println!("{:?}\n{:?}", table.skiplist, table.keys);

    for p in points {
        let found = table.get_by_id(&p.0);
        let point = p.0;
        let [x, y] = point.0;
        let key = MortonKey::new(x as u16, y as u16);
        assert_eq!(found, Some(&p.1), "{:?} {:?}", p.0, key);
    }
}

#[test]
fn morton_key_reconstruction_rand() {
    let mut rng = rand::thread_rng();

    for _ in 0..(1 << 12) {
        let x = rng.gen_range(0, 2000);
        let y = rng.gen_range(0, 2000);

        let morton = MortonKey::new_u32(x, y);

        let res = morton.as_point();

        assert_eq!([x, y], res);
    }
}

#[test]
fn from_iterator_inserts_correctly() {
    let mut rng = rand::thread_rng();

    let len = 1 << 12;
    let mut points = HashMap::with_capacity(len);
    let mut table = MortonTable::default();
    table.extend((0..len).filter_map(|_| {
        let pos = Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2));
        if !points.contains_key(&pos) {
            return None;
        }
        let val = rng.next_u32();
        let val = Value(val);
        points.insert(pos.clone(), val);
        Some((pos, val))
    }));
    for (pos, val) in points {
        let v = *table.get_by_id(&pos).expect("to find the value");
        assert_eq!(val, v);
    }
}

#[test]
fn test_litmax_bigmin_y() {
    let a = MortonKey::new(5, 5);
    let b = MortonKey::new(9, 8);

    let [litmax, bigmin] = litmax_bigmin(a.0, a.as_point(), b.0, b.as_point());

    assert_eq!(litmax, MortonKey::new(9, 7));
    assert_eq!(bigmin, MortonKey::new(5, 8));
}

#[test]
fn test_litmax_bigmin_x() {
    let a = MortonKey::new(5, 5);
    let b = MortonKey::new(9, 7);

    let [litmax, bigmin] = litmax_bigmin(a.0, a.as_point(), b.0, b.as_point());

    assert_eq!(litmax, MortonKey(63));
    assert_eq!(bigmin, MortonKey(98));
}
