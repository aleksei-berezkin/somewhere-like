/// Returns (min, max). Panics if the input is empty.
/// 
/// ```
/// use backend::library::minmax::minmax;
/// assert_eq!(minmax(&vec![1]), (1, 1));
/// assert_eq!(minmax(&vec![1, 2, 3]), (1, 3));
/// assert_eq!(minmax(&vec![3.0, 1.0, 2.0]), (1.0, 3.0));
/// ```
/// # Panics
///
/// ```rust,should_panic
/// use backend::library::minmax::minmax;
/// minmax(&Vec::<u8>::new());
/// ```
pub fn minmax<T: PartialOrd + Copy>(values: &Vec<T>) -> (T, T) {
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

/// Reduces 2 minmax tuples into a single minmax tuple.
/// 
/// ```
/// use backend::library::minmax::reduce_minmax;
/// assert_eq!(reduce_minmax((1, 3), (2, 4)), (1, 4));
/// assert_eq!(reduce_minmax((2, 4), (1, 3)), (1, 4));
/// ```
pub fn reduce_minmax<T: PartialOrd + Copy>(a: (T, T), b: (T, T)) -> (T, T) {
    let min = if a.0 < b.0 { a.0 } else { b.0 };
    let max = if a.1 > b.1 { a.1 } else { b.1 };
    (min, max)
}

/// Returns (min, max) relative to `range`, i.e. if `arg` equals `range`, returns (0, 1).
///
/// ```
/// use backend::library::minmax::get_relative_minmax;
/// assert_eq!(get_relative_minmax((1.0, 3.0), (1.0, 3.0)), (0.0, 1.0));
/// assert_eq!(get_relative_minmax((1.0, 3.0), (0.0, 4.0)), (0.25, 0.75));
/// assert_eq!(get_relative_minmax((-1.0, 6.0), (0.0, 4.0)), (-0.25, 1.5));
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
