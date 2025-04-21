use std::{fs::File, io::{BufReader, Read, Seek}, os::unix::fs::FileExt, path::{Path, PathBuf}, vec};
use crate::{map_insert, utils::utils::map_get, Value, GLOBAL_HASHMAP_CONFIG};

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
    match parse_rdb_file(full_path.to_string_lossy().into_owned()) {
        Ok(result) => {
        },
        Err(e) => {
            println!("rdb_file_parsing_error: {:?}", e);
        }
    }
}

fn parse_rdb_file(file_path: String) -> Result<(), std::io::Error> {
    let file = BufReader::new(File::open(file_path)?);
    let bytes_vec: Vec<u8> = file.bytes().map(|f| f.unwrap()).collect();
    let mut idx = 0;
    let size = bytes_vec.len();
    while idx < size {
        match bytes_vec.get(idx) {
            Some(b) => {
                let item: String = format!("{:02x}",b);
                match item.as_str() {
                    "fe" => {
                        // db section found
                        println!("db section starts");
                        parse_db_section(idx + 1, size, &bytes_vec);
                        break;
                    },
                    _ => {

                    }
                }
            },
            None => {
                break;
            }
        }
        idx = idx + 1;
    }

    Ok(())
}

fn parse_db_section(mut idx: usize, size: usize, bytes_vec: &Vec<u8>) {
    let got_db_idx = false;
    while idx < size {
        // get the index of the db
        let db_idx = get_ele_normal(idx, &bytes_vec);
        idx = idx + 2;

        // get hash-table size
        let mut hash_table_size: u8 = get_ele_normal(idx, &bytes_vec);
        idx = idx + 1;

        println!("keys size: {}", hash_table_size);

        // get expiry Hash-table elements count
        let hash_table_expiry_count = get_ele_normal(idx, &bytes_vec);
        idx = idx + 1;

        println!("expiry keys size: {}", hash_table_expiry_count);

        // loop hash_table_size size and get elements from file
        while hash_table_size > 0 {
            // check which type of kv pair is this string or expiry
            match bytes_vec.get(idx) {
                Some(b) => {
                    let str = format!("{:02x}",b);
                    match str.as_str() {
                        "fc" => {
                            // read next 8 bytes
                            let x: [u8; 8] = bytes_vec[idx+1..idx+9].try_into().unwrap();
                            let timestamp = u64::from_le_bytes(x);
                            println!("{}", timestamp);
                            idx = idx + 9;

                            let KV = get_key_value_pair(idx, &bytes_vec);
                            let val = Value{ value: KV.1.clone(), px: 0, updated_time: timestamp as u128};
                            map_insert(KV.0, val);
                        },
                        "fd" => {
                            // read next 4 bytes
                            let x: [u8; 8] = bytes_vec[idx+1..idx+5].try_into().unwrap();
                            let timestamp = u64::from_le_bytes(x);
                            println!("{}", timestamp);
                            idx = idx + 5;

                            let KV = get_key_value_pair(idx, &bytes_vec);
                            let val = Value{ value: KV.1.clone(), px: 0, updated_time: timestamp as u128};
                            map_insert(KV.0, val);
                        },
                        _ => {
                            // parse string encoded KV
                            println!("normal KV found {}", str);

                            let KV = get_key_value_pair(idx, &bytes_vec);
                            let val = Value{ value: KV.1.clone(), px: 0, updated_time: 0};
                            map_insert(KV.0, val);
                        }
                    }
                },
                None => {
                    // do nothing
                },
            }

            hash_table_size = hash_table_size - 1;
        }
        
        match get_ele(idx, &bytes_vec).as_str() {
            "fe" => {
                idx = idx + 1;
            },
            _ => {
                break;
            }
        }
    }
}

fn get_ele(idx: usize, bytes_vec: &Vec<u8>) -> String {
    match bytes_vec.get(idx) {
        Some(b) => {
            // println!("{:?}", b.to_le());
            format!("{:02x}",b)
        },
        None => String::new(),
    }
}

fn get_ele_normal(idx: usize, bytes_vec: &Vec<u8>) -> u8 {
    match bytes_vec.get(idx) {
        Some(b) => b.to_le(),
        None => 0
    }
}

fn get_key_value_pair(mut idx: usize, bytes_vec: &Vec<u8>) -> (String, String) {
    // parse string encoded KV
    
    // skip flag variable
    idx = idx + 1;

    let key_size: u8 = get_ele_normal(idx, &bytes_vec);
    println!("item key Size: {}", key_size);

    let key_end_idx = idx + key_size as usize + 1;
    let key_bytes = &bytes_vec[idx..key_end_idx];
    let key = match String::from_utf8(key_bytes.to_vec()) {
        Ok(s) => s,
        Err(_) => String::from("<Invalid UTF-8>"),
    };
    println!("Key: {}", key);
    idx = key_end_idx;

    let value_size: u8 = get_ele_normal(idx, &bytes_vec);
    println!("item Value Size: {}", value_size);
    let value_end_idx = idx + value_size as usize + 1;
    let value_bytes = &bytes_vec[idx..value_end_idx];
    let value = match String::from_utf8(value_bytes.to_vec()) {
        Ok(s) => s,
        Err(_) => String::from("<Invalid UTF-8>"),
    };
    println!("Key: {}", &value);
    idx = value_end_idx;
    return (key, value);
    //map_insert(key, val);
}
