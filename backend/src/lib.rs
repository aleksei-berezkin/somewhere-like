pub mod intern;

/// Copied from strsim::generic_jaro_winkler and optimized for Vec.
/// See https://docs.rs/strsim/latest/strsim/fn.generic_jaro_winkler.html
/// 
/// ```
/// use strsim::jaro_winkler;
/// use backend::jaro_winkler_vec;
/// 
/// fn assert_eq_with_delta(a: f64, b: f32) {
///     assert!((a as f32 - b).abs() < 0.0001);
/// }
/// 
/// assert_eq_with_delta(jaro_winkler("ab", "cd"), jaro_winkler_vec(&vec!['a', 'b'], &vec!['c', 'd']));
/// assert_eq_with_delta(jaro_winkler("abc", "abcde"), jaro_winkler_vec(&vec!['a', 'b', 'c'], &vec!['a', 'b', 'c', 'd', 'e']));
/// assert_eq_with_delta(jaro_winkler("abcde", "abc"), jaro_winkler_vec(&vec!['a', 'b', 'c', 'd', 'e'], &vec!['a', 'b', 'c']));
/// assert_eq_with_delta(jaro_winkler("abcdef", "abcdefg"), jaro_winkler_vec(&vec!['a', 'b', 'c', 'd', 'e', 'f'], &vec!['a', 'b', 'c', 'd', 'e', 'f', 'g']));
/// ```
pub fn jaro_winkler_vec<T: Eq>(a: &Vec<T>, b: &Vec<T>) -> f32 {
    let sim = jaro_vec(a, b);

    if sim > 0.7 {
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
/// 
/// ```
/// use strsim::jaro;
/// use backend::jaro_vec;
/// 
/// fn assert_eq_with_delta(a: f64, b: f32) {
///     assert!((a as f32 - b).abs() < 0.0001);
/// }
/// 
/// let empty = Vec::<char>::new();
/// assert_eq_with_delta(1.0, jaro_vec(&empty, &empty));
/// assert_eq_with_delta(0.0, jaro_vec(&vec!['a'], &empty));
/// assert_eq_with_delta(0.0, jaro_vec(&empty, &vec!['a']));
/// assert_eq_with_delta(1.0, jaro_vec(&vec!['a'], &vec!['a']));
/// assert_eq_with_delta(jaro("a", "ab"), jaro_vec(&vec!['a'], &vec!['a', 'b']));
/// assert_eq_with_delta(jaro("ab", "a"), jaro_vec(&vec!['a', 'b'], &vec!['a']));
/// assert_eq_with_delta(0.0, jaro_vec(&vec!['a', 'b', 'c'], &empty));
/// assert_eq_with_delta(jaro("abc", "bac"), jaro_vec(&vec!['a', 'b', 'c'], &vec!['b', 'a', 'c']));
/// assert_eq_with_delta(jaro("abc", "ab"), jaro_vec(&vec!['a', 'b', 'c'], &vec!['a', 'b']));
/// assert_eq_with_delta(jaro("ab", "abc"), jaro_vec(&vec!['a', 'b'], &vec!['a', 'b', 'c']));
/// assert_eq_with_delta(jaro("abcd", "abd"), jaro_vec(&vec!['a', 'b', 'c', 'd'], &vec!['a', 'b', 'c']));
/// assert_eq_with_delta(jaro("abcd", "babd"), jaro_vec(&vec!['a', 'b', 'c', 'd'], &vec!['b', 'a', 'b', 'd']));
/// assert_eq_with_delta(jaro("abc def", "ab de"), jaro_vec(&vec!['a', 'b', 'c', ' ', 'd', 'e', 'f'], &vec!['a', 'b', ' ', 'd', 'e']));
/// assert_eq_with_delta(jaro("abc def ghi", "ab d hi"), jaro_vec(&vec!['a', 'b', 'c', ' ', 'd', 'e', 'f', ' ', 'g', 'h', 'i'], &vec!['a', 'b', ' ', 'd', ' ', 'h', 'i']));
/// assert_eq_with_delta(jaro("abc123def", "abc321def"), jaro_vec(&vec!['a', 'b', 'c', '1', '2', '3', 'd', 'e', 'f'], &vec!['a', 'b', 'c', '2', '3', '1', 'd', 'e', 'f']));
/// ```
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

/// Split the input into all possible combinations of the form "name rest".
/// Order of variants starts from the smallest name.
/// The last variant is always (input, None), and it's the only case
/// when None is returned.
/// 
/// ```rust
/// use backend::split_name_rest;
/// assert_eq!(split_name_rest(""), vec![("", None)]);
/// assert_eq!(split_name_rest("ab"), vec![("ab", None)]);
/// assert_eq!(split_name_rest("a b c"), vec![("a", Some("b c")), ("a b", Some("c")), ("a b c", None)]);
/// assert_eq!(split_name_rest(";,a, ,b,;"), vec![(";,a", Some("b,;")), (";,a, ,b,;", None)]);
/// ```
pub fn split_name_rest(input: &str) -> Vec<(&str, Option<&str>)> {
    let delimiter = regex::Regex::new(r"[ ,;]+").unwrap();
    delimiter.find_iter(&input)
        .filter_map(|m| {
            let name = &input[..m.start()];
            let rest = &input[m.end()..];
            if name.len() > 0 && rest.len() > 0 {
                Some((name, Some(rest)))
            } else {
                None
            }
        })
        .chain(std::iter::once((input, None)))
        .collect()
}
