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
    let mut rng = rand::thread_rng();

    let mut table = MortonTable::new();

    for i in 0..256 {
        let p = Point::new(rng.gen_range(0, 128), rng.gen_range(0, 128));
        table.insert(p, Value(i)).unwrap();
    }

    let mut res = Vec::new();
    table.find_in_range(&Point::new(0, 0), 0xeeee, &mut res);

    assert_eq!(res.len(), 256);
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
    let mut table = MortonTable::default();
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
