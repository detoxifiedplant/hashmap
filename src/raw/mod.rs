pub mod bitmask;
pub mod sse2;

pub const EMPTY: u8 = 0b1111_1111;
pub const DELETED: u8 = 0b1000_0000;

use sse2::Group;
use std::borrow::Borrow;
use std::cmp::max;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::mem::{swap, take};
const GROUP_SIZE: usize = 16;

const BIT_SHIFT: u8 = 64 - 7;
fn h2(hash: usize) -> u8 {
    let top7_bits = hash >> (BIT_SHIFT);
    (top7_bits & 0x7f) as u8 // truncation
}

#[derive(Debug, Clone, Copy)]
pub struct Entry<Key, Value> {
    key: Key,
    val: Value,
}

#[derive(Debug)]
pub struct HashMap<Key, Val> {
    data: Vec<Option<Entry<Key, Val>>>,
    control_bytes: Vec<u8>,
    n_occupied: usize,
    n_vacant: usize,
}

pub struct InsertSlot {
    index: usize,
}

impl<Key: Eq + Hash + Debug + Copy + Clone, Val: Debug + Copy + Clone> HashMap<Key, Val> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let new_size = 64;
        Self {
            data: vec![None; new_size],
            control_bytes: vec![EMPTY; new_size],
            n_occupied: 0,
            n_vacant: new_size,
        }
    }

    pub fn insert(&mut self, key: Key, value: Val) -> Option<Val> {
        if self.load_factor() >= 0.85 {
            self.resize();
        }

        self.insert_helper(key, value)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&Val>
    where
        Key: Borrow<Q>,
        Q: Eq + Hash,
    {
        if self.n_occupied == 0 {
            return None;
        }

        let idx = self.get_index(key);
        if let Some(index) = idx {
            return Some(&self.data[index].as_ref().unwrap().val);
        }
        None
    }

    pub fn get_mut(&mut self, key: &Key) -> Option<&mut Val>
    where
        Key: Borrow<Key> + Eq + Hash,
    {
        if self.is_empty() {
            return None;
        }
        let index = self.get_index(key)?;
        self.data[index].as_mut().map(|entry| &mut entry.val)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<Val>
    where
        Key: Borrow<Q>,
        Q: Eq + Hash,
    {
        if self.is_empty() {
            return None;
        }
        let index = self.get_index(key)?;
        self.data[index] = None;
        self.control_bytes[index] = DELETED;
        self.n_occupied -= 1;
        self.n_vacant += 1;
        None
    }

    fn is_empty(&self) -> bool {
        self.n_occupied == 0
    }

    fn get_hash<Q>(key: &Q) -> usize
    where
        Key: Borrow<Q>,
        Q: Eq + Hash,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Key: Borrow<Q>,
        Q: Eq + Hash,
    {
        let hash = Self::get_hash(key);
        let h2_hash = h2(hash);
        let number_of_groups = self.data.capacity() >> 4;
        let mut target_group = hash & (number_of_groups - 1);

        for i in 0..number_of_groups {
            target_group = (target_group + ((i + 1) * i) / 2) & (number_of_groups - 1);
            let group_base_index = target_group * GROUP_SIZE;
            let group = unsafe { Group::load(self.control_bytes.as_ptr().add(group_base_index)) };
            for bit in group.match_byte(h2_hash) {
                let index = group_base_index + bit;

                if self.data[index].unwrap().key.borrow() == key {
                    return Some(index);
                }
            }

            if group.match_empty().any_bit_set() {
                return None;
            }
        }
        None
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
        let resize_factor = if self.occupied_factor() > 0.85 { 2 } else { 1 };
        let new_size = max(64, self.data.len() * resize_factor);

        let mut new_table = Self {
            data: vec![None; new_size],
            control_bytes: vec![EMPTY; new_size],
            n_occupied: 0,
            n_vacant: new_size,
        };
        for entry in take(&mut self.data) {
            if let Some(e) = entry {
                new_table.insert_helper(e.key, e.val);
            }
        }
        swap(self, &mut new_table)
    }

    unsafe fn find_insert_slot(&self, hash: usize) -> InsertSlot {
        let number_of_groups = self.data.capacity() >> 4;
        let mut target_group = hash & (number_of_groups - 1);
        for i in 0..number_of_groups {
            target_group = (target_group + ((i + 1) * i) / 2) & (number_of_groups - 1);
            let group_base_index = target_group * GROUP_SIZE;
            let group = unsafe { Group::load(self.control_bytes.as_ptr().add(group_base_index)) };
            let index = self.find_insert_slot_in_group(&group, group_base_index);
            if let Some(idx) = index {
                return InsertSlot { index: idx };
            }
        }
        panic!("unreachable");
    }

    fn find_insert_slot_in_group(&self, group: &Group, probe_seq: usize) -> Option<usize> {
        let bit = group.match_empty_or_deleted().lowest_set_bit();

        bit.map(|b| (probe_seq + b) & (self.data.len() - 1))
    }

    fn insert_helper(&mut self, key: Key, val: Val) -> Option<Val> {
        let hash = Self::get_hash(&key);
        let entry_exists = self.get_index(&key);
        if let Some(index) = entry_exists {
            self.data[index] = Some(Entry { key, val });
            return Some(val);
        }
        unsafe {
            let slot = self.find_insert_slot(hash);

            self.insert_in_slot(hash, slot, Entry { key, val });
        }
        Some(val)
    }

    unsafe fn insert_in_slot(
        &mut self,
        hash: usize,
        slot: InsertSlot,
        entry: Entry<Key, Val>,
    ) -> Entry<Key, Val> {
        self.record_item_insert_at(slot.index, hash);
        self.data[slot.index] = Some(entry);
        entry
    }

    unsafe fn record_item_insert_at(&mut self, index: usize, hash: usize) {
        self.control_bytes[index] = h2(hash);
        self.n_occupied += 1;
        self.n_vacant -= 1;
    }
}
