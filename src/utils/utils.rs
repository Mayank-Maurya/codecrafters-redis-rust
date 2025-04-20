use std::{collections::HashMap, sync::{LazyLock, Mutex}};

pub fn map_get<V: Clone>(hash_map: &LazyLock<Mutex<HashMap<String, V>>>, key: String) -> Option<V> {
    if let Ok(hashmap) = hash_map.lock() {
        return hashmap.get(&key).cloned();
    } else {
        println!("Failed to acquire lock on GLOBAL_HASHMAP_CONFIG");
    }
    None
}