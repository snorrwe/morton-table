#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::mem;

pub mod litmax_bigmin;
pub mod morton_key;
pub mod sorting;
#[cfg(test)]
mod tests;

use crate::{Point, Value};
use litmax_bigmin::litmax_bigmin;
use morton_key::*;
use sorting::sort;
use std::convert::TryFrom;

// at most 15 bits long non-negative integers
// having the 16th bit set might create problems in find_key
const POS_MASK: u32 = 0b0111111111111111;

const SKIP_LEN: usize = 8;
type SkipList = [u32; SKIP_LEN];

#[derive(Debug, Clone, Default)]
pub struct MortonTable {
    skipstep: u32,
    skiplist: SkipList,
    // ---- 9 * 4 bytes so far
    // `keys` is 24 bytes in memory
    // I'll make these public to be able to flush them from the cache in benchmarks
    // However in practice you'll want to make these private as there are contracts that when
    // broken will lead to UB!!
    pub keys: Vec<MortonKey>,
    pub positions: Vec<Point>,
    pub values: Vec<Value>,
}

impl MortonTable {
    pub fn new() -> Self {
        Self {
            skiplist: Default::default(),
            skipstep: 0,
            keys: vec![],
            values: vec![],
            positions: vec![],
        }
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
        let step = len / (SKIP_LEN - 1);
        self.skipstep = step as u32;
        if step == 0 {
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
        let [x, y] = [x as u32, y as u32];

        let ind = self
            .keys
            .binary_search(&MortonKey::new_u32(x, y))
            .unwrap_or_else(|i| i);
        self.keys.insert(ind, MortonKey::new_u32(x, y));
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

        self.find_key_morton(&key)
    }

    /// Find the position of `key` or the position where it needs to be inserted to keep the
    /// container sorted
    fn find_key_morton(&self, key: &MortonKey) -> Result<usize, usize> {
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

        let [x, y] = **center;
        let [x, y] = [x as i32, y as i32];
        let min = MortonKey::new((x - r).max(0) as u16, (y - r).max(0) as u16);
        let max = MortonKey::new((x + r) as u16, (y + r) as u16);

        self.find_in_range_impl(center, radius, min, max, out);
    }

    fn find_in_range_impl<'a>(
        &'a self,
        center: &Point,
        radius: u32,
        min: MortonKey,
        max: MortonKey,
        out: &mut Vec<(Point, &'a Value)>,
    ) {
        let (imin, pmin) = self
            .find_key_morton(&min)
            .map(|i| (i, *self.positions[i]))
            .unwrap_or_else(|i| (i, min.as_point()));
        // start at the imin parameter
        // this is used to skip already visited nodes when recursing

        let (imax, pmax) = self
            .find_key_morton(&max)
            // add 1 to include this node in the range query as otherwise an element might be
            // missed
            .map(|i| (i + 1, *self.positions[i]))
            .unwrap_or_else(|i| (i, max.as_point()));

        if imax < imin {
            return;
        }

        // The original paper counts the garbage items and splits above a threshold.
        // Instead let's speculate if we need a split or if it more beneficial to just scan the
        // range
        // The number I picked is more or less arbitrary, it is a power of two and I ran the basic
        // benchmarks to probe a few numbers.
        if imax - imin > 16 {
            let [litmax, bigmin] = litmax_bigmin(min.0, pmin, max.0, pmax);
            // split and recurse
            self.find_in_range_impl(center, radius, min, litmax, out);
            self.find_in_range_impl(center, radius, bigmin, max, out);
            return;
        }

        for (i, id) in self.positions[imin..imax].iter().enumerate() {
            if center.dist(&id) < radius {
                out.push((*id, &self.values[i + imin]));
            }
        }
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
    let results_a: __m128i = _mm_cmpgt_epi32(keys4, skiplist_a);
    let results_b: __m128i = _mm_cmpgt_epi32(keys4, skiplist_b);

    // create a mask from the most significant bit of each 8bit element
    let mask_a: i32 = _mm_movemask_epi8(results_a);
    let mask_b: i32 = _mm_movemask_epi8(results_b);

    // count the number of bits set to 1
    let index: i32 = _popcnt32(mask_a) + _popcnt32(mask_b);
    // because the mask was created from 8 bit wide items every key in skip list is counted
    // 4 times.
    // We know that index is unsigned to we can optimize by using bitshifting instead
    //   of division.
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
