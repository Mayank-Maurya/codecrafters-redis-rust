use std::{fs::File, io::Read, path::{Path, PathBuf}};
use crate::{utils::utils::map_get, GLOBAL_HASHMAP_CONFIG};

pub fn parse() {
    // get file dir
    let file_path = match map_get(&GLOBAL_HASHMAP_CONFIG, String::from("dir")) {
        Some(path) => {
            path
        },
        None => {
            println!("could not get file_path");
            return;
        },
    };
    // get file name
    let file_name = match map_get(&GLOBAL_HASHMAP_CONFIG, String::from("dbfilename")) {
        Some(file_name) => {
            file_name
        },
        None => {
            println!("could not get file_name");
            return;
        },
    };
    // get full file path
    let full_path = PathBuf::from(file_path).join(file_name);
    
    // get file
    match get_rdb_file(full_path.to_string_lossy().into_owned()) {
        Ok(file_buffer) => {
            parse_rdb_file(file_buffer);
        },
        Err(e) => {
            println!("rdb_file_parsing_error: {:?}", e);
        }
    }
}

fn get_rdb_file(file_path: String) -> std::io::Result<Vec<u8>> {
    let mut file = match File::open(file_path) {
        Ok(file) => {
            file
        },
        Err(e) => {
            println!("file not found: {:?}", e);
            return Err(e);
        }
    };
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer);
    Ok(buffer)
}

fn parse_rdb_file(file_buffer: Vec<u8>) {
    println!("file_buffer: {:?}", file_buffer);
    let hex_string: String = file_buffer
        .iter()
        .map(|byte| format!("{:02x}", byte)) // Format each byte as a two-digit hexadecimal
        .collect();

    println!("File content as hexadecimal:\n{}", hex_string);

    // Optionally, convert the hexadecimal string back to a regular string (if needed)
    match String::from_utf8(file_buffer) {
        Ok(content) => {
            println!("File content as string:\n{}", content);
        }
        Err(e) => {
            println!("Failed to convert file buffer to string: {:?}", e);
        }
    }
}