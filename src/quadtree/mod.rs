#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

use rayon::prelude::*;
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::mem;

mod morton_key;
#[cfg(test)]
mod tests;

use crate::{Point, Value};
use morton_key::*;
use std::convert::{TryFrom, TryInto};

// at most 15 bits long non-negative integers
// having the 16th bit set might create problems in find_key
const POS_MASK: u32 = 0b0111111111111111;

const SKIP_LEN: usize = 8;
type SkipList = [u32; SKIP_LEN];

#[derive(Debug, Clone)]
pub struct Quadtree {
    skiplist: SkipList,
    skipstep: u32,
    // ---- 9 * 4 bytes so far
    // assuming 64 byte long L1 cache lines we can fit 10 keys
    // keys is 24 bytes in memory
    keys: Vec<MortonKey>,
    positions: Vec<Point>,
    values: Vec<Value>,
}

impl Default for Quadtree {
    fn default() -> Self {
        Self {
            skiplist: [0; SKIP_LEN],
            skipstep: 0,
            keys: Default::default(),
            values: Default::default(),
            positions: Default::default(),
        }
    }
}

impl Quadtree {
    pub fn new() -> Self {
        Self {
            skiplist: Default::default(),
            skipstep: 0,
            keys: vec![],
            values: vec![],
            positions: vec![],
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            skiplist: Default::default(),
            skipstep: 0,
            values: Vec::with_capacity(cap),
            keys: Vec::with_capacity(cap),
            positions: Vec::with_capacity(cap),
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (Point, &'a Value)> + 'a {
        let values = self.values.as_ptr();
        self.positions.iter().enumerate().map(move |(i, id)| {
            let val = unsafe { &*values.add(i) };
            (*id, val)
        })
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.skiplist = [Default::default(); SKIP_LEN];
        self.values.clear();
        self.positions.clear();
    }

    fn rebuild_skip_list(&mut self) {
        #[cfg(debug_assertions)]
        {
            // assert that keys is sorted.
            // at the time of writing is_sorted is still unstable
            if self.keys.len() > 2 {
                let mut it = self.keys.iter();
                let mut current = it.next().unwrap();
                for item in it {
                    assert!(current <= item);
                    current = item;
                }
            }
        }

        let len = self.keys.len();
        let step = len / SKIP_LEN;
        self.skipstep = step as u32;
        if step < 1 {
            if let Some(key) = self.keys.last() {
                self.skiplist[0] = key.0;
            }
            return;
        }
        for (i, k) in (0..len).step_by(step).skip(1).take(SKIP_LEN).enumerate() {
            self.skiplist[i] = self.keys[k].0;
        }
    }

    /// May trigger reordering of items, if applicable prefer `extend` and insert many keys at once.
    pub fn insert(&mut self, id: Point, row: Value) -> Result<(), Point> {
        if !self.intersects(&id) {
            return Err(id);
        }
        let [x, y] = id.0;
        let [x, y] = [x as u16, y as u16];

        let ind = self
            .keys
            .binary_search(&MortonKey::new(x, y))
            .unwrap_or_else(|i| i);
        self.keys.insert(ind, MortonKey::new(x, y));
        self.positions.insert(ind, id);
        self.values.insert(ind, row);
        self.rebuild_skip_list();
        Ok(())
    }

    pub fn from_iterator<It>(it: It) -> Self
    where
        It: Iterator<Item = (Point, Value)>,
    {
        let mut res = Self::default();
        res.extend(it);
        res
    }

    /// Extend the map by the items provided. Panics on invalid items.
    pub fn extend<It>(&mut self, it: It)
    where
        It: Iterator<Item = (Point, Value)>,
    {
        for (id, value) in it {
            assert!(self.intersects(&id));

            let [x, y] = id.0;
            let [x, y] = [x as u16, y as u16];
            let key = MortonKey::new(x, y);
            self.keys.push(key);
            self.positions.push(id);
            self.values.push(value);
        }
        sort(
            self.keys.as_mut_slice(),
            self.positions.as_mut_slice(),
            self.values.as_mut_slice(),
        );
        self.rebuild_skip_list();
    }

    /// Returns the first item with given id, if any
    pub fn get_by_id<'a>(&'a self, id: &Point) -> Option<&'a Value> {
        if !self.intersects(&id) {
            return None;
        }

        self.find_key(id).map(|ind| &self.values[ind]).ok()
    }

    pub fn contains_key(&self, id: &Point) -> bool {
        if !self.intersects(&id) {
            return false;
        }
        self.find_key(id).is_ok()
    }

    /// Find the position of `id` or the position where it needs to be inserted to keep the
    /// container sorted
    fn find_key(&self, id: &Point) -> Result<usize, usize> {
        let [x, y] = id.0;
        let key = MortonKey::new(x as u16, y as u16);

        let step = self.skipstep as usize;
        if step == 0 {
            return self.keys.binary_search(&key);
        }

        let index = if is_x86_feature_detected!("sse2") {
            unsafe { find_key_partition_sse2(&self.skiplist, &key) }
        } else {
            sse_panic()
        };
        let (begin, end) = {
            if index < 8 {
                let begin = index * step;
                let end = self.keys.len().min(begin + step + 1);
                (begin, end)
            } else {
                debug_assert!(self.keys.len() >= step + 3);
                let end = self.keys.len();
                let begin = end - step - 3;
                (begin, end)
            }
        };
        self.keys[begin..end]
            .binary_search(&key)
            .map(|ind| ind + begin)
            .map_err(|ind| ind + begin)
    }

    /// For each id returns the first item with given id, if any
    pub fn get_by_ids<'a>(&'a self, ids: &[Point]) -> Vec<(Point, &'a Value)> {
        ids.par_iter()
            .filter_map(|id| self.get_by_id(id).map(|row| (*id, row)))
            .collect()
    }

