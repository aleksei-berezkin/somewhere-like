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
