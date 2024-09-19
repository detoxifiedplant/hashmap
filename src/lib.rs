#![allow(unused_imports)]

use core::panic;
use std::io::Cursor;
use hashbrown::hash_map::Keys;
use rand::distributions::{Uniform, Distribution};
use std::borrow::Borrow;
use std::cmp::max;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap as StdHashMap;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::mem::{swap, take};
pub mod raw;

const EMPTY: u8 = 0b1111_1111;
const DELETED: u8 = 0b1000_0000;
const GROUP_SIZE: u8 = 16;
type GroupWord = u64;
pub(crate) struct Group(GroupWord);

impl Group{
    pub(crate) unsafe fn load(ptr: *const u8) -> Self {
        Group(ptr::read_unaligned(ptr.cast()))
    }
}

// Checks whether a control byte represents a full bucket (top bit is clear).
#[inline]
fn is_full(ctrl: u8) -> bool {
    ctrl & 0x80 == 0
}

// Checks whether a control byte represents a special value (top bit is set).
#[inline]
fn is_special(ctrl: u8) -> bool {
    ctrl & 0x80 != 0
}

// Checks whether a special control value is EMPTY (just check 1 bit).
#[inline]
fn special_is_empty(ctrl: u8) -> bool {
    debug_assert!(is_special(ctrl));
    ctrl & 0x01 != 0
}

const BIT_SHIFT: u8 = 64 - 7;
fn h2(hash: u64) -> u8 {
    let top7_bits = hash >> (BIT_SHIFT);
    (top7_bits & 0x7f) as u8 // truncation
}



#[derive(Debug)]
enum Entry<Key, Value> {
    Vacant,
    Deleted,
    Occupied { key: Key, val: Value },
}

#[derive(Debug)]
struct HashMap<Key, Val> {
    data: Vec<Entry<Key, Val>>,

    // Mask to get an index from a hash value. The value is one less than the
    // number of buckets in the table.
    control_bytes: Vec<u8>,

    n_occupied: usize,
    n_vacant: usize,
}

