use core::panic;
use rand::distributions::{Uniform, Distribution};
use std::borrow::Borrow;
use std::cmp::max;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap as StdHashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::mem::{swap, take};

#[derive(Debug)]
enum Entry<Key, Value> {
    Vacant,
    Tombstone,
    Occupied { key: Key, val: Value },
}

#[derive(Debug)]
struct HashMap<Key, Val> {
    xs: Vec<Entry<Key, Val>>,
    n_occupied: usize,
    n_vacant: usize,
}

impl<Key, Val> Entry<Key, Val> {
    fn take(&mut self) -> Option<Val> {
        match self {
            Self::Occupied { .. } => {
                let mut occupied = Self::Tombstone;
                swap(self, &mut occupied);
                if let Self::Occupied { key: _, val } = occupied {
                    Some(val)
                } else {
                    panic!("unreachable");
                }
            }
            _ => None,
        }
    }

    fn replace(&mut self, mut x: Val) -> Option<Val> {
        match self {
            Self::Occupied { val, .. } => {
                swap(&mut x, val);
                Some(x)
            }
            _ => None,
        }
    }
}

pub trait MapTrait<Key, Val> {
    fn new() -> Self;
    fn insert(&mut self, key: Key, val: Val) -> Option<Val>;
    fn get(&self, key: &Key) -> Option<&Val>;
    fn remove(&mut self, key: &Key) -> Option<Val>;
    fn get_mut(&mut self, key: &Key) -> Option<&mut Val>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

// impl<Key, Val> MapTrait<Key, Val> for HashMap<Key, Val> {}
impl<Key, Val> MapTrait<Key, Val> for std::collections::HashMap<Key, Val>
where
    Key: std::cmp::Eq + std::hash::Hash,
{
    fn new() -> Self {
        StdHashMap::new()
    }
    fn insert(&mut self, key: Key, val: Val) -> Option<Val> {
        StdHashMap::insert(self, key, val)
    }
    fn get(&self, key: &Key) -> Option<&Val> {
        StdHashMap::get(self, key)
    }
    fn remove(&mut self, key: &Key) -> Option<Val> {
        StdHashMap::remove(self, key)
    }
    fn get_mut(&mut self, key: &Key) -> Option<&mut Val> {
        StdHashMap::get_mut(self, key)
    }
    fn len(&self) -> usize {
        StdHashMap::len(self)
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<Key: Eq + Hash, Val> MapTrait<Key, Val> for HashMap<Key, Val> {
    fn new() -> Self {
        Self {
            xs: Vec::new(),
            n_occupied: 0,
            n_vacant: 0,
        }
    }

    fn insert(&mut self, key: Key, value: Val) -> Option<Val> {
        if self.load_factor() >= 0.75 {
            self.resize();
        }

        self.insert_helper(key, value)
    }

    fn len(&self) -> usize {
        self.n_occupied
    }

    fn get(&self, key: &Key) -> Option<&Val>
    where
        Key: Borrow<Key> + Eq + Hash,
    {
        if self.len() == 0 {
            return None;
        }

        let mut idx = self.get_index(key);
        loop {
            match &self.xs[idx] {
                Entry::Vacant => break None,
                Entry::Occupied { key: k, val } if k.borrow() == key => {
                    break Some(val);
                }
                _ => idx = (idx + 1) % self.xs.len(),
            }
        }
    }

    fn get_mut(&mut self, key: &Key) -> Option<&mut Val>
    where
        Key: Borrow<Key> + Eq + Hash,
    {
        if self.len() == 0 {
            return None;
        }
        let idx = self.get_index(key);
        for entry in self.iter_mut_starting_at(idx) {
            match entry {
                Entry::Vacant => return None,
                Entry::Occupied { key: k, val } if (k as &Key).borrow() == key => {
                    return Some(val);
                }
                _ => {}
            }
        }
        panic!("unreachable");
    }

    fn remove(&mut self, key: &Key) -> Option<Val>
    where
        Key: Borrow<Key> + Eq + Hash,
    {
        if self.len() == 0 {
            return None;
        }
        let idx = self.get_index(key);
        let mut result = None;
        for entry in self.iter_mut_starting_at(idx) {
            match entry {
                Entry::Occupied { key: k, .. } if (k as &Key).borrow() == key => {
                    result = entry.take();
                    break;
                }
                Entry::Vacant => {
                    break;
                }
                _ => {}
            }
        }
        result.map(|val| {
            self.n_occupied -= 1;
            val
        })
    }

    fn is_empty(&self) -> bool {
        self.n_occupied == 0
    }
}

impl<Key: Eq + Hash, Val> HashMap<Key, Val> {
    fn index(&self, hash: usize) -> usize {
        hash & (self.xs.len() - 1)
    }

    fn get_index<Q>(&self, key: &Q) -> usize
    where
        Key: Borrow<Q>,
        Q: Eq + Hash,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        // hasher.finish() as usize % self.xs.len()
        self.index(hasher.finish() as usize)
    }

    fn iter_mut_starting_at(&mut self, idx: usize) -> impl Iterator<Item = &mut Entry<Key, Val>> {
        let (s1, s2) = self.xs.split_at_mut(idx);
        s2.iter_mut().chain(s1.iter_mut())
    }

    fn load_factor(&self) -> f64 {
        if self.xs.is_empty() {
            1.0
        } else {
            1.0 - self.n_vacant as f64 / self.xs.len() as f64
        }
    }

    fn occupied_factor(&self) -> f64 {
        if self.xs.is_empty() {
            1.0
        } else {
            self.n_occupied as f64 / self.xs.len() as f64
        }
    }

    fn resize(&mut self) {
        let resize_factor = if self.occupied_factor() > 0.75 { 2 } else { 1 };
        let new_size = max(64, self.xs.len() * resize_factor);

        let mut new_table = Self {
            xs: (0..new_size).map(|_| Entry::Vacant).collect(),
            n_occupied: 0,
            n_vacant: new_size,
        };
        for entry in take(&mut self.xs) {
            if let Entry::Occupied { key, val } = entry {
                new_table.insert_helper(key, val);
            }
        }
        swap(self, &mut new_table)
    }

    fn insert_helper(&mut self, key: Key, val: Val) -> Option<Val> {
        let idx = self.get_index(&key);
        let mut result = None;
        let mut swap_tombstone: Option<&mut Entry<Key, Val>> = None;
        for entry in self.iter_mut_starting_at(idx) {
            match entry {
                Entry::Occupied { key: k, .. } if (k as &Key).borrow() == &key => {
                    result = entry.replace(val);
                    break;
                }
                Entry::Tombstone if swap_tombstone.is_none() => {
                    swap_tombstone = Some(entry);
                }
                Entry::Vacant => {
                    if swap_tombstone.is_some(){
                        *swap_tombstone.unwrap() = Entry::Occupied { key, val };
                    } else {
                        *entry = Entry::Occupied { key, val };
                    }
                    break;
                }
                _ => {}
            }
        }

        if result.is_none() {
            self.n_occupied += 1;
            self.n_vacant -= 1;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn check() {
    //     let mut map1 = HashMap::new();
    //     dbg!(map1.insert("namah", 9));
    //     println!("{:?}", map1);
    // }

    #[test]
    fn check() {
        // test::<HashMap<i64, i64>>();
        test::<std::collections::HashMap<i64, i64>>();
    }

    fn test<M>()
    where
        M: MapTrait<i64, i64> + Debug,
    {
        let mut map = M::new();

        let key_gen = Uniform::from(0..1000);
        let op_gen = Uniform::from(0..5);
        let mut rng = rand::thread_rng();

        for _ in 0..10_000_000 {
            let val = key_gen.sample(&mut rng);
            let key = val;
            match op_gen.sample(&mut rng) {
                0 => _ = map.insert(key, val),
                1 => {
                    map.get_mut(&key).map(|x| {
                        *x += 1;
                        x
                    });
                }
                2 => _ = map.get(&key),
                3 => _ = map.remove(&key),
                _ => (),
            }
        }
        println!("{:?}", map.len());
    }
}
