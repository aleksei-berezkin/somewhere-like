use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

pub type InternedId = usize;

pub struct InternerBuilder {
    next_id: AtomicUsize,
    string_to_id: DashMap<Vec<char>, InternedId>,
}

impl InternerBuilder {
    pub fn new() -> InternerBuilder {
        InternerBuilder {
            next_id: AtomicUsize::new(0),
            string_to_id: DashMap::new(),
        }
    }

    pub fn intern(&self, string: Vec<char>) -> InternedId {
        *self.string_to_id
            .entry(string)
            .or_insert_with(|| self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    pub fn into_interner(self) -> Interner {
        let len = self.next_id.load(Ordering::Relaxed);
        let placeholder: Vec<char> = vec![];
        let mut id_to_string = vec![placeholder; len];
        self.string_to_id.into_iter()
            .for_each(|(string, id)| {
                id_to_string[id] = string;
            });
        Interner { id_to_string }
    }
}

pub struct Interner {
    id_to_string: Vec<Vec<char>>
}

impl Interner {
    pub fn len(&self) -> usize {
        self.id_to_string.len()
    }
    pub fn resolve(&self, id: InternedId) -> Option<&Vec<char>> {
        self.id_to_string.get(id)
    }
}
