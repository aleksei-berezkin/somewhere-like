/// Returns (min, max). Panics if the input is empty.
/// 
/// ```
/// use backend::library::minmax::minmax;
/// assert_eq!((1, 1), minmax(&vec![1]));
/// assert_eq!((1, 3), minmax(&vec![1, 2, 3]));
/// assert_eq!((1.0, 3.0), minmax(&vec![3.0, 1.0, 2.0]));
/// ```
/// # Panics
///
/// ```rust,should_panic
/// use backend::library::minmax::minmax;
/// minmax(&Vec::<u8>::new());
/// ```
pub fn minmax<T: PartialOrd + Copy>(values: &[T]) -> (T, T) {
    let mut min = values[0];
    let mut max = values[0];
    for value in values.iter().skip(1) {
        if *value < min {
            min = *value;
        } else if *value > max {
            max = *value;
        }
    }
    (min, max)
}

/// Returns Some((min, max)) if at least one value is present, None otherwise.
/// 
/// ```
/// use backend::library::minmax::minmax_maybe;
/// assert_eq!(None, minmax_maybe::<u8>(&vec![]));
/// assert_eq!(None, minmax_maybe::<u8>(&vec![None, None]));
/// assert_eq!(Some((1, 3)), minmax_maybe(&vec![None, Some(3), None, Some(1)]));
/// ```
pub fn minmax_maybe<T: PartialOrd + Copy>(values: &[Option<T>]) -> Option<(T, T)> {
    let some_values = values.iter()
        .filter_map(|it| *it)
        .collect::<Vec<_>>();
    return if some_values.is_empty() {
        None
    } else {
        Some(minmax(&some_values))
    }
}

/// Reduces 2 minmax tuples into a single minmax tuple.
/// 
/// ```
/// use backend::library::minmax::reduce_minmax;
/// assert_eq!((1, 4), reduce_minmax((1, 3), (2, 4)));
/// assert_eq!((1, 4), reduce_minmax((2, 4), (1, 3)));
/// ```
pub fn reduce_minmax<T: PartialOrd + Copy>(a: (T, T), b: (T, T)) -> (T, T) {
    let min = if a.0 < b.0 { a.0 } else { b.0 };
    let max = if a.1 > b.1 { a.1 } else { b.1 };
    (min, max)
}

/// For 2 None returns None, for one Some returns that Some,
/// for both Some returns Some(reduce_minmax(a, b)).
/// 
/// ```
/// use backend::library::minmax::reduce_minmax_maybe;
/// assert_eq!(None, reduce_minmax_maybe::<u8>(None, None));
/// assert_eq!(Some((2, 4)), reduce_minmax_maybe(None, Some((2, 4))));
/// assert_eq!(Some((1, 3)), reduce_minmax_maybe(Some((1, 3)), None));
/// assert_eq!(Some((1, 4)), reduce_minmax_maybe(Some((2, 4)), Some((1, 3))));
/// ```
pub fn reduce_minmax_maybe<T: PartialOrd + Copy>(a_maybe: Option<(T, T)>, b_maybe: Option<(T, T)>) -> Option<(T, T)> {
    match (a_maybe, b_maybe) {
        (Some(a), Some(b)) => Some(reduce_minmax(a, b)),
        (Some(_), None) => a_maybe,
        _ => b_maybe
    }
}

/// Returns (min, max) relative to `range`, i.e. if `arg` equals `range`, returns (0, 1).
///
/// ```
/// use backend::library::minmax::get_relative_minmax;
/// assert_eq!((0.0, 1.0), get_relative_minmax((1.0, 3.0), (1.0, 3.0)));
/// assert_eq!((0.25, 0.75), get_relative_minmax((1.0, 3.0), (0.0, 4.0)));
/// assert_eq!((-0.25, 1.5), get_relative_minmax((-1.0, 6.0), (0.0, 4.0)));
/// ```
/// 
/// # Panics
/// Panics if `total_minmax` is not a proper range.
/// 
/// ```rust,should_panic
/// use backend::library::minmax::get_relative_minmax;
/// get_relative_minmax((0.0, 0.0), (0.0, 0.0));
/// ```
pub fn get_relative_minmax(arg: (f32, f32), range: (f32, f32)) -> (f32, f32) {
    let span = range.1 - range.0;
    if span <= 0.0 { panic!("Invalid minmax: {:?}", range); }
    let min = (arg.0 - range.0) / span;
    let max = (arg.1 - range.0) / span;
    (min, max)
}

/// Returns the sum of the absolute difference between the 2 minmax tuples.
/// 
/// ```
/// use backend::library::minmax::diff_minmax;
/// assert_eq!(2.0, diff_minmax((1.0, 4.0), (2.0, 3.0)));
/// assert_eq!(2.0, diff_minmax((2.0, 3.0), (1.0, 4.0)));
/// assert_eq!(0.0, diff_minmax((1.0, 3.0), (1.0, 3.0)));
/// ```
pub fn diff_minmax(a: (f32, f32), b: (f32, f32)) -> f32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

/// Returns diff_minmax if both are Some, None otherwise.
/// ```
/// use backend::library::minmax::diff_minmax_maybe;
/// 
/// assert_eq!(Some(2.0), diff_minmax_maybe(Some((1.0, 4.0)), Some((2.0, 3.0))));
/// assert_eq!(None, diff_minmax_maybe(Some((1.0, 4.0)), None));
/// assert_eq!(None, diff_minmax_maybe(None, Some((2.0, 3.0))));
/// assert_eq!(None, diff_minmax_maybe(None, None));
/// ```
pub fn diff_minmax_maybe(a: Option<(f32, f32)>, b: Option<(f32, f32)>) -> Option<f32> {
    match (a, b) {
        (Some(a), Some(b)) => Some(diff_minmax(a, b)),
        _ => None,
    }
}
