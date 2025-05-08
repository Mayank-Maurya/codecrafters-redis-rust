use std::{collections::HashMap, sync::{LazyLock, Mutex}};
use crate::Value;

// global hashmaps
pub static GLOBAL_HASHMAP: LazyLock<Mutex<HashMap<String, Value>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static GLOBAL_HASHMAP_CONFIG: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn map_insert(key: String, value: Value) {
    if let Ok(mut hashmap) = GLOBAL_HASHMAP.lock() {
        hashmap.insert(key, value);
    } else {
        eprintln!("Failed to acquire lock on GLOBAL_HASHMAP");
    }
}

pub fn map_get(key: String) -> Option<Value> {
    if let Ok(hashmap) = GLOBAL_HASHMAP.lock() {
        return hashmap.get(&key).cloned();
    }
    None
}

pub fn map_config_insert(key: String, value: String) {
    if let Ok(mut hashmap) = GLOBAL_HASHMAP_CONFIG.lock() {
        hashmap.insert(key, value);
    } else {
        eprintln!("Failed to acquire lock on GLOBAL_HASHMAP");
    }
}

pub fn map_config_get(key: String) -> Option<String> {
    if let Ok(hashmap) = GLOBAL_HASHMAP_CONFIG.lock() {
        return hashmap.get(&key).cloned();
    }
    None
}
