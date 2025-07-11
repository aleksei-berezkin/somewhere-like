#[macro_export]
macro_rules! assert_eq_d {
    ($lhs:expr, $rhs:expr, $msg:expr) => {
        assert!(
            ($lhs as f64 - $rhs as f64).abs() < $crate::library::test_util::ASSERT_EQ_D_EPSILON,
            "lhs=\"{}\", rhs=\"{}\", message: {}",
            $lhs,
            $rhs,
            $msg
        );
    };
    ($lhs:expr, $rhs:expr) => {
        assert!(
            ($lhs as f64 - $rhs as f64).abs() < $crate::library::test_util::ASSERT_EQ_D_EPSILON,
            "lhs=\"{}\", rhs=\"{}\"",
            $lhs,
            $rhs
        );
    };
}

// Not `#[cfg(test)]` to be visible from doctests
pub const ASSERT_EQ_D_EPSILON: f64 = 0.0001;
