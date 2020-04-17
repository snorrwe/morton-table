use super::*;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

#[test]
fn insertions() {
    let mut table = Quadtree::new();

    table.insert(Point::new(16, 32), Value(123)).unwrap();
}

#[test]
fn test_range_query_all() {
    let mut rng = rand::thread_rng();

    let mut table = Quadtree::new();
    let mut values = HashMap::new();

    for i in 0..256 {
        let mut p = Point::new(rng.gen_range(0, 128), rng.gen_range(0, 128));
        while values.contains_key(&p) {
            p = Point::new(rng.gen_range(0, 128), rng.gen_range(0, 128));
        }
        table.insert(p, Value(i)).unwrap();
        values.insert(p, Value(i));
    }

    let mut res = Vec::new();
    // sqrt a^2 + b^2 = 90; where a = 64 and b = 64
    let radius = 90;
    table.find_in_range(&Point::new(64, 64), radius, &mut res);

    assert_eq!(res.len(), 256);
    let res = res
        .into_iter()
        .map(|(k, v)| (k, *v))
        .collect::<HashMap<_, _>>();

    assert_eq!(res.len(), 256, "There were duplicates in the output!");
}

#[test]
fn get_by_id() {
    let mut rng = rand::thread_rng();

    let mut table = Quadtree::new();

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

        let morton = MortonKey::new(x, y);

        let res = morton.as_point();

        assert_eq!([x, y], res);
    }
}

#[test]
fn from_iterator_inserts_correctly() {
    let mut rng = rand::thread_rng();

    let len = 1 << 12;
    let mut points = HashMap::with_capacity(len);
    let mut table = Quadtree::default();
    table.extend((0..len).filter_map(|_| {
        let pos = Point::new(rng.gen_range(0, 3900 * 2), rng.gen_range(0, 3900 * 2));
        if !points.contains_key(&pos) {
            return None;
        }
        let val = rng.next_u32();
        let val = Value(val as i32);
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

    let [litmax, bigmin] = litmax_bigmin(&a, &b);

    assert_eq!(litmax, MortonKey::new(9, 7));
    assert_eq!(bigmin, MortonKey::new(5, 8));
}

#[test]
fn test_litmax_bigmin_x() {
    let a = MortonKey::new(5, 5);
    let b = MortonKey::new(9, 7);

    let [litmax, bigmin] = litmax_bigmin(&a, &b);

    assert_eq!(litmax, MortonKey(63));
    assert_eq!(bigmin, MortonKey(98));
}
