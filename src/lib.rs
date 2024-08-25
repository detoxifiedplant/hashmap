const DEFAULT_MAX_SIZE: u64 = 256;

pub struct HashMap<T, V> {
    current_size: usize,
    arr: [Option<KeyValue<T,V>>; DEFAULT_MAX_SIZE as usize]
}

pub struct KeyValue<T,V> {
    key: T,
    value: V,
    next: Option<Box<KeyValue<T, V>>>
}