impl<Key, Val> Entry<Key, Val> {
    fn take(&mut self) -> Option<Val> {
        match self {
            Self::Occupied { .. } => {
                // TODO: implement round robin and eliminate deleted
                let mut occupied = Self::Deleted;
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

impl<Key: Eq + Hash + Debug, Val> HashMap<Key, Val> {
    // unsafe fn find_inner(&self, hash: u64, eq: &mut dyn FnMut(usize) -> bool) -> Option<usize> {
    //     let h2_hash = h2(hash);
    //     let mut probe_seq = self.probe_seq(hash);
    //
    //     loop {
    //         // SAFETY:
    //         // * Caller of this function ensures that the control bytes are properly initialized.
    //         //
    //         // * `ProbeSeq.pos` cannot be greater than `self.bucket_mask = self.buckets() - 1`
    //         //   of the table due to masking with `self.bucket_mask`.
    //         //
    //         // * Even if `ProbeSeq.pos` returns `position == self.bucket_mask`, it is safe to
    //         //   call `Group::load` due to the extended control bytes range, which is
    //         //  `self.bucket_mask + 1 + Group::WIDTH` (in fact, this means that the last control
    //         //   byte will never be read for the allocated table);
    //         //
    //         // * Also, even if `RawTableInner` is not already allocated, `ProbeSeq.pos` will
    //         //   always return "0" (zero), so Group::load will read unaligned `Group::static_empty()`
    //         //   bytes, which is safe (see RawTableInner::new_in).
    //         let group = unsafe { Group::load(self.ctrl(probe_seq.pos)) };
    //
    //         for bit in group.match_byte(h2_hash) {
    //             // This is the same as `(probe_seq.pos + bit) % self.buckets()` because the number
    //             // of buckets is a power of two, and `self.bucket_mask = self.buckets() - 1`.
    //             let index = (probe_seq.pos + bit) & self.bucket_mask;
    //
    //             if likely(eq(index)) {
    //                 return Some(index);
    //             }
    //         }
    //
    //         if likely(group.match_empty().any_bit_set()) {
    //             return None;
    //         }
    //
    //         probe_seq.move_next(self.bucket_mask);
    //     }
    // }

    fn new() -> Self {
        Self {
            data: Vec::with_capacity(64),
            control_bytes: vec![EMPTY; 64],
            n_occupied: 0,
            n_vacant: 0,
        }
    }

    fn insert(&mut self, key: Key, value: Val) -> Option<Val> {
        if self.load_factor() >= 0.85 {
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
            match &self.data[idx] {
                Entry::Vacant => break None,
                Entry::Occupied { key: k, val } if k.borrow() == key => {
                    break Some(val);
                }
                _ => idx = (idx + 1) % self.data.len(),
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
    fn get_index<Q>(&self, key: &Q) -> u64
    where
        Key: Borrow<Q>,
        Q: Eq + Hash + Debug,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        let h2_hash = h2(hash);
        let number_of_groups = self.data.capacity() >> 4;
        let mut target_group = hash & (number_of_groups as u64 - 1);
        for i in 0..number_of_groups {
            for bit in target_group.match_byte(h2_hash) {
                let index = (probe_seq.pos + bit) & self.bucket_mask;

                if likely(eq(index)) {
                    return Some(index);
                }
            }

            if likely(group.match_empty().any_bit_set()) {
                return None;
            }
            // We should have found an empty bucket by now and ended the probe.
            target_group = target_group + ((i as u64 + 1) * i as u64) / 2;
        }

        0
    }

    fn iter_mut_starting_at(&mut self, idx: usize) -> impl Iterator<Item = &mut Entry<Key, Val>> {
        let (s1, s2) = self.data.split_at_mut(idx);
        s2.iter_mut().chain(s1.iter_mut())
    }

    fn load_factor(&self) -> f64 {
        if self.data.is_empty() {
            0.0
        } else {
            self.n_occupied as f64 / self.data.capacity() as f64
        }
    }

    fn occupied_factor(&self) -> f64 {
        if self.data.is_empty() {
            1.0
        } else {
            self.n_occupied as f64 / self.data.len() as f64
        }
    }

    fn resize(&mut self) {
        let resize_factor = if self.occupied_factor() > 0.75 { 2 } else { 1 };
        let new_size = max(64, self.data.len() * resize_factor);

        let mut new_table = Self {
            data: (0..new_size).map(|_| Entry::Vacant).collect(),
            control_bytes: vec![EMPTY; new_size],
            n_occupied: 0,
            n_vacant: new_size,
        };
        for entry in take(&mut self.data) {
            if let Entry::Occupied { key, val } = entry {
                new_table.insert_helper(key, val);
            }
        }
        swap(self, &mut new_table)
    }

    fn insert_helper(&mut self, key: Key, val: Val) -> Option<Val> {
        let idx = self.get_index(&key);
        let mut result = None;
        let mut swap_deleted: Option<&mut Entry<Key, Val>> = None;
        for entry in self.iter_mut_starting_at(idx) {
            match entry {
                Entry::Occupied { key: k, .. } if (k as &Key).borrow() == &key => {
                    result = entry.replace(val);
                    break;
                }
                Entry::Deleted if swap_deleted.is_none() => {
                    swap_deleted = Some(entry);
                }
                Entry::Vacant => {
                    if swap_deleted.is_some() {
                        *swap_deleted.unwrap() = Entry::Occupied { key, val };
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

    #[test]
    fn check() {
        // test::<HashMap<i64, i64>>();
        // test::<std::collections::HashMap<i64, i64>>();
    }

    fn test() {
        let mut map = HashMap::new();

        let key_gen = Uniform::from(0..1000);
        let op_gen = Uniform::from(0..4);
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
    }
}
