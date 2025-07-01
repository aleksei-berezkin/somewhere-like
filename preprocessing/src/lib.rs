/// Returns the index of the value in ascending or descending `values` that is closest to `value`.
/// Always returns an index in `values` bounds.
/// 
/// # Examples
/// 
/// ```
/// use preprocessing::get_closest_index;
/// 
/// assert_eq!(get_closest_index(&[1.0], 0.0), 0);
/// assert_eq!(get_closest_index(&[1.0], 1.0), 0);
/// assert_eq!(get_closest_index(&[1.0], 2.0), 0);
/// 
/// assert_eq!(get_closest_index(&[1.0, 2.0], 0.5), 0);
/// assert_eq!(get_closest_index(&[1.0, 2.0], 1.0), 0);
/// assert_eq!(get_closest_index(&[1.0, 2.0], 1.4), 0);
/// assert_eq!(get_closest_index(&[1.0, 2.0], 1.5), 1);
/// assert_eq!(get_closest_index(&[1.0, 2.0], 2.0), 1);
/// assert_eq!(get_closest_index(&[1.0, 2.0], 2.5), 1);
/// 
/// assert_eq!(get_closest_index(&[1.0, 2.0, 3.0, 4.0, 5.0], 2.5), 2);
/// assert_eq!(get_closest_index(&[1.0, 2.0, 3.0, 4.0, 5.0], 25.0), 4);
/// 
/// assert_eq!(get_closest_index(&[5.0, 4.0, 3.0, 2.0, 1.0], 5.1), 0);
/// assert_eq!(get_closest_index(&[5.0, 4.0, 3.0, 2.0, 1.0], 2.6), 2);
/// assert_eq!(get_closest_index(&[5.0, 4.0, 3.0, 2.0, 1.0], 2.4), 3);
/// assert_eq!(get_closest_index(&[5.0, 4.0, 3.0, 2.0, 1.0], 0.9), 4);
/// ```
/// 
/// # Panics
/// Empty `values` is not allowed.
/// 
/// ```rust,should_panic
/// preprocessing::get_closest_index(&[], 0.0);
/// ```

pub fn get_closest_index(values: &[f64], value: f64) -> usize {
    if values.is_empty() {
        panic!("Values is empty");
    }

    let ascending = values[values.len() - 1] > values[0];

    let mut low = 0;
    let mut bound = values.len();
    while low < bound {
        let mid = (low + bound) / 2;
        if values[mid] == value {
            return mid
        } else if ascending && values[mid] < value
                || !ascending && values[mid] > value {
            low = mid + 1;
        } else {
            bound = mid;
        }
    }

    if low == 0 {
        return 0
    }
    if low >= values.len() {
        return values.len() - 1
    }

    let left_value = values[low - 1];
    let right_value = values[low];
    if (value - left_value).abs() < (value - right_value).abs() {
        low - 1
    } else {
        low
    }
}

/// First yields `(x_center, y_center)`, then yields all points of increasing concentric squares.
/// The following shows the example for `r_max = 2`:
/// 
/// ```text
/// .......
/// .22222.
/// .21112.
/// .21012.
/// .21112.
/// .22222.
/// .......
/// ```
/// Assuming (0, 0) is top-left, the order of iteration is:
/// 
/// * top, including corners
/// * bottom, including corners
/// * left, not including corners
/// * right, not including corners
/// 
/// # Examples
/// 
/// ```rust
/// use preprocessing::iterate_increasing_squares;
/// 
/// assert_eq!(
///     iterate_increasing_squares(2, 2, 0, 5, 5).collect::<Vec<_>>(),
///     [(2, 2)]
/// );
/// assert_eq!(
///     iterate_increasing_squares(2, 2, 1, 5, 5).collect::<Vec<_>>(),
///     [(2, 2), /* top */ (1, 1), (2, 1), (3, 1), /* bottom */ (1, 3), (2, 3), (3, 3), /* left */ (1, 2), /* right */ (3, 2)]
/// );
/// assert_eq!(
///     iterate_increasing_squares(0, 0, 5, 2, 2).collect::<Vec<_>>(),
///     [(0, 0), /* top oob */ /* bottom */ (0, 1), (1, 1), /* left oob */  /* right */ (1, 0)]
/// );
/// assert_eq!(
///     iterate_increasing_squares(5, 5, 4, 2, 3).collect::<Vec<_>>(),
///     [/* top */ (1, 1), /* left */ (1, 2)]
/// );
/// ```
pub fn iterate_increasing_squares(x_center: usize, y_center: usize, r_max: usize, x_bound: usize, y_bound: usize) -> impl Iterator<Item = (usize, usize)> {
    let increasing_square_itr = (1..=r_max).flat_map(move |r| {
        let x_left = x_center as isize - r as isize;
        let x_right = x_center as isize + r as isize;
        let y_top = y_center as isize - r as isize;
        let y_bottom = y_center as isize + r as isize;

        let top_itr = (x_left..=x_right).map(move |x| (x, y_top));
        let bottom_itr = (x_left..=x_right).map(move |x| (x, y_bottom));
        let left_itr = (y_top + 1..y_bottom).map(move |y| (x_left, y));
        let right_itr = (y_top + 1..y_bottom).map(move |y| (x_right, y));

        top_itr.chain(bottom_itr).chain(left_itr).chain(right_itr)
    });

    [(x_center as isize, y_center as isize)].into_iter().chain(increasing_square_itr)
        .filter_map(move |(x, y)|
            if 0 <= x && x < x_bound as isize && 0 <= y && y < y_bound as isize {
                Some((x as usize, y as usize))
            } else {
                None
            }
        )
}
