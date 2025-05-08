use chrono::Utc;
use memchr::memchr;

use crate::{
    codec::encoder::{encode_array, encode_bulk_string},
    store::store::{map_get, map_insert, GLOBAL_HASHMAP, GLOBAL_HASHMAP_CONFIG},
    utils::utils::{generate_random_string, get_key_value_pair_string},
    BufSplit, RESPError, RESPTypes, RedisResult, Value,
};

pub fn parse_and_decode(buf: &[u8]) -> Option<Vec<u8>> {
    if buf.is_empty() {
        return None;
    }

    match parse_redis_protocol(buf, 0) {
        Ok(Some((pos, value))) => {
            return Some(encode(buf, value));
        }
        _ => return None,
    }
}

fn parse_redis_protocol(buf: &[u8], pos: usize) -> RedisResult {
    match buf[pos] {
        // b'+' => simple_string(buf, pos + 1),
        // b'-' => error(buf, pos + 1),
        // b'$' => bulk_string(buf, pos + 1),
        // b':' => resp_int(buf, pos + 1),
        b'*' => array(buf, pos + 1),
        _ => Err(RESPError::UnknownStartingByte),
    }
}

fn simple_string(buf: &[u8], pos: usize) -> RedisResult {
    println!("Simple String came");
    Ok(word(buf, pos).map(|(pos, word)| (pos, RESPTypes::String(word))))
}

fn word(buf: &[u8], pos: usize) -> Option<(usize, BufSplit)> {
    if buf.len() <= pos {
        return None;
    }

    memchr(b'\r', &buf[pos..]).and_then(|end| {
        if end + 1 < buf.len() {
            Some((pos + end + 2, BufSplit(pos, pos + end)))
        } else {
            None
        }
    })
}

fn array(buf: &[u8], mut pos: usize) -> RedisResult {
    let size_string = word(buf, pos).unwrap();
    let mut array_size = String::from_utf8_lossy(&buf[size_string.1 .0..size_string.1 .1])
        .into_owned()
        .to_string()
        .parse::<u8>()
        .unwrap();
    println!("array size is: {}", array_size);
    pos = pos + size_string.0;
    array_size = array_size * 2;
    let mut results: Vec<String> = Vec::new();
    for i in 0..array_size {
        let item_str = memchr(b'\r', &buf[pos..])
            .and_then(|end| {
                if end + 1 < buf.len() {
                    Some((pos + end + 2, BufSplit(pos, pos + end)))
                } else {
                    None
                }
            })
            .unwrap();
        pos = item_str.0;

        if i % 2 != 0 {
            results.push(
                String::from_utf8_lossy(&buf[item_str.1 .0..item_str.1 .1])
                    .into_owned()
                    .to_string(),
            );
        }
    }

    if results.len() > 0 {
        return Ok(Some((results.len(), RESPTypes::Array(results))));
    }

    Err(RESPError::IntParseFailure)
}

fn encode(buf: &[u8], value: RESPTypes) -> Vec<u8> {
    match value {
        RESPTypes::Array(v) => {
            let mut ans: Vec<u8> = Vec::new();
            println!("{}",v[0]);
            match v[0].as_str() {
                "ECHO" => {
                    let length = v[1].len(); // Get the length of the string
                    let length_str = length.to_string(); // Convert the length to a string
                    let length_bytes = length_str.as_bytes();
                    ans.extend_from_slice(b"$");
                    ans.extend_from_slice(length_bytes);
                    ans.extend_from_slice(b"\r\n");
                    ans.extend_from_slice(v[1].as_bytes());
                    ans.extend_from_slice(b"\r\n");
                    return ans;
                }
                "PING" => {
                    ans.extend_from_slice(b"+PONG\r\n");
                    return ans;
                }
                "SET" => {
                    let mut val = Value {
                        value: v[2].clone(),
                        expiry: false,
                        expires_at: 0,
                    };
                    // println!("{}", v[4].clone().parse::<i64>().unwrap());
                    if v.len() > 3 {
                        val.expires_at = Utc::now().timestamp_millis() as u64
                            + v[4].clone().parse::<u64>().unwrap();
                        val.expiry = true;
                    }
                    map_insert(v[1].clone(), val);
                    //GLOBAL_HASHMAP[v.get_mut(0)] = v[1].clone();
                    ans.extend_from_slice(b"+OK\r\n");
                    return ans;
                }
                "GET" => {
                    println!("coming bere");
                    let key = v[1].clone();
                    let val = map_get(key).unwrap();
                    println!("{:?}", val);
                    if val.expiry && Utc::now().timestamp_millis() - val.expires_at as i64 > 0 {
                        ans.extend_from_slice(b"$-1\r\n");
                        return ans;
                    }
                    let length = val.value.len(); // Get the length of the string
                    let length_str = length.to_string(); // Convert the length to a string
                    let length_bytes = length_str.as_bytes();
                    ans.extend_from_slice(b"$");
                    ans.extend_from_slice(length_bytes);
                    ans.extend_from_slice(b"\r\n");
                    ans.extend_from_slice(val.value.as_bytes());
                    ans.extend_from_slice(b"\r\n");
                    return ans;
                }
                "CONFIG" => match v[1].as_str() {
                    "GET" => {
                        let value: String;
                        if let Ok(hashmap) = GLOBAL_HASHMAP_CONFIG.lock() {
                            value = hashmap.get(&v[2]).cloned().unwrap();
                        } else {
                            return ans;
                        }
                        let mut args = Vec::new();
                        args.push(v[2].clone());
                        args.push(value);
                        return encode_array(args);
                    }
                    _ => todo!(),
                },
                "KEYS" => {
                    match v[1].as_str() {
                        "*" => {
                            let value: String;
                            let mut args: Vec<String> = Vec::new();
                            if let Ok(hashmap) = GLOBAL_HASHMAP.lock() {
                                for (k, v) in hashmap.iter() {
                                    args.push(k.to_string());
                                }
                            } else {
                                return ans;
                            }
                            return encode_array(args);
                        }
                        _ => todo!(),
                    }
                }
                "INFO" => match v[1].as_str() {
                    "replication" => {
                        let mut bulk_string_array: Vec<String> = Vec::new();
                        let mut role: String = String::from("role:master");
                        if let Ok(hashmap) = GLOBAL_HASHMAP_CONFIG.lock() {
                            if hashmap.contains_key("master_port") {
                                role = String::from("role:slave");
                            }
                        }
                        bulk_string_array.push(role.clone());
                        if role.contains("master") {
                            let master_replid = generate_random_string();
                            let master_repl_offset = 0;
                            bulk_string_array.push(get_key_value_pair_string(
                                String::from("master_replid"),
                                master_replid,
                                ':',
                            ));
                            bulk_string_array.push(get_key_value_pair_string(
                                String::from("master_repl_offset"),
                                master_repl_offset.to_string(),
                                ':',
                            ));
                        }
                        return encode_bulk_string(bulk_string_array);
                    }
                    _ => {
                        todo!()
                    }
                },
                _ => todo!(),
            }
        }
        _ => todo!(),
    }
}
