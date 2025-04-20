use std::{fs::File, io::Read};
use crate::{utils::utils::map_get, GLOBAL_HASHMAP_CONFIG};

pub fn parse() {
    let file_path = map_get(&GLOBAL_HASHMAP_CONFIG, String::from("value")).unwrap();
    match get_rdb_file(file_path) {
        Ok(file_buffer) => {
            parse_rdb_file(file_buffer);
        },
        Err(e) => {
            println!("rdb_file_parsing_error: {:?}", e);
        }
    }
}

fn get_rdb_file(file_path: String) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer: Vec<u8>  = Vec::new();
    let file_red = file.read_to_end(&mut buffer);
    println!("RDB File contents: {:?}", buffer);
    Ok(buffer)
}

fn parse_rdb_file(file_buffer: Vec<u8>) {
    println!("file_buffer: {:?}", file_buffer);
}