    pub fn find_in_range<'a>(
        &'a self,
        center: &Point,
        radius: u32,
        out: &mut Vec<(Point, &'a Value)>,
    ) {
        debug_assert!(
            radius & 0xefff == radius,
            "Radius must fit into 31 bits!; {} != {}",
            radius,
            radius & 0xefff
        );
        let r = i32::try_from(radius).expect("radius to fit into 31 bits");
        let r = (r >> 1) + 1;
        let min = *center + Point::new(-r, -r);
        let max = *center + Point::new(r, r);

        let [min, max] = self.morton_min_max(&min, &max);
        let it = self.positions[min..max]
            .iter()
            .enumerate()
            .filter_map(|(i, id)| {
                if center.dist(&id) < radius {
                    Some((*id, &self.values[i + min]))
                } else {
                    None
                }
            });
        out.extend(it);
    }

    /// Count in AABB
    pub fn count_in_range<'a>(&'a self, center: &Point, radius: u32) -> u32 {
        let r = radius as i32 / 2 + 1;
        let min = *center + Point::new(-r, -r);
        let max = *center + Point::new(r, r);

        let [min, max] = self.morton_min_max(&min, &max);

        self.positions[min..max]
            .iter()
            .filter(move |id| center.dist(&id) < radius)
            .count()
            .try_into()
            .expect("count to fit into 32 bits")
    }

    /// Turn AABB min-max to from-to indices
    /// Clamps `min` and `max` to intersect `self`
    fn morton_min_max(&self, min: &Point, max: &Point) -> [usize; 2] {
        let min: usize = {
            if !self.intersects(&min) {
                0
            } else {
                self.find_key(&min).unwrap_or_else(|i| i)
            }
        };
        let max: usize = {
            let lim = (self.keys.len() as i64 - 1).max(0) as usize;
            if !self.intersects(&max) {
                lim
            } else {
                self.find_key(&max).unwrap_or_else(|i| i)
            }
        };
        [min, max]
    }

    /// Return wether point is within the bounds of this node
    pub fn intersects(&self, point: &Point) -> bool {
        let [x, y] = point.0;
        let [x, y] = [x as u32, y as u32];
        (x & POS_MASK) == x && (y & POS_MASK) == y
    }

    /// Return [min, max) of the bounds of this table
    pub fn bounds(&self) -> (Point, Point) {
        let max = POS_MASK + 1;
        let max = i32::try_from(max).expect("Max to fit into i32");
        (Point::new(0, 0), Point::new(max, max))
    }

    pub fn delete(&mut self, id: &Point) -> Option<Value> {
        if !self.contains_key(id) {
            return None;
        }

        self.find_key(&id)
            .map(|ind| {
                self.keys.remove(ind);
                self.positions.remove(ind);
                self.values.remove(ind)
            })
            .ok()
    }
}

