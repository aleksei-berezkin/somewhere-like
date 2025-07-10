/// Copied from strsim::generic_jaro_winkler and optimized for Vec.
/// See https://docs.rs/strsim/latest/strsim/fn.generic_jaro_winkler.html
pub fn jaro_winkler_vec<T: Eq>(a: &Vec<T>, b: &Vec<T>) -> f32 {
    let sim = jaro_vec(a, b);

    if sim > 0.69999 { // Rounding issue with f32 vs f64. Original uses f64, and threshold is 0.7 here
        let max_prefix_len = a.len().min(b.len()).min(4);
        let mut prefix_len = 0;
        while prefix_len < max_prefix_len && a[prefix_len] == b[prefix_len] {
            prefix_len += 1;
        }
        sim + 0.1 * prefix_len as f32 * (1.0 - sim)
    } else {
        sim
    }
}

/// Copied from strsim::generic_jaro and optimized for Vec.
/// See https://docs.rs/strsim/latest/strsim/fn.generic_jaro.html
pub fn jaro_vec<T: Eq>(a: &Vec<T>, b: &Vec<T>) -> f32 {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 && b_len == 0 {
        return 1.0;
    }

    if a_len == 0 || b_len == 0 {
        return 0.0;
    }

    let search_extension = (a_len.max(b_len) / 2).saturating_sub(1);

    // This is slightly faster than 2 separate vecs
    let mut flags_memory = vec![false; a_len + b_len];
    let (a_flags, b_flags) = flags_memory.split_at_mut(a_len);

    let mut matches = 0_usize;
    'outer: for i in 0..a_len {
        let j_from = i.saturating_sub(search_extension);
        let j_bound = (i + 1 + search_extension).min(b_len);
        for j in j_from..j_bound {
            if a[i] == b[j] && !b_flags[j] {
                a_flags[i] = true;
                b_flags[j] = true;
                matches += 1;
                if matches == b_len {
                    break 'outer;
                }
                break;
            }
        }
    }

    if matches == 0 {
        return 0.0
    }

    // Number of transpositions cannot exceed the number of matches
    let mut transpositions = 0_usize;
    if matches > 1 { // In case of exactly one match, no transpositions possible
        let mut j = 0_usize;
        for i in 0..a_len {
            if a_flags[i] {
                // Number of `true` flags match, so `j` will always be found
                while !b_flags[j] {
                    j += 1;
                }
                if a[i] != b[j] {
                    transpositions += 1;
                    if transpositions == matches {
                        break;
                    }
                }
                j += 1;
            }
        }
    }

    ((matches as f32 / a_len as f32)
        + (matches as f32 / b_len as f32)
        + ((matches - transpositions / 2) as f32 / matches as f32)
    ) / 3.0
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rng, Rng};
    use rayon::prelude::*;
    use strsim::{jaro, jaro_winkler};

    const COMMON_EXAMPLES: [(&str, &str, f32); 7] = [
        ("", "a", 0.0),
        ("a", "", 0.0),
        ("a", "b", 0.0),
        ("ab", "cd", 0.0),
        ("", "", 1.0),
        ("a", "a", 1.0),
        ("ab", "ab", 1.0),
    ];

    #[test]
    fn test_common() {
        for (a, b, expected) in COMMON_EXAMPLES {
            assert_eq!(expected, jaro_vec(&v(a), &v(b)), "jaro a=\"{}\", b=\"{}\"", a, b);
            assert_eq!(expected, jaro_winkler_vec(&v(a), &v(b)), "jaro-winkler a=\"{}\", b=\"{}\"", a, b);
        }
    }

    macro_rules! assert_eq_d {
        ($lhs:expr, $rhs:expr, $msg:expr) => {
            assert!(
                ($lhs as f32 - $rhs).abs() < 0.0001,
                "lhs=\"{}\", rhs=\"{}\", message: {}",
                $lhs,
                $rhs,
                $msg
            );
        };
        ($lhs:expr, $rhs:expr) => {
            assert_eq_d!($lhs, $rhs, "");
        };
    }

    #[test]
    fn test_jaro() {
        assert_eq_d!(jaro("a", "ab"), jaro_vec(&v("a"), &v("ab")));
        assert_eq_d!(jaro("ab", "a"), jaro_vec(&v("ab"), &v("a")));
        assert_eq_d!(jaro("abc", "bac"), jaro_vec(&v("abc"), &v("bac")));
        assert_eq_d!(jaro("abc", "ab"), jaro_vec(&v("abc"), &v("ab")));
        assert_eq_d!(jaro("ab", "abc"), jaro_vec(&v("ab"), &v("abc")));
        assert_eq_d!(jaro("abcd", "abd"), jaro_vec(&v("abcd"), &v("abd")));
        assert_eq_d!(jaro("abcd", "babd"), jaro_vec(&v("abcd"), &v("babd")));
        assert_eq_d!(jaro("abc def", "ab de"), jaro_vec(&v("abc def"), &v("ab de")));
        assert_eq_d!(jaro("abc def ghi", "ab d hi"), jaro_vec(&v("abc def ghi"), &v("ab d hi")));
        assert_eq_d!(jaro("abc123def", "abc321def"), jaro_vec(&v("abc123def"), &v("abc321def")));
    }

    #[test]
    fn test_jaro_winkler() {
        assert_eq_d!(jaro_winkler("a", "a"), jaro_winkler_vec(&v("a"), &v("a")));
        assert_eq_d!(jaro_winkler("ab", "cd"), jaro_winkler_vec(&v("ab"), &v("cd")));
        assert_eq_d!(jaro_winkler("abcdefgh", "a"), jaro_winkler_vec(&v("abcdefgh"), &v("a")));
        assert_eq_d!(jaro_winkler("a", "abcdefgh"), jaro_winkler_vec(&v("a"), &v("abcdefgh")));
        assert_eq_d!(jaro_winkler("abc", "abcde"), jaro_winkler_vec(&v("abc"), &v("abcde")));
        assert_eq_d!(jaro_winkler("abcde", "abc"), jaro_winkler_vec(&v("abcde"), &v("abc")));
        assert_eq_d!(jaro_winkler("abcdef", "abcdefg"), jaro_winkler_vec(&v("abcdef"), &v("abcdefg")));
    }


    #[test]
    fn test_randomized() {
        (0..100_000).into_par_iter().for_each(|_| {
            let a = make_random_str();
            let b = make_random_str();
            let msg = format!("a=\"{}\", b=\"{}\"", a, b);
    
            assert_eq_d!(jaro(&a, &b) as f32, jaro_vec(&v(&a), &v(&b)), msg);
            assert_eq_d!(jaro_winkler(&a, &b) as f32, jaro_winkler_vec(&v(&a), &v(&b)), msg);
        })
    }

    const RND_CHARS: [char; 11] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', ' '];
    const MAX_LEN: usize = 15;

    fn make_random_str() -> String {
        let len = rng().random_range(..=MAX_LEN);
        (0..len)
            .map(|_| RND_CHARS[rng().random_range(..RND_CHARS.len())])
            .collect::<String>()
    }

    fn v(s: &str) -> Vec<char> {
        s.chars().collect()
    }
}
