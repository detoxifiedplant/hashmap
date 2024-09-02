use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[cfg(test)]
mod tests;

const DEFAULT_MAX_SIZE: u64 = 256;

pub struct HashMap<T, V> {
    curr_size: usize,
    arr: [Option<KeyValue<T, V>>; DEFAULT_MAX_SIZE as usize],
}

#[derive(Clone, Debug)]
pub struct KeyValue<T, V> {
    key: T,
    value: V,
    next: Option<Box<KeyValue<T, V>>>,
}

impl<T: std::cmp::PartialEq + Hash + Clone, V: Clone + Copy> HashMap<T, V> {
    // allows us to work around lack of `Copy` trait
    const INIT: Option<KeyValue<T, V>> = None;
    pub fn new() -> HashMap<T, V> {
        HashMap {
            curr_size: 0,
            arr: [Self::INIT; DEFAULT_MAX_SIZE as usize],
        }
    }

    /// Inserts a key: value pair into the hashmap
    ///
    /// Returns None if the key didn't exist
    /// Returns the old value if the key wasn't present
    /// and updates it with the new value.
    pub fn put(&mut self, key: T, val: V) -> Option<V> {
        let hash_val: u64 = hash_key(key.clone());

        let position = hash_val % DEFAULT_MAX_SIZE;

        match &self.arr[position as usize] {
            Some(_) => self.update_or_link_new_val(key, val, position as usize),
            None => {
                self.insert_new_value(key, val, position as usize);
                None
            }
        }
    }

    /// Gets a the given value for a key.
    ///
    /// Returns the value if it exists
    /// None otherwise
    pub fn get(&self, key: T) -> Option<V> {
        let hash_val: u64 = hash_key(key.clone());
        let position = hash_val % DEFAULT_MAX_SIZE;

        match &self.arr[position as usize] {
            Some(_) => self.check_list_for_key(key, position as usize),
            None => None,
        }
    }

    /// Removes a value from the map, returning the value
    /// if that key existed.
    ///
    /// Returns none if the value does not exist.
    pub fn remove(&mut self, key: T) -> Option<V> {
        let hash_val: u64 = hash_key(key.clone());
        let position: u64 = hash_val % DEFAULT_MAX_SIZE;

        match &self.arr[position as usize] {
            Some(_) => self.check_item_in_list_and_remove(key, position as usize),
            None => None,
        }
    }

    /// Clears the HashMap
    pub fn clear(&mut self) {
        // overwrite the array to yeet everything
        self.curr_size = 0;
        self.arr = [Self::INIT; DEFAULT_MAX_SIZE as usize];
    }

    /// Returns the number of keys in
    /// the HashMap
    pub fn length(&self) -> usize {
        self.curr_size
    }

    fn insert_new_value(&mut self, key: T, val: V, position: usize) {
        let new_entry = KeyValue::new(key, val);

        self.arr[position] = Some(new_entry);
        self.curr_size += 1;
    }

    fn update_or_link_new_val(&mut self, key: T, val: V, position: usize) -> Option<V> {
        // traverse linked list until either find value (update)
        // or stick a new value on the end

        // can safely unwrap as we've already checked this pos exists
        let key_val = self.arr[position].as_mut().unwrap();
        if key_val.key == key {
            let old_val = key_val.value;
            key_val.value = val;
            // return the old value
            return Some(old_val);
        }

        let mut current = key_val;
        while current.next.is_some() {
            let node = current.next.as_mut().unwrap();

            if node.key == key {
                // update the value
                let old_val = node.value;
                node.value = val;
                return Some(old_val);
            }

            current = node;
        }

        // append the new value to the end of the linked list
        let new_key_val = KeyValue::new(key, val);

        current.next = Some(Box::new(new_key_val));
        self.curr_size += 1;

        None
    }

    fn check_list_for_key(&self, key: T, position: usize) -> Option<V> {
        let mut current = self.arr[position].as_ref().unwrap();
        if current.key == key {
            return Some(current.value);
        }

        while let Some(node) = current.next.as_ref() {
            if node.key == key {
                return Some(node.value);
            }

            current = node;
        }

        None
    }

    fn check_item_in_list_and_remove(&mut self, key: T, position: usize) -> Option<V> {
        let mut current = self.arr[position].as_mut().unwrap();
        if current.key == key {
            let return_val = current.value;

            // check if there is a next val
            // if there is next, update array to point to this val
            if let Some(node) = current.next.to_owned() {
                self.arr[position] = Some(*node);
            } else {
                self.arr[position] = None
            }

            // return the value the node held
            self.curr_size -= 1;
            return Some(return_val);
        }

        // iterate through until key found
        while current.next.is_some() {
            let next = current.next.as_mut().unwrap();
            if next.key == key {
                let return_val = next.value;

                // check if there is a next val
                // if there is next, update array to point to this val
                if let Some(next_next) = next.next.to_owned() {
                    current.next = Some(Box::new(*next_next));
                } else {
                    current.next = None
                }

                // return the value the node held
                self.curr_size -= 1;
                return Some(return_val);
            }

            // set current equal to the next
            current = current.next.as_mut().unwrap();
        }

        None
    }
}

impl<T, V> KeyValue<T, V> {
    pub fn new(key: T, value: V) -> KeyValue<T, V> {
        KeyValue {
            key,
            value,
            next: None,
        }
    }
}

fn hash_key<T: Hash>(key: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);

    hasher.finish()
}
