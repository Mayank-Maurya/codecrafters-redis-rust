#![allow(unused_imports)]
// custom modules
mod parsers;
pub mod utils;
use chrono::{DateTime, TimeDelta, Utc};
use parsers::args_parse::parse as args_parse;
use parsers::rdb_file_parser::parse as rdb_file_parse;
use utils::utils::map_get as map_get_generic;

// imports
use std::collections::HashMap;
use std::{env, vec};
use std::fs::File;
use std::ops::Add;
use std::rc::Rc;
use std::io::{self, BufRead, Error, Read, Write};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use bytes::{buf, Bytes, BytesMut};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio_util::codec::{Decoder, Encoder};
use memchr::memchr;
use std::sync::{LazyLock, Mutex};

// global hashmaps
pub static GLOBAL_HASHMAP: LazyLock<Mutex<HashMap<String, Value>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
pub static GLOBAL_HASHMAP_CONFIG: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

// some definations
#[derive(Debug)]
struct BufSplit(usize, usize);

#[derive(Debug, Clone)]
pub struct Value {
    value: String,
    expiry: bool,
    expires_at: u64,
}

#[derive(Debug)]
pub enum RESPTypes {
    String(BufSplit),
    Error(BufSplit),
    Int(i64),
    Array(Vec<String>),
    NullArray,
    NullBulkString,
}

#[derive(Debug)]
pub enum RESPError {
    UnexpectedEnd,
    UnknownStartingByte,
    IOError(std::io::Error),
    IntParseFailure,
    BadBulkStringSize(i64),
    BadArraySize(i64),
}

impl From<std::io::Error> for RESPError {
    fn from(e: std::io::Error) -> RESPError {
        RESPError::IOError(e)
    }
}

type RedisResult = Result<Option<(usize, RESPTypes)>, RESPError>;

#[tokio::main]
async fn main() -> io::Result<()> {
    //parsing arguments
    args_parse();

    // parsing rdb file
    rdb_file_parse();

    // setup connection
    let mut port = "6379".to_string();
    if let Ok(hashmap) = GLOBAL_HASHMAP_CONFIG.lock() {
        port = hashmap.get("port").map_or("6379".to_string(), |s| s.to_string());
    }
    let listener = TcpListener::bind("127.0.0.1:".to_owned() + port.as_str()).await?;
    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((mut stream, _)) => {
                println!("Client Connected");

                tokio::spawn(async move {
                    let mut buf: Vec<u8> = vec![0;512];
            
                    loop {
                        match stream.read(&mut buf).await {
                            Ok(0) => {
                                // println!("nothing came");
                            },
                            Ok(n) => {
                                // parse the string and get the result
                                if let Some(result) = parse_and_decode(&buf[..n]) {
                                    if let Err(e) = stream.write_all(&result).await {
                                        eprintln!("Failed to write to stream: {}", e);
                                    }
                                } else {
                                    println!("Failed to parse the message.");
                                }
                            },
                            Err(e) => {
                                eprintln!("failed to read from stream; err = {:?}", e);
                                return;
                            }
                        };
                    }
                });
            },
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

}

fn parse_and_decode(buf: &[u8]) -> Option<Vec<u8>> {
    if buf.is_empty() {
        return None;
    }

    match parse_redis_protocol(buf,0) {
        Ok(Some((pos, value))) => {
            return Some(encode(buf, value));
        },
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
    let mut array_size = String::from_utf8_lossy(&buf[size_string.1.0..size_string.1.1]).into_owned().to_string().parse::<u8>().unwrap();
    println!("array size is: {}",array_size);
    pos = pos + size_string.0;
    array_size = array_size * 2;
    let mut results: Vec<String> = Vec::new();
    for i in 0..array_size {
        let item_str = memchr(b'\r', &buf[pos..]).and_then(|end| {
            if end + 1 < buf.len() {
                Some((pos + end + 2, BufSplit(pos, pos + end)))
            } else {
                None
            }
        }).unwrap();
        pos = item_str.0;

        if i%2!=0 {
            results.push(String::from_utf8_lossy(&buf[item_str.1.0..item_str.1.1]).into_owned().to_string());
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
            // println!("{}",v[0]);
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
                },
                "PING" => {
                    ans.extend_from_slice(b"+PONG\r\n");
                    return ans;
                },
                "SET" => {
                    let mut val = Value { value: v[2].clone(), expiry: false, expires_at: 0 };
                    // println!("{}", v[4].clone().parse::<i64>().unwrap());
                    if v.len() > 3 {
                        val.expires_at = Utc::now().timestamp_millis() as u64 + v[4].clone().parse::<u64>().unwrap();
                        val.expiry = true;
                    }
                    map_insert(v[1].clone(), val);
                    //GLOBAL_HASHMAP[v.get_mut(0)] = v[1].clone();
                    ans.extend_from_slice(b"+OK\r\n");
                    return ans;
                },
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
                },
                "CONFIG" => {
                    match v[1].as_str() {
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
                        },
                        _ => todo!()
                    }
                },
                "KEYS" => {
                    match v[1].as_str() {
                        "*" => {
                            let value: String;
                            let mut args: Vec<String> = Vec::new();
                            if let Ok(hashmap) = GLOBAL_HASHMAP.lock() {
                                for (k, v) in hashmap.iter() {
                                    args.push(k.to_string());
                                    // args.push(v.value.clone());
                                }
                            } else {
                                return ans;
                            }
                            return encode_array(args);
                        },
                        _ => todo!()
                    }
                },
                "INFO" => {
                    match v[1].as_str() {
                        "replication" => {
                            ans.extend_from_slice(b"$");
                            ans.extend_from_slice(b"11");
                            ans.extend_from_slice(b"\r\n");
                            ans.extend_from_slice(b"role:master");
                            ans.extend_from_slice(b"\r\n");
                            return ans;
                        },
                        _ => {
                            todo!()
                        }
                    }
                }
                _ => todo!(),
            }
        },
        _ => todo!(),
    }
}

fn encode_array(array: Vec<String>) -> Vec<u8> {
    let mut ans = Vec::new();
    let mut length = array.len(); // Get the length of the string
    let mut length_str = length.to_string(); // Convert the length to a string
    let mut length_bytes = length_str.as_bytes(); 
    ans.extend_from_slice(b"*");
    ans.extend_from_slice(length_bytes);
    ans.extend_from_slice(b"\r\n");
    for i in array {
        length = i.len();
        length_str = length.to_string();
        length_bytes = length_str.as_bytes();
        ans.extend_from_slice(b"$");
        ans.extend_from_slice(length_bytes);
        ans.extend_from_slice(b"\r\n");
        ans.extend_from_slice(i.as_bytes());
        ans.extend_from_slice(b"\r\n");
    }
   // println!("{:?}", ans);
    return ans;
}

pub fn map_insert(key: String, value: Value) {
    if let Ok(mut hashmap) = GLOBAL_HASHMAP.lock() {
        hashmap.insert(key, value);
    } else {
        eprintln!("Failed to acquire lock on GLOBAL_HASHMAP");
    }
}

fn map_get(key: String) -> Option<Value> {
    if let Ok(hashmap) = GLOBAL_HASHMAP.lock() {
        return hashmap.get(&key).cloned();
    }
    None
}

pub fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}
