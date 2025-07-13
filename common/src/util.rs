use std::{env::current_dir, path::PathBuf};

use num_traits::Float;

/// Returns path to `data_in` directory, assuming the current directory is somewhere inside the project.
/// ```
/// assert!(common::utils::get_data_in_dir().to_str().unwrap().ends_with("data-in"));
/// ```
pub fn get_data_in_dir() -> PathBuf {
    get_project_dir().join("data-in")
}

/// Returns path to `data_out` directory, assuming the current directory is somewhere inside the project.
/// ```
/// assert!(common::utils::get_data_out_dir().to_str().unwrap().ends_with("data-out"));
/// ```
pub fn get_data_out_dir() -> PathBuf {
    get_project_dir().join("data-out")
}

fn get_project_dir() -> PathBuf {
    let current_dir = current_dir().unwrap();

    let mut dir = current_dir.as_path();
    loop {
        if dir.file_name().unwrap() == "somewhere-like" {
            break dir.to_path_buf();
        }
        dir = dir.parent().expect(&format!("Project root dir not found in: {:?}", current_dir));
    }
}

pub fn eprintln_memory_usage() {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    let mem_bytes = sys.process(sysinfo::get_current_pid().unwrap()).unwrap().memory();
    eprintln!("Using {} MB of RAM", (mem_bytes as f64) / 1024.0 / 1024.0);
}

/// Rounds to 1 decimal place and checks it's finite.
/// 
/// Because the most data is int * 0.1, this util is useful
/// to remove rounding artifacts like in the first example.
/// 
/// ```
/// use common::utils::round_0_1_and_assert_finite;
/// 
/// assert_eq!(1.2, round_0_1_and_assert_finite(1.200007));
/// assert_eq!(1.2, round_0_1_and_assert_finite(1.19));
/// assert_eq!(-10.6, round_0_1_and_assert_finite(-10.56));
/// assert_eq!(0.0, round_0_1_and_assert_finite(0.0));
/// ```
/// # Panics
/// 
/// NAN
/// ```rust,should_panic
/// use common::utils::round_0_1_and_assert_finite;
/// round_0_1_and_assert_finite(f32::NAN);
/// ```
/// 
/// INFINITY
/// ```rust,should_panic
/// use common::utils::round_0_1_and_assert_finite;
/// round_0_1_and_assert_finite(f32::INFINITY);
/// ```
/// 
/// NEG_INFINITY
/// ```rust,should_panic
/// use common::utils::round_0_1_and_assert_finite;
/// round_0_1_and_assert_finite(f32::NEG_INFINITY);
/// ```
pub fn round_0_1_and_assert_finite<T: Float + std::fmt::Debug>(val: T) -> T {
    let ten = T::from(10.0).unwrap();
    let rounded = (val * ten).round() / ten;
    if !rounded.is_finite() {
        panic!("Non-finite value: {:?}", rounded);
    }
    rounded
}
