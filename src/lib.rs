#![allow(unused_imports)]
use rand::distributions::{Distribution, Uniform};
pub mod raw;
use raw::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut map: HashMap<u32, u32> = HashMap::new();
        // let mut map: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();

        // let key_gen = Uniform::from(0..1_000_000);
        // let op_gen = Uniform::from(0..4);
        // let mut rng = rand::thread_rng();

        for i in 0..3 {
            for j in 0..1_000_000 {
                // let val = key_gen.sample(&mut rng);
                // let key = val;
                let (key, val) = (j, j);
                match i {
                    0 => {
                        map.insert(key, val);
                        // assert_eq!(&val, map.get(&key).unwrap())
                    }
                    1 => {
                        map.get_mut(&key).map(|x| {
                            *x *= 10;
                            x
                        });
                    }
                    2 => assert_eq!(val * 10, *map.get(&key).unwrap()),
                    // 3 => _ = map.remove(&key),
                    _ => (),
                }
            }
        }
        for j in 0..1_000_000 {
            let (key, val) = (j, j);
            assert_eq!(val * 10, *map.get(&key).unwrap());
        }
    }

    #[test]
    fn test_insert_and_get() {
        let mut map = HashMap::new();

        // Insert a key-value pair
        map.insert(1, "value1");

        // Check that the value is correctly inserted
        assert_eq!(map.get(&1), Some(&"value1"));

        // Insert a new key-value pair
        map.insert(2, "value2");

        // Check that both keys are present
        assert_eq!(map.get(&1), Some(&"value1"));
        assert_eq!(map.get(&2), Some(&"value2"));
    }

    #[test]
    fn test_get_mut() {
        let mut map = HashMap::new();

        // Insert a key-value pair
        map.insert(1, "value1");

        // Mutate the value via get_mut
        if let Some(v) = map.get_mut(&1) {
            *v = "value1_modified";
        }

        // Check that the value was modified
        assert_eq!(map.get(&1), Some(&"value1_modified"));
    }

    #[test]
    fn test_remove() {
        let mut map = HashMap::new();

        // Insert a key-value pair
        map.insert(1, "value1");

        // Remove the key
        let removed_value = map.remove(&1);

        // Check that the correct value was removed
        assert_eq!(removed_value, Some("value1"));

        // Ensure that the key is no longer present in the map
        assert_eq!(map.get(&1), None);
    }

    #[test]
    fn test_insert_overwrite() {
        let mut map = HashMap::new();

        // Insert a key-value pair
        map.insert(1, "value1");

        // Overwrite the value for the same key
        let old_value = map.insert(1, "value1_new");

        // Check that the old value is returned on insert
        assert_eq!(old_value, Some("value1"));

        // Ensure the value has been updated
        assert_eq!(map.get(&1), Some(&"value1_new"));
    }

    #[test]
    fn test_insert_and_remove_non_existent_key() {
        let mut map = HashMap::new();

        // Try to remove a non-existent key
        let removed_value = map.remove(&2);

        // Ensure the return is None for a non-existent key
        assert_eq!(removed_value, None);

        // Insert a key and then remove it
        map.insert(2, "value2");
        let removed_value = map.remove(&2);

        // Ensure the correct value is returned
        assert_eq!(removed_value, Some("value2"));

        // Ensure the key is no longer present
        assert_eq!(map.get(&2), None);
    }
}
