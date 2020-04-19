use super::morton_key::MortonKey;

/// Split an AABB. Return its location codes on a Z curve.
/// Taking the points as params will allow the calling code to use cached positions instead of
/// calculating the positions from the hash every time.
///
/// # Contracts:
///
/// ## Inputs:
///
/// - `mortonmin < mortonmax`
/// - `x1, y1` is the position with the hash `mortonmin`
/// - `x2, y2` is the position with the hash `mortonmax`
///
/// Violating the input contracts is a panic in debug mode and undefined behaviour in release mode.
///
/// ## Output:
///
/// The ending and beginning positions of the two new ranges
/// - litmax: The end, or max point of the new AABB (mortonmin, litmax)
/// - bigmin: The end, or max point of the new AABB (bigmin, mortonmax)
/// - `litmax < bigmin`
/// - `mortonmin <= litmax`
/// - `bigmin <= mortonmax`
pub fn litmax_bigmin(
    mortonmin: u32,
    [x1, y1]: [u32; 2],
    mortonmax: u32,
    [x2, y2]: [u32; 2],
) -> [MortonKey; 2] {
    debug_assert!(mortonmin < mortonmax);
    debug_assert!(MortonKey(mortonmin).as_point() == [x1, y1]);
    debug_assert!(MortonKey(mortonmax).as_point() == [x2, y2]);

    // find the most significant bit that's different
    let diff = mortonmin ^ mortonmax;
    let diff_msb = msb_de_bruijn(diff);

    // split among the side with the higher most significant bit
    // even msb will mean the x axis.
    let [litmax, bigmin] = if diff_msb & 1 == 0 {
        let [x1, x2] = impl_litmax_bigmin(x1, x2, diff_msb / 2);
        debug_assert!(x1 < x2);

        [MortonKey::new_u32(x1, y2), MortonKey::new_u32(x2, y1)]
    } else {
        let [m1, y2] = impl_litmax_bigmin(y1, y2, diff_msb / 2);
        let y1 = m1 | y1;
        debug_assert!(y1 < y2);

        [MortonKey::new_u32(x2, y1), MortonKey::new_u32(x1, y2)]
    };

    debug_assert!(litmax.0 < bigmin.0);
    debug_assert!(mortonmin <= litmax.0);
    debug_assert!(bigmin.0 <= mortonmax);
    [litmax, bigmin]
}

/// `diff_msb`: position of the most significant bit that's different between `a` and `b`
fn impl_litmax_bigmin(a: u32, b: u32, diff_msb: u32) -> [u32; 2] {
    debug_assert!(a < b);

    let prefix2 = 1 << diff_msb;
    let prefix1 = prefix2 - 1;

    // calculate the common most significant bits
    // aka. the prefix
    let mask = !(!prefix2 & prefix1);
    let z = (a & b) & mask;
    // append the suffixes
    let litmax = z | prefix1;
    let bigmin = z | prefix2;

    debug_assert!(litmax < bigmin);
    debug_assert!(a <= litmax);
    debug_assert!(bigmin <= b);

    [litmax, bigmin]
}

/// [See](http://supertech.csail.mit.edu/papers/debruijn.pdf)
/// calculates the most significant bit that's set
fn msb_de_bruijn(mut v: u32) -> u32 {
    const DE_BRUIJN_BIT_POS: &[u32] = &[
        0, 9, 1, 10, 13, 21, 2, 29, 11, 14, 16, 18, 22, 25, 3, 30, 8, 12, 20, 28, 15, 17, 24, 7,
        19, 27, 23, 6, 26, 5, 4, 31,
    ];

    // first round down to one less than a power of 2
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;

    // *magic*
    let ind = v as usize * 0x07c4acdd;
    let ind = ind as u32 >> 27;
    return DE_BRUIJN_BIT_POS[ind as usize];
}
