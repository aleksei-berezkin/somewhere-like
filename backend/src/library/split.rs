/// Split the input into all possible combinations of the form "name rest".
/// Order of variants starts from the smallest name.
/// The last variant is always (input, None), and it's the only case
/// when None is returned.
/// 
/// ```rust
/// use backend::library::split::split_name_rest;
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
