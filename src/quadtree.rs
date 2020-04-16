use crate::{Point, Value};
use arrayvec::ArrayVec;
use std::mem::size_of;

const CACHELINE_SIZE: usize = 64; // bytes

// CACHELINE_SIZE - size_of(from) - size_of(to) - size_of(children) - 1 (account for bookkeeping bytes in ArrayVec)
const LEN_CHILDREN: usize =
    (CACHELINE_SIZE - size_of::<Point>() * 2 - size_of::<Option<Box<i32>>>() - 1)
        / size_of::<(Point, Value)>();
type Children = Option<Box<[Quadtree; 4]>>;

#[derive(Debug, Clone)]
pub struct Quadtree {
    // bounds as an AABB
    from: Point,
    to: Point,

    children: Children,

    items: ArrayVec<[(Point, Value); LEN_CHILDREN]>,
}

impl Default for Quadtree {
    fn default() -> Self {
        Self::new(Point::new(-0xeeee, -0xeeee), Point::new(0xeeee, 0xeeee))
    }
}

impl Quadtree {
    pub fn new(from: Point, to: Point) -> Self {
        assert!(from[0] <= to[0]);
        assert!(from[1] <= to[1]);
        Self {
            from,
            to,
            children: None,
            items: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        if let Some(children) = self.children.as_mut() {
            for child in children.iter_mut() {
                child.clear();
            }
        }
    }
    pub fn from_iterator<It>(it: It) -> Self
    where
        It: Iterator<Item = (Point, Value)>,
    {
        let mut min = [0xeeee, 0xeeee];
        let mut max = [-0xeeee, -0xeeee];
        let values = it
            .map(|(p, v)| {
                if p[0] < min[0] {
                    min[0] = p[0];
                }
                if p[1] < min[1] {
                    min[1] = p[1];
                }
                if p[0] > max[0] {
                    max[0] = p[0];
                }
                if p[1] > max[1] {
                    max[1] = p[1];
                }
                (p, v)
            })
            .collect::<Vec<_>>();
        // calculate the minimum bounding box to speed up queries by having a more balanced tree
        let mut tree = Self::new(Point(min), Point(max));
        tree.extend(values.into_iter());
        tree
    }

    pub fn extend<It>(&mut self, it: It)
    where
        It: Iterator<Item = (Point, Value)>,
    {
        for (p, v) in it {
            self.insert(p, v).unwrap();
        }
    }

    /// Return if the insertion was successful
    pub fn insert(&mut self, point: Point, value: Value) -> Result<(), Point> {
        if !self.intersects(&point) {
            return Err(point);
        }

        if let Ok(_) = self.items.try_push((point, value)) {
            return Ok(());
        }

        if self.children.is_none() {
            self.split();
        }

        for c in self.children.as_mut().unwrap().iter_mut() {
            if let Ok(()) = c.insert(point, value) {
                return Ok(());
            }
        }

        unreachable!("All insertions failed");
    }

    pub fn intersects(&self, point: &Point) -> bool {
        let [x, y] = **point;

        self.from[0] <= x && self.from[1] <= y && x <= self.to[0] && y <= self.to[1]
    }

    fn split(&mut self) {
        assert!(self.children.is_none());

        let [fromx, fromy] = *self.from;
        let [tox, toy] = *self.to;

        let radius_x = (tox - fromx) / 2;
        let radius_y = (toy - fromy) / 2;

        self.children = Some(Box::new([
            Self::new(
                Point::new(fromx + radius_x, fromy),
                Point::new(tox, fromy + radius_y),
            ),
            Self::new(
                Point::new(fromx + radius_x, fromy + radius_y),
                Point::new(tox, toy),
            ),
            Self::new(
                Point::new(fromx, fromy + radius_y),
                Point::new(fromx + radius_x, toy),
            ),
            Self::new(
                Point::new(fromx, fromy),
                Point::new(fromx + radius_x, fromy + radius_y),
            ),
        ]));
    }

    pub fn intersects_aabb(&self, from: &Point, to: &Point) -> bool {
        if self.to[0] < from[0] || self.from[0] > to[0] {
            return false;
        }
        if self.to[1] < from[1] || self.from[1] > to[1] {
            return false;
        }
        true
    }

    pub fn find_in_range<'a>(
        &'a self,
        center: &Point,
        radius: u32,
        out: &mut Vec<&'a (Point, Value)>,
    ) {
        let r = radius as i32;
        let aabb = [
            Point::new(center[0] - r, center[1] - r),
            Point::new(center[0] + r, center[1] + r),
        ];

        if !self.intersects_aabb(&aabb[0], &aabb[1]) {
            return;
        }

        for p in self.items.iter() {
            if p.0.dist(center) <= radius {
                out.push(p);
            }
        }

        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.find_in_range(center, radius, out);
            }
        }
    }

    pub fn get_by_id<'a>(&'a self, id: &Point) -> Option<&'a Value> {
        if !self.intersects(id) {
            return None;
        }

        for p in self.items.iter() {
            if p.0 == *id {
                return Some(&p.1);
            }
        }

        if let Some(ref children) = self.children {
            for child in children.iter() {
                if let Some(v) = child.get_by_id(id) {
                    return Some(v);
                }
            }
        }
        None
    }

    pub fn contains_key(&self, id: &Point) -> bool {
        if !self.intersects(id) {
            return false;
        }

        for p in self.items.iter() {
            if p.0 == *id {
                return true;
            }
        }

        if let Some(ref children) = self.children {
            for child in children.iter() {
                if child.contains_key(id) {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;
    use std::collections::HashSet;

    #[test]
    fn test_range_query_all() {
        let mut rng = rand::thread_rng();

        let mut table = Quadtree::new(Point::new(0, 0), Point::new(128, 128));

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

        let mut table = Quadtree::new(Point::new(0, 0), Point::new(128, 128));

        let mut points = HashSet::with_capacity(64);

        for _ in 0..64 {
            let p = Point::new(rng.gen_range(0, 128), rng.gen_range(0, 128));
            let [x, y] = p.0;
            let i = 1000 * x + y;
            points.insert((p, Value(i)));
        }

        for (p, e) in points.iter() {
            table.insert(*p, *e).unwrap();
        }

        for p in points {
            let found = table.get_by_id(&p.0);
            assert_eq!(found, Some(&p.1),);
        }
    }
}
