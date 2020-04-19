//! Linear Quadtree.
//! # Contracts:
//! - Key axis must be an integer in the interval [0, 2^16)
//!
pub mod quadtree;
pub mod morton_table;

use std::ops::{Add, AddAssign, Deref};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Point(pub [u32; 2]);

impl AddAssign for Point {
    fn add_assign(&mut self, p: Self) {
        self.0[0] += p.0[0];
        self.0[1] += p.0[1];
    }
}

impl Deref for Point {
    type Target = [u32; 2];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add for Point {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self([x, y])
    }

    pub fn dist(&self, rhs: &Self) -> u32 {
        let x = self[0] as i32 - rhs[0] as i32;
        let y = self[1] as i32 - rhs[1] as i32;
        let squared = (x * x + y * y) as f32;
        let res = squared.sqrt();
        res as u32
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Value(pub u32);
