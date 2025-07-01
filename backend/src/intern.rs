use std::collections::HashMap;

pub type InternedId = u32;

pub struct Interner {
    first_id: InternedId,
    string_to_id: HashMap<Vec<char>, InternedId>,
    id_to_string: Vec<Vec<char>>
}

impl Interner {
    pub fn new() -> Interner {
        Interner {
            first_id: 0,
            string_to_id: HashMap::<Vec<char>, InternedId>::new(),
            id_to_string: Vec::<Vec<char>>::new(),
        }
    }

    pub fn len(&self) -> u32 {
        self.string_to_id.len() as u32
    }

    pub fn intern(&mut self, string: Vec<char>) -> InternedId {
        if let Some(id) = self.string_to_id.get(&string) {
            return *id
        }

        let new_id = self.first_id + self.string_to_id.len() as InternedId;
        self.string_to_id.insert(string.clone(), new_id);
        self.id_to_string.push(string);
        new_id
    }

    pub fn resolve(self: &Self, id: InternedId) -> Option<&Vec<char>> {
        let index = id as isize - self.first_id as isize;
        if index < 0 {
            return None;
        }
        self.id_to_string.get(index as usize)
    }

}
