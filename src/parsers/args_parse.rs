use crate::{store::store::map_config_insert, GLOBAL_HASHMAP_CONFIG};
use std::env;

pub fn parse() {
    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        match args[i].as_str() {
            "--dir" => {
                // println!("{}", args[i+1]);
                map_config_insert("dir".to_string(), args[i + 1].clone());
            }
            "--dbfilename" => {
                map_config_insert("dbfilename".to_string(), args[i + 1].clone());
            }
            "--port" => {
                map_config_insert("port".to_string(), args[i + 1].clone());
            }
            "--replicaof" => {
                // println!("{}", args[i+1].clone());
                let master_config: Vec<&str> = args[i + 1].split(" ").collect();
                map_config_insert("master_host".to_string(), master_config[0].to_string());
                map_config_insert("master_port".to_string(), master_config[1].to_string());
            }
            _ => {}
        }
    }
}
