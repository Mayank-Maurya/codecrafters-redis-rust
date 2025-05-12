use core::str;
use std::{collections::HashMap, sync::{LazyLock, Mutex}, time::{SystemTime, UNIX_EPOCH}};
use base64::{engine::general_purpose, Engine};
use rand::{distributions::Alphanumeric, Rng};

use crate::codec::encoder::encode_bulk_string_rdb_file;

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

pub fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

pub fn empty_rdb_file() -> Vec<u8> {
    let hex = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";
    match base64_to_string(hex) {
        Ok(decoded) => {
            return decoded;
        },
        Err(e) => {
            println!("Error: {}", e);
        },
    }

    return vec![];
}

pub fn base64_to_string(encoded: &str) -> Result<Vec<u8>, String> {
    match general_purpose::STANDARD.decode(encoded) {
        Ok(bytes) => {
            let mut ans: Vec<u8> = Vec::new();
            // convert string to bulk String
            ans.extend_from_slice(b"$");
            ans.extend_from_slice(bytes.len().to_string().as_bytes());
            ans.extend_from_slice(b"\r\n");
            ans.extend_from_slice(bytes.as_slice());
            return Ok(ans);
        },
        Err(_) => Err("Invalid Base64 string".to_string()),
    }
}