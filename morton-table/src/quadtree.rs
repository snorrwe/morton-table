use crate::{Point, Value};
use arrayvec::ArrayVec;

const LEN_CHILDREN: usize = 16;

type Children = Box<[Quadtree; 4]>;

#[derive(Debug, Clone)]
pub enum Body {
    Children(Children),
    Items(Box<ArrayVec<[(Point, Value); LEN_CHILDREN]>>),
}

#[derive(Debug, Clone)]
pub struct Quadtree {
    // bounds as an AABB
    from: Point,
    to: Point,

    // public so I can flush the cache in benchmarks
    pub body: Body,
}

impl Default for Quadtree {
    fn default() -> Self {
        Self::new(Point::new(0, 0), Point::new(0xffff, 0xffff))
    }
}

impl Quadtree {
    pub fn new(from: Point, to: Point) -> Self {
        assert!(from[0] <= to[0]);
        assert!(from[1] <= to[1]);
        Self {
            from,
            to,
            body: Body::Items(Box::new(Default::default())),
        }
    }

    pub fn clear(&mut self) {
        match &mut self.body {
            Body::Items(items) => items.clear(),
            Body::Children(children) => {
                for child in children.iter_mut() {
                    child.clear();
                }
            }
        }
    }
    pub fn from_iterator<It>(it: It) -> Self
    where
        It: Iterator<Item = (Point, Value)>,
    {
        // calculate the minimum bounding box to speed up queries by having a more balanced tree
        let mut min = [0xeeee, 0xeeee];
        let mut max = [0, 0];
        let values = it
            .map(|(p, v)| {
                min[0] = min[0].min(p[0]);
                min[1] = min[1].min(p[1]);
                max[0] = max[0].max(p[0]);
                max[1] = max[1].max(p[1]);
                (p, v)
            })
            .collect::<Vec<_>>();
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

    /// Returns `Err` if the insertion failed.
    pub fn insert(&mut self, point: Point, value: Value) -> Result<(), Point> {
        if !self.intersects(&point) {
            // point is out of bounds
            return Err(point);
        }

        match &mut self.body {
            Body::Items(items) => {
                if let Ok(_) = items.try_push((point, value)) {
                    // there was capacity left in this node. We're done.
                    return Ok(());
                }
                self.split();
                return self.insert(point, value);
            }
            Body::Children(children) => {
                for c in children.iter_mut() {
                    if let Ok(()) = c.insert(point, value) {
                        // Return when we found a child that can accept this node.
                        return Ok(());
                    }
                }

                // Executing this code would mean that the bounds of this node contain the point
                // , but no child node accepted this point.
                // This would indicate be a programming error in the tree implementation!
                unreachable!("All insertions failed");
            }
        }
    }

    pub fn intersects(&self, point: &Point) -> bool {
        let [x, y] = **point;

        self.from[0] <= x && self.from[1] <= y && x <= self.to[0] && y <= self.to[1]
    }

    pub fn intersects_aabb(&self, from: &Point, to: &Point) -> bool {
        // separating axis test
        if self.to[0] < from[0] || self.from[0] > to[0] {
            return false;
        }
        if self.to[1] < from[1] || self.from[1] > to[1] {
            return false;
        }
        true
    }

    fn split(&mut self) {
        if let Body::Children(_) = self.body {
            panic!("Trying to split a node that's already split");
        }

        let [fromx, fromy] = *self.from;
        let [tox, toy] = *self.to;

        let radius_x = (tox - fromx) / 2;
        let radius_y = (toy - fromy) / 2;

        // split each axis of the bounds in half.
        // | child3 | child0 |
        // | ------ | ------ |
        // | child2 | child1 |

        let children = Box::new([
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
        ]);
        let mut body = Body::Children(children);
        std::mem::swap(&mut body, &mut self.body);
        if let Body::Items(items) = body {
            for (p, v) in items.into_iter() {
                self.insert(p, v).unwrap();
            }
        } else {
            unreachable!()
        }
    }

    pub fn find_in_range<'a>(
        &'a self,
        center: &Point,
        radius: u32,
        out: &mut Vec<&'a (Point, Value)>,
    ) {
        // calculat ethe bounding box of the circle
        let aabb = [
            Point::new(
                center[0].checked_sub(radius).unwrap_or(0),
                center[1].checked_sub(radius).unwrap_or(0),
            ),
            Point::new(
                center[0].checked_add(radius).unwrap_or(0xffff),
                center[1].checked_add(radius).unwrap_or(0xffff),
            ),
        ];

        self.find_in_range_impl(center, radius, &aabb, out);
    }

    fn find_in_range_impl<'a>(
        &'a self,
        center: &Point,
        radius: u32,
        aabb: &[Point; 2],
        out: &mut Vec<&'a (Point, Value)>,
    ) {
        if !self.intersects_aabb(&aabb[0], &aabb[1]) {
            // if the node does not contain the aabb, then it can't intersect this circle either
            return;
        }

        match &self.body {
            Body::Items(items) => {
                // insert all items that are within the circle
                for p in items.iter() {
                    if p.0.dist(center) <= radius {
                        out.push(p);
                    }
                }
            }
            Body::Children(children) => {
                // if the node has children then repeat the procedure for all children
                for child in children.iter() {
                    child.find_in_range_impl(center, radius, aabb, out);
                }
            }
        }
    }

    pub fn get_by_id<'a>(&'a self, point: &Point) -> Option<&'a Value> {
        if !self.intersects(point) {
            return None;
        }

        match &self.body {
            Body::Items(items) => {
                for p in items.iter() {
                    if p.0 == *point {
                        return Some(&p.1);
                    }
                }
            }
            Body::Children(children) => {
                for child in children.iter() {
                    if let Some(v) = child.get_by_id(point) {
                        return Some(v);
                    }
                }
            }
        }
        None
    }

    pub fn contains_key(&self, point: &Point) -> bool {
        if !self.intersects(point) {
            return false;
        }
        match &self.body {
            Body::Items(items) => {
                // if this node contains this point then we're done
                for p in items.iter() {
                    if p.0 == *point {
                        return true;
                    }
                }
            }
            Body::Children(children) => {
                // this node did not contain the key
                // check the children, if any
                for child in children.iter() {
                    if child.contains_key(point) {
                        return true;
                    }
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
