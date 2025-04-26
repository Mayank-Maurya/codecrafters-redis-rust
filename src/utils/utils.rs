use std::{collections::HashMap, sync::{LazyLock, Mutex}};
use rand::{distributions::Alphanumeric, Rng};

pub fn map_get<V: Clone>(hash_map: &LazyLock<Mutex<HashMap<String, V>>>, key: String) -> Option<V> {
    if let Ok(hashmap) = hash_map.lock() {
        return hashmap.get(&key).cloned();
    } else {
        println!("Failed to acquire lock on GLOBAL_HASHMAP_CONFIG");
    }
    None
}

pub fn generate_random_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect()
}

pub fn get_key_value_pair_string(key: String, value: String, delimeter: char) -> String {
    let mut result: String = key;
    result.push(delimeter);
    result.push_str(&value);
    result
}