/// Parallel Quicksort implementation to sort the 3 slices representing the Quadtree.
fn sort<Point: Send, Value: Send>(
    keys: &mut [MortonKey],
    positions: &mut [Point],
    values: &mut [Value],
) {
    debug_assert!(
        keys.len() == positions.len(),
        "{} {}",
        keys.len(),
        positions.len()
    );
    debug_assert!(
        keys.len() == values.len(),
        "{} {}",
        keys.len(),
        values.len()
    );
    if keys.len() < 2 {
        return;
    }
    let pivot = sort_partition(keys, positions, values);
    let (klo, khi) = keys.split_at_mut(pivot);
    let (plo, phi) = positions.split_at_mut(pivot);
    let (vlo, vhi) = values.split_at_mut(pivot);
    rayon::join(
        || sort(klo, plo, vlo),
        || sort(&mut khi[1..], &mut phi[1..], &mut vhi[1..]),
    );
}

fn sort_partition<Point: Send, Value: Send>(
    keys: &mut [MortonKey],
    positions: &mut [Point],
    values: &mut [Value],
) -> usize {
    debug_assert!(!keys.is_empty());

    let lim = keys.len() - 1;
    let mut i = 0;
    let pivot = keys[lim];
    for j in 0..lim {
        if keys[j] < pivot {
            keys.swap(i, j);
            positions.swap(i, j);
            values.swap(i, j);
            i += 1;
        }
    }
    keys.swap(i, lim);
    positions.swap(i, lim);
    values.swap(i, lim);
    i
}

/// Find the index of the partition where `key` _might_ reside.
/// This is the index of the second to first item in the `skiplist` that is greater than the `key`
#[inline(always)]
unsafe fn find_key_partition_sse2(skiplist: &[u32; SKIP_LEN], key: &MortonKey) -> usize {
    let key = key.0 as i32;
    let keys4 = _mm_set_epi32(key, key, key, key);

    let [s0, s1, s2, s3, s4, s5, s6, s7]: [i32; SKIP_LEN] = mem::transmute(*skiplist);
    let skiplist_a: __m128i = _mm_set_epi32(s0, s1, s2, s3);
    let skiplist_b: __m128i = _mm_set_epi32(s4, s5, s6, s7);

    // set every 32 bits to 0xFFFF if key < skip else sets it to 0x0000
    let results_a = _mm_cmpgt_epi32(keys4, skiplist_a);
    let results_b = _mm_cmpgt_epi32(keys4, skiplist_b);

    // create a mask from the most significant bit of each 8bit element
    let mask_a = _mm_movemask_epi8(results_a);
    let mask_b = _mm_movemask_epi8(results_b);

    // count the number of bits set to 1
    let index = _popcnt32(mask_a) + _popcnt32(mask_b);
    // because the mask was created from 8 bit wide items every key in skip list is counted
    // 4 times.
    // We know that index is unsigned to we can optimize by using bitshifting instead
    //   of division.
    //   This resulted in a 1ns speedup on my Intel Core i7-8700 CPU.
    let index = index >> 2;
    index as usize
}

#[inline(never)]
fn sse_panic() -> usize {
    println!(
        r#"
AVX: {}
SSE: {}
                "#,
        is_x86_feature_detected!("avx"),
        is_x86_feature_detected!("sse"),
    );
    unimplemented!("find_key is not implemented for the current CPU")
}
