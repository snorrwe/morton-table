use super::morton_key::MortonKey;

/// Parallel Quicksort implementation to sort the 3 slices representing the Quadtree.
pub fn sort<Point: Send, Value: Send>(
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

/// Assumes that all 3 slices are equal in size.
/// Assumes that the slices are not empty
fn sort_partition<Pos, Row>(
    keys: &mut [MortonKey],
    positions: &mut [Pos],
    values: &mut [Row],
) -> usize {
    debug_assert!(!keys.is_empty());

    macro_rules! swap {
        ($i: expr, $j: expr) => {
            keys.swap($i, $j);
            positions.swap($i, $j);
            values.swap($i, $j);
        };
    };

    let len = keys.len();
    let lim = len - 1;

    let (pivot, pivot_ind) = {
        use std::mem::swap;
        // choose the median of the first, middle and last elements as the pivot

        let mut first = 0;
        let mut last = lim;
        let mut median = len / 2;

        if keys[last] < keys[median] {
            swap(&mut median, &mut last);
        }
        if keys[last] < keys[first] {
            swap(&mut last, &mut first);
        }
        if keys[median] < keys[first] {
            swap(&mut median, &mut first);
        }
        (keys[median], median)
    };

    swap!(pivot_ind, lim);

    let mut i = 0; // index of the last item <= pivot
    for j in 0..lim {
        if keys[j] < pivot {
            swap!(i, j);
            i += 1;
        }
    }
    swap!(i, lim);
    i
}
