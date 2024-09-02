use std::collections::HashSet;

use super::HashMap;
use rstest::{fixture, rstest};
use uuid::Uuid;

#[fixture]
fn multiple_keys() -> [String; 5] {
    [
        "hello".to_string(),
        "world".to_string(),
        "rupert".to_string(),
        "here".to_string(),
        "test".to_string(),
    ]
}

#[rstest]
fn test_can_add_item() {
    let key = "hello".to_string();
    let value: i32 = 1;

    let mut my_hash: HashMap<String, i32> = HashMap::new();
    my_hash.put(key, value);
}

#[rstest]
fn test_can_add_multiple_items(multiple_keys: [String; 5]) {
    let mut my_hash: HashMap<String, i32> = HashMap::new();

    for (i, key) in multiple_keys.iter().enumerate() {
        my_hash.put(key.clone(), i as i32);
    }
}

#[rstest]
fn test_can_get_item() {
    let key = "hello".to_string();
    let value: i32 = 1;

    let mut my_hash: HashMap<String, i32> = HashMap::new();
    my_hash.put(key.clone(), value);

    let result = my_hash.get(key).unwrap();

    assert_eq!(result, value)
}

#[rstest]
fn test_can_update_item() {
    let key = "hello".to_string();
    let value: i32 = 1;

    let mut my_hash: HashMap<String, i32> = HashMap::new();
    my_hash.put(key.clone(), value);

    let mut result = my_hash.get(key.clone()).unwrap();

    assert_eq!(result, value);

    my_hash.put(key.clone(), 2);
    result = my_hash.get(key).unwrap();
    assert_eq!(result, 2);
}

#[rstest]
fn test_can_get_item_from_multiple(multiple_keys: [String; 5]) {
    let mut my_hash: HashMap<String, i32> = HashMap::new();

    for (i, key) in multiple_keys.iter().enumerate() {
        my_hash.put(key.clone(), i as i32);
    }

    for (i, key) in multiple_keys.iter().enumerate() {
        let val = my_hash.get(key.to_string()).unwrap();
        my_hash.put(key.clone(), i as i32);
        assert_eq!(val, i as i32);
    }
}

fn create_random_key_array(size: u64) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for _ in 0..size {
        let key_value = Uuid::new_v4();
        keys.push(key_value.to_string());
    }

    keys
}

const DEFAULT_MAX_SIZE: u64 = 256;
#[rstest]
fn test_can_still_put_values_from_greater_than_max_size() {
    let keys = create_random_key_array(DEFAULT_MAX_SIZE + 10);

    let mut my_hash = HashMap::new();
    for (i, key) in keys.iter().enumerate() {
        my_hash.put(key.clone(), i as i32);
    }
}

#[rstest]
fn test_can_still_get_values_from_greater_than_max_size() {
    let unique: HashSet<String> =
        HashSet::from_iter(create_random_key_array(DEFAULT_MAX_SIZE + 100));

    let keys: Vec<String> = unique.into_iter().collect();

    let mut my_hash: HashMap<String, i32> = HashMap::new();
    for (i, key) in keys.iter().enumerate() {
        my_hash.put(key.clone(), i as i32);
    }
    for (i, key) in keys.iter().enumerate() {
        let result = my_hash.get(key.to_string()).unwrap();
        assert_eq!(result, i as i32)
    }
}

#[rstest]
fn test_can_remove_item() {
    let key = "hello".to_string();
    let value: i32 = 1;

    let mut my_hash: HashMap<String, i32> = HashMap::new();
    my_hash.put(key.clone(), value);

    let mut result = my_hash.get(key.clone()).unwrap();

    assert_eq!(result, value);

    result = my_hash.remove(key.clone()).unwrap();

    assert_eq!(result, value);
    assert_eq!(my_hash.get(key), None)
}

#[rstest]
fn test_can_add_more_than_max_vals_and_remove_all_vals() {
    let unique: HashSet<String> =
        HashSet::from_iter(create_random_key_array(DEFAULT_MAX_SIZE + 100));

    let keys: Vec<String> = unique.into_iter().collect();

    let mut my_hash: HashMap<String, i32> = HashMap::new();
    for (i, key) in keys.iter().enumerate() {
        my_hash.put(key.clone(), i as i32);
    }
    for (i, key) in keys.iter().enumerate() {
        let result = my_hash.remove(key.to_string()).unwrap();
        assert_eq!(result, i as i32)
    }
}
