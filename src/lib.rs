use core::panic;
use rand::distributions::*;
use std::borrow::Borrow;
use std::cmp::max;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem::{swap, take};

enum Entry<Key, Value> {
    Vacant,
    Tombstone,
    Occupied { key: Key, val: Value },
}

struct HashMap<Key, Val> {
    xs: Vec<Entry<Key, Val>>,
    n_occupied: usize,
    n_vacant: usize,
}

impl<Key, Val> Entry<Key, Val> {
    fn take(&mut self) -> Option<Val> {
        match self {
            Self::Occupied { key, val } => {
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
            Self::Occupied { key, val } => {
                swap(&mut x, val);
                Some(x)
            }
            _ => None,
        }
    }
}

impl<Key: Eq + Hash, Val> HashMap<Key, Val> {
    pub fn new() -> Self {
        Self {
            xs: Vec::new(),
            n_occupied: 0,
            n_vacant: 0,
        }
    }

    fn index(&self, hash: usize) -> usize {
        hash & (self.xs.len() - 1)
    }

    pub fn get_index<Q>(&self, key: &Q) -> usize
    where
        Key: Borrow<Q>,
        Q: Eq + Hash,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        // hasher.finish() as usize % self.xs.len()
        self.index(hasher.finish() as usize)
    }

    pub fn len(&self) -> usize {
        self.n_occupied
    }

    pub fn insert(&mut self, key: Key, value: Val) -> Self {
        if self.load_factor() >= 0.75 {
            self.resize();
        }

        self.insert(key, value)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&Val>
    where
        Key: Borrow<Q>,
        Q: Eq + Hash,
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

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut Val>
    where
        Key: Borrow<Q>,
        Q: Eq + Hash,
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

    fn iter_mut_starting_at(&mut self, idx: usize) -> impl Iterator<Item = &mut Entry<Key, Val>> {
        let (s1, s2) = self.xs.split_at_mut(idx);
        s2.iter_mut().chain(s1.iter_mut())
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<Val>
    where
        Key: Borrow<Q>,
        Q: Eq + Hash,
    {
        if self.len() == 0 {
            return None;
        }
        let idx = self.get_index(key);
        let mut result = None;
        for entry in self.iter_mut_starting_at(idx) {
            match entry {
                Entry::Occupied { key: k, val } if (k as &Key).borrow() == key => {
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
        for entry in self.iter_mut_starting_at(idx) {
            match entry {
                Entry::Occupied { key: k, .. } if (k as &Key).borrow() == &key => {
                    result = entry.replace(val);
                    break;
                }
                Entry::Vacant => {
                    *entry = Entry::Occupied { key, val };
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

fn test() {
    let mut map1 = HashMap::new();
    let mut map2 = std::collections::HashMap::new();

    let key_gen = Uniform::from(0..1000);
    let op_gen = Uniform::from(0..5);
    let mut rng = rand::thread_rng();

    for _ in 0..1_000_1000 {
        let val = key_gen.sample(&mut rng);
        let key = val;
        match op_gen.sample(&mut rng) {
            0 => {
                // insert
                assert_eq!(map1.insert(key, val), map2.insert(key, val));
            }
            1 => {
                // get mut
                assert_eq!(
                    map1.get_mut(&key).map(|x| {
                        *x += 1;
                        x
                    }),
                    map2.get_mut(&key).map(|x| {
                        *x += 1;
                        x
                    })
                );
            }
            2 => {
                assert_eq!(map1.get(&key), map2.get(&key));
            }
            3 => {
                assert_eq!(map1.remove(&key), map2.remove(&key));
            }
            _ => {
                assert_eq!(map1.len(), map2.len());
            }
        }
    }
}

fn run_bench_i64<M>()
where
    M: MapTrait<i64, i64>,
{
    let mut map = M::new();

    let key_gen = Uniform::from(0..1000);
    let op_gen = Uniform::from(0..4);
    let mut rng = rand::thread_rng();
    for _ in 0..10000000 {
        let val = key_gen.sample(&mut rng);
        let key = val;
        match op_gen.sample(&mut rng) {
            0 => {
                // insert
                map.insert(key, val);
            }
            1 => {
                // get mut
                map.get_mut(&key).map(|x| {
                    *x += 1;
                    x
                });
            }
            2 => {
                // get
                map.get(&key);
            }
            3 => {
                // remove
                map.remove(&key);
            }
            _ => {}
        }
    }
}
