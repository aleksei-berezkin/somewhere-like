use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

pub type InternId = usize;

pub struct InternBuilder {
    next_id: AtomicUsize,
    string_to_id: DashMap<Vec<char>, InternId>,
}

impl InternBuilder {
    pub fn new() -> InternBuilder {
        InternBuilder {
            next_id: AtomicUsize::new(0),
            string_to_id: DashMap::new(),
        }
    }

    pub fn intern(&self, string: Vec<char>) -> InternId {
        *self.string_to_id
            .entry(string)
            .or_insert_with(|| self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    pub fn build(self) -> InternRegistry {
        let len = self.next_id.load(Ordering::Relaxed);
        let mut id_to_string_maybe = vec![None; len];
        self.string_to_id.into_iter()
            .for_each(|(string, id)| {
                id_to_string_maybe[id] = Some(string);
            });

        let id_to_string = id_to_string_maybe.into_iter().enumerate()
            .map(|(index, opt)| opt.expect(&format!("Missing string for id={}", index)))
            .collect();

        InternRegistry { id_to_string }
    }
}

pub struct InternRegistry {
    id_to_string: Vec<Vec<char>>
}

impl InternRegistry {
    pub fn len(&self) -> usize {
        self.id_to_string.len()
    }
    pub fn resolve(&self, id: InternId) -> Option<&Vec<char>> {
        self.id_to_string.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;

    #[test]
    fn test_empty() {
        let registry = InternBuilder::new().build();
        assert_eq!(0, registry.len());
        assert_eq!(None, registry.resolve(0));
        assert_eq!(None, registry.resolve(1));
        assert_eq!(None, registry.resolve(100));
    }

    #[test]
    fn test_several() {
        let builder = InternBuilder::new();
        let a_id = builder.intern(v("a"));
        assert_eq!(0, a_id);
        let b_id = builder.intern(v("b"));
        assert_eq!(1, b_id);

        let registry = builder.build();
        assert_eq!(2, registry.len());

        assert_eq!(Some(&v("a")), registry.resolve(0));
        assert_eq!(Some(&v("b")), registry.resolve(1));
        assert_eq!(None, registry.resolve(2));
    }

    #[test]
    fn test_multithreaded() {
        let len = 10_000;
        let builder = InternBuilder::new();
        (0..len).into_par_iter()
            .for_each(|num| {
                builder.intern(v(&num.to_string()));
            });
        let registry = builder.build();
        assert_eq!(len, registry.len());

        let mut values = (0..len).into_par_iter()
            .map(|id| {
                registry.resolve(id).unwrap().iter().collect::<String>().parse::<usize>().unwrap()
            })
            .collect::<Vec<_>>();
        values.sort();

        assert_eq!((0..len).into_iter().collect::<Vec<usize>>(), values);
    }

    fn v(s: &str) -> Vec<char> {
        s.chars().collect()
    }
}
