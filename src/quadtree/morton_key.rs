#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord, Default)]
pub struct MortonKey(pub u32);

impl MortonKey {
    pub fn new(x: u16, y: u16) -> Self {
        Self(Self::morton2(x as u32, y as u32))
    }

    fn morton2(x: u32, y: u32) -> u32 {
        Self::partition(x) + (Self::partition(y) << 1)
    }

    fn partition(mut n: u32) -> u32 {
        // n = ----------------fedcba9876543210 : Bits initially
        // n = --------fedcba98--------76543210 : After (1)
        // n = ----fedc----ba98----7654----3210 : After (2)
        // n = --fe--dc--ba--98--76--54--32--10 : After (3)
        // n = -f-e-d-c-b-a-9-8-7-6-5-4-3-2-1-0 : After (4)
        n = (n ^ (n << 8)) & 0x00ff00ff; // (1)
        n = (n ^ (n << 4)) & 0x0f0f0f0f; // (2)
        n = (n ^ (n << 2)) & 0x33333333; // (3)
        (n ^ (n << 1)) & 0x55555555 // (4)
    }

    /// Calculate the original point of this hash key.
    /// In practice it is more beneficial to just store the original key if you need to access it
    /// later.
    #[allow(unused)]
    pub fn as_point(&self) -> [u16; 2] {
        let x = Self::reconstruct(self.0) as u16;
        let y = Self::reconstruct(self.0 >> 1) as u16;
        [x, y]
    }

    fn reconstruct(mut n: u32) -> u32 {
        // -f-e-d-c-b-a-9-8-7-6-5-4-3-2-1-0 : After (1)
        // -ffeeddccbbaa9988776655443322110 : After (2)
        // --fe--dc--ba--98--76--54--32--10 : After (3)
        // --fefedcdcbaba989876765454323210 : After (4)
        // ----fedc----ba98----7654----3210 : After (5)
        // ----fedcfedcba98ba98765476543210 : After (6)
        // --------fedcba98--------76543210 : After (7)
        // --------fedcba98fedcba9876543210 : After (8)
        // ----------------fedcba9876543210 : After (9)
        n &= 0x55555555;
        n |= n >> 1;
        n &= 0x33333333;
        n |= n >> 2;
        n &= 0x0f0f0f0f;
        n |= n >> 4;
        n &= 0x00ff00ff;
        n |= n >> 8;
        n & 0x0000ffff
    }
}
