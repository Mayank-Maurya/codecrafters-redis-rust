use std::env;
use crate::GLOBAL_HASHMAP_CONFIG;

pub fn parse() {
    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        match args[i].as_str() {
            "--dir" => {
                println!("{}", args[i+1]);
                if let Ok(mut hashmap) = GLOBAL_HASHMAP_CONFIG.lock() {
                    hashmap.insert("dir".to_string(), args[i+1].clone());
                } else {
                    eprintln!("Failed to acquire lock on GLOBAL_HASHMAP_CONFIG");
                }
            },
            "--dbfilename" => {
                if let Ok(mut hashmap) = GLOBAL_HASHMAP_CONFIG.lock() {
                    hashmap.insert("dbfilename".to_string(), args[i+1].clone());
                } else {
                    eprintln!("Failed to acquire lock on GLOBAL_HASHMAP_CONFIG");
                }
            },
            "--port" => {
                if let Ok(mut hashmap) = GLOBAL_HASHMAP_CONFIG.lock() {
                    hashmap.insert("port".to_string(), args[i+1].clone());
                } else {
                    eprintln!("Failed to acquire lock on GLOBAL_HASHMAP_CONFIG");
                }
            },
            "--replicaof" => {
                println!("{}", args[i+1].clone());
                let master_config: Vec<&str> = args[i+1].split(" ").collect();
                if let Ok(mut hashmap) = GLOBAL_HASHMAP_CONFIG.lock() {
                    hashmap.insert("master_host".to_string(), master_config[0].to_string());
                    hashmap.insert("master_port".to_string(), master_config[1].to_string());
                } else {
                    eprintln!("Failed to acquire lock on GLOBAL_HASHMAP_CONFIG");
                }
            }
            _ => {
            }
        }
    }
}