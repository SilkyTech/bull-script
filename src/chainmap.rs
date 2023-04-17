use std::collections::HashMap;

pub struct ChainMap<K, V> {
    vec: Vec<HashMap<K, V>>,
}

impl<K, V> ChainMap<K, V> {
    pub fn new() -> Self {
        ChainMap { vec: vec![] }
    }

    pub fn push_hash(&mut self) {
        self.vec.push(HashMap::new());
    }

    pub fn pop_hash(&mut self) {
        self.vec.pop();
    }
}
