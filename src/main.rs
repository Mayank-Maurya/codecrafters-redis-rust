#![allow(unused_imports)]
use std::collections::HashMap;
use std::env;
use std::ops::Add;
use std::rc::Rc;
use std::io::{self, BufRead, Error, Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use bytes::{Bytes, BytesMut};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio_util::codec::{Decoder, Encoder};
use memchr::memchr;
use std::sync::{LazyLock, Mutex};

static GLOBAL_HASHMAP: LazyLock<Mutex<HashMap<String, Value>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static GLOBAL_HASHMAP_CONFIG: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
struct BufSplit(usize, usize);

#[derive(Debug, Clone)]
struct Value {
    value: String,
    px: u128,
    updated_time: u128,
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
            _ => {
            }
        }
    }

    let listener = TcpListener::bind("127.0.0.1:6380").await?;

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

    match parse(buf,0) {
        Ok(Some((pos, value))) => {
            return Some(encode(buf, value));
        },
        _ => return None,
    }
}

fn parse(buf: &[u8], pos: usize) -> RedisResult {
    match buf[pos] {
        // b'+' => simple_string(buf, pos + 1),
        // b'-' => error(buf, pos + 1),
        // b'$' => bulk_string(buf, pos + 1),
        // b':' => resp_int(buf, pos + 1),
        b'*' => array(buf, pos + 1),
        _ => Err(RESPError::UnknownStartingByte),
    }
}

#[allow(clippy::unnecessary_wraps)]
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
                    let mut val = Value{ value: v[2].clone(), px: 0, updated_time: current_timestamp_millis(),};
                    if v.len() > 3 {
                        val.px = v[4].clone().parse::<u128>().unwrap();
                    }
                    map_insert(v[1].clone(), val);
                    //GLOBAL_HASHMAP[v.get_mut(0)] = v[1].clone();
                    ans.extend_from_slice(b"+OK\r\n");
                    return ans;
                },
                "GET" => {
                    let key = v[1].clone();
                    let val = map_get(key).unwrap();
                    println!("{:?}", val);
                    let cur_time = current_timestamp_millis();
                    if cur_time - val.updated_time > val.px && val.px > 0 {
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
                            let mut value;
                            if let Ok(hashmap) = GLOBAL_HASHMAP_CONFIG.lock() {
                                value = hashmap.get(&v[2]).cloned().unwrap();
                            } else {
                                return ans;
                            }
                            let length = &value; // Get the length of the string
                            let length_str = length.to_string(); // Convert the length to a string
                            let length_bytes = length_str.as_bytes(); 
                            ans.extend_from_slice(b"$");
                            ans.extend_from_slice(length_bytes);
                            ans.extend_from_slice(b"\r\n");
                            ans.extend_from_slice(&value.as_bytes());
                            ans.extend_from_slice(b"\r\n");
                            return ans;
                        },
                        _ => todo!()
                    }
                }
                _ => todo!(),
            }
        },
        _ => todo!(),
    }
}

fn map_insert(key: String, value: Value) {
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

fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}
