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
        let key_gen = Uniform::from(0..100000);
        let op_gen = Uniform::from(0..4);
        let mut rng = rand::thread_rng();

        for _ in 0..1_000_000 {
            let val = key_gen.sample(&mut rng);
            let key = val;
            match op_gen.sample(&mut rng) {
                0 => {
                    map.insert(key, val);
                    assert_eq!(&val, map.get(&key).unwrap())
                    }
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
