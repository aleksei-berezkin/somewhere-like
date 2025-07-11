const EARTH_RADIUS_KM: f64 = 6_371.0;

/// https://en.wikipedia.org/wiki/Chord_(geometry)#In_trigonometry
/// 
/// ```
/// use backend::assert_eq_d;
/// use backend::library::earth::arc_length_to_chord_length;
/// 
/// assert_eq_d!(0.0, arc_length_to_chord_length(0.0));
/// assert_eq_d!(199.991788, arc_length_to_chord_length(200.0));
/// assert_eq_d!(1_991.797834, arc_length_to_chord_length(2_000.0));
/// assert_eq_d!(12_741.991068, arc_length_to_chord_length(20_000.0));
/// ```
pub fn arc_length_to_chord_length(a: f64) -> f64 {
    let theta = a / EARTH_RADIUS_KM;
    EARTH_RADIUS_KM * 2.0 * (theta / 2.0).sin()
}

/// ```
/// use backend::assert_eq_d;
/// use backend::library::earth::get_cartesian_xyz;
/// 
/// let R = 6371.0;
/// 
/// let a = get_cartesian_xyz(0.0, 0.0);
/// assert_eq_d!(R, a[0]);
/// assert_eq_d!(0.0, a[1]);
/// assert_eq_d!(0.0, a[2]);
///
/// let b = get_cartesian_xyz(0.0, 90.0);
/// assert_eq_d!(0.0, b[0]);
/// assert_eq_d!(R, b[1]);
/// assert_eq_d!(0.0, b[2]);
/// 
/// let c = get_cartesian_xyz(-90.0, 111.0);
/// assert_eq_d!(0.0, c[0]);
/// assert_eq_d!(0.0, c[1]);
/// assert_eq_d!(-R, c[2]);
///
/// let d = get_cartesian_xyz(48.158430, 11.542951); // Munich
/// assert_eq_d!(4_163.968320, d[0]);
/// assert_eq_d!(850.420094, d[1]);
/// assert_eq_d!(4_746.345382, d[2]);
/// 
/// let e = get_cartesian_xyz(-43.951479, -176.560482); // Waitangi, Chatham Islands
/// assert_eq_d!(-4_578.398091, e[0]);
/// assert_eq_d!(-275.176052, e[1]);
/// assert_eq_d!(-4_421.785846, e[2]);
/// 
/// ```
pub fn get_cartesian_xyz(lat: f64, lon: f64) -> [f64; 3] {
    [
        EARTH_RADIUS_KM * lat.to_radians().cos() * lon.to_radians().cos(),
        EARTH_RADIUS_KM * lat.to_radians().cos() * lon.to_radians().sin(),
        EARTH_RADIUS_KM * lat.to_radians().sin(),
    ]
}

/// ```
/// use backend::library::earth::get_cartesian_distance_km_squared;
/// 
/// assert_eq!(0.0, get_cartesian_distance_km_squared(&[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0]));
/// assert_eq!(3.0, get_cartesian_distance_km_squared(&[1.0, 0.0, 1.0], &[0.0, 1.0, 0.0]));
/// assert_eq!(56.0, get_cartesian_distance_km_squared(&[-1.0, 2.0, -3.0], &[1.0, -2.0, 3.0]));
/// ```
pub fn get_cartesian_distance_km_squared(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// https://en.wikipedia.org/wiki/Great-circle_distance#Formulae
/// 
/// ```
/// use backend::assert_eq_d;
/// use backend::library::earth::get_arc_distance_km;
/// 
/// // One minute (1/60 deg) is approx. one sea mile
/// let minute = 1.0 / 60.0;
/// assert_eq_d!(1.85325, get_arc_distance_km(0.0, -100.0, 0.0, -100.0 + minute));
/// assert_eq_d!(1.85325, get_arc_distance_km(0.0, 100.0, 0.0, 100.0 - minute));
/// assert_eq_d!(1.85325, get_arc_distance_km(80.0, -50.0, 80.0 + minute, -50.0));
/// assert_eq_d!(1.85325, get_arc_distance_km(-70.0, 50.0, -70.0 - minute, 50.0));
/// 
/// // Almost square
/// assert_eq_d!(2.62089, get_arc_distance_km(0.0, 0.0, minute, -minute));
/// 
/// // Copenhagen to Lisbon
/// assert_eq_d!(2_477.55360, get_arc_distance_km(55.674802, 12.569040, 38.720452, -9.139727));
/// 
/// // Pole to pole
/// assert_eq_d!(20_015.08680, get_arc_distance_km(90.0, -111.0, -90.0, 88.8));
/// assert_eq_d!(20_015.08680, get_arc_distance_km(-40.0, 10.0, 40.0, -170.0));
/// 
/// // Overlap
/// assert_eq_d!(111.19493, get_arc_distance_km(0.0, 0.0, 0.0, 359.0));

/// // Full circle
/// assert_eq_d!(0.0, get_arc_distance_km(0.0, 0.0, 0.0, 360.0));
/// ```
pub fn get_arc_distance_km(a_lat: f64, a_lon: f64, b_lat: f64, b_lon: f64) -> f64 {
    let phi_a = a_lat.to_radians();
    let phi_b = b_lat.to_radians();
    let lambda_a = a_lon.to_radians();
    let lambda_b = b_lon.to_radians();
    EARTH_RADIUS_KM * (phi_a.sin() * phi_b.sin() + phi_a.cos() * phi_b.cos() * (lambda_a - lambda_b).abs().cos()).acos()
}
