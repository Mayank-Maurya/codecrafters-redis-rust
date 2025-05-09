use std::sync::atomic::Ordering;

use chrono::Utc;
use memchr::memchr;

use crate::{
    codec::encoder::{encode_array, encode_bulk_string}, server::handshake::{handshake_2, handshake_3, send_command}, store::store::{map_config_get, map_get, map_insert, GLOBAL_HASHMAP, GLOBAL_HASHMAP_CONFIG, IS_OK}, utils::utils::{generate_random_string, get_key_value_pair_string}, BufSplit, RESPError, RESPTypes, RedisResult, Value
};

pub fn parse_and_decode_handshake(buf: &[u8], flag: bool) -> Option<Vec<u8>> {
    if buf.is_empty() {
        return None;
    }

    match parse_redis_protocol(buf, 0) {
        Ok(Some((pos, value))) => {
            return Some(encode(buf, value, flag));
        }
        _ => return None,
    }
}

fn parse_redis_protocol(buf: &[u8], pos: usize) -> RedisResult {
    match buf[pos] {
        b'+' => simple_string(buf, pos + 1),
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

fn encode(buf: &[u8], value: RESPTypes, mut flag: bool) -> Vec<u8> {
    match value {
        RESPTypes::Array(v) => {
            let mut ans: Vec<u8> = Vec::new();
            println!("{}",v[0]);
            match v[0].as_str() {
                "PONG" => {
                    let slave_port = map_config_get(String::from("port")).map_or("6379".to_string(), |s| s.to_string());
                    return handshake_2(slave_port);
                }
                "REPLCONF" => match v[1].as_str() {
                    "listening-port" => {
                        let listening_port = v[2].as_str();
                        println!("came here REPLCONF");
                        ans.extend_from_slice(b"+OK\r\n");
                        return ans; 
                    },
                    "capa" => {
                        match v[2].as_str() {
                            "psync2" => {
                                println!("came here psync2");
                                ans.extend_from_slice(b"+OK\r\n");
                                return ans; 
                            },
                            _ => {
                                todo!()
                            }
                        }
                    },
                    _ => todo!(),
                }
                _ => todo!(),
            }
        },
        RESPTypes::String(s) => {
            let ans: Vec<u8> = buf[s.0..s.1].to_vec();
            println!("{:?}", String::from_utf8_lossy(&ans));
            let resp_str = String::from_utf8_lossy(&ans).to_string();
            match resp_str.as_str() {
                "PONG" => {
                    let slave_port = map_config_get(String::from("port")).map_or("6379".to_string(), |s| s.to_string());
                    return handshake_2(slave_port);
                },
                "OK" => {
                    println!("{}", flag);
                    if IS_OK.load(Ordering::SeqCst) == false {
                        IS_OK.store(true, Ordering::SeqCst);
                        return handshake_3();
                    }
                    println!("doing nothing");
                },
                _ => {

                }
            }
            return ans;
        }
        _ => todo!(),
    }
}
