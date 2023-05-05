use std::{collections::HashMap, hash::Hash, fmt::Debug};


#[derive(Debug, Clone)]
pub struct ChainMapElement<T> {
    pub val: T,
    pub name: String,
}
#[derive(Debug, Clone)]
pub struct NamedChainMap<K, V> {
    vec: Vec<ChainMapElement<HashMap<K, V>>>,
}

impl<K: Hash + PartialEq + Eq + Clone + Debug, V: Clone + Debug> NamedChainMap<K, V> {
    pub fn new() -> Self {
        NamedChainMap { vec: vec![] }
    }

    pub fn push_hash(&mut self, name: String) {
        self.vec.push(ChainMapElement { name, val: HashMap::new() });
    }

    pub fn pop_hash(&mut self) -> Option<ChainMapElement<HashMap<K, V>>> {
        self.vec.pop()
    }

    pub fn insert_top(&mut self, key: K, value: V) -> Option<()> {
        match self.vec.iter().last() {
            None => return None,
            Some(_) => {
                let i = self.vec.len()-1;
                self.vec[i].val.insert(key, value);
                return Some(())
            }
        }
    }

    pub fn move_to_back(&mut self) -> Option<()> {
        match self.vec.iter().last() {
            None => return None,
            Some(_) => {
                let p = self.vec.pop().unwrap();
                self.vec.insert(0, p.clone());
                dbg!(self.vec.clone());
                return Some(())
            }
        }
    }
    
    pub fn find(&self, key: K) -> Option<V> {
        for hash in self.vec.iter() {
            if let Some(v) = hash.val.get(&key) {
                return Some(v.clone());
            }
        }
        return None;
    }

    pub fn top(&self) -> Option<&ChainMapElement<HashMap<K, V>>> {
        self.vec.iter().last()
    }

}
