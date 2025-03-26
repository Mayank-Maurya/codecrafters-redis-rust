#![allow(unused_imports)]
use std::ops::Add;
use std::rc::Rc;
use std::io::{self, BufRead, Error, Read, Write};
use bytes::{Bytes, BytesMut};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio_util::codec::{Decoder, Encoder};
use memchr::memchr;

#[derive(Debug)]
struct BufSplit(usize, usize);

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

#[derive(Debug)]
pub enum RedisValueRef {
    String(Bytes),
    Error(Bytes),
    Int(i64),
    Array(Vec<RedisValueRef>),
    NullArray,
    NullBulkString,
    ErrorMsg(Vec<u8>),
}

type RedisResult = Result<Option<(usize, RESPTypes)>, RESPError>;

// impl RESPTypes {
//     fn get_value(self, buf: &Bytes) -> RedisValueRef {
//         match self {
//             RESPTypes::String(bfs) => RedisValueRef::String( buf.slice(bfs.0..bfs.1)),
//             _ => RedisValueRef::ErrorMsg([0].to_vec()),
//         }
//     }
// }

// impl Encoder<RedisValueRef> for RespParser {
//     type Error = io::Error;
//     fn encode(&mut self, item: RedisValueRef, dst: &mut BytesMut) -> io::Result<()> {
//         write_resp_output(item, dst);
//         Ok(())
//     }
// }

fn write_resp_output(item: RedisValueRef, dst: &mut BytesMut) {
    match item {
        RedisValueRef::String(s) => {
            dst.extend_from_slice(b"+");
            dst.extend_from_slice(&s);
            dst.extend_from_slice(b"\r\n");
        },
        _ => println!(" not done yet")
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

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
                                println!("nothing came");
                            },
                            Ok(n) => {
                                // don't convert it to string it wont work and make work miserable
                                // convert buffer to message string
                                // let received: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&buf[..n]);
                                // println!("Received: {}", received);

                                // parse the string and get the result
                                if let Some(result) = parse_and_decode(&buf[..n]) {
                                    // println!("{:?}", String::from_utf8_lossy(&result));
                                    println!("Parsed Result: {:?}", result);
                                    if let Err(e) = stream.write_all(&result).await {
                                        eprintln!("Failed to write to stream: {}", e);
                                    }
                                } else {
                                    println!("Failed to parse the message.");
                                }

                                // write the result to stream
                                // if let Err(e) = stream.write_all(&buf[..n]).await {
                                //     eprintln!("Failed to write to stream: {}", e);
                                // }
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
            // Replace with actual logic to encode the value
            // return Some(&buf[pos..]); // Example: returning a slice of the buffer
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
    pos = pos + array_size as usize;
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
                }
                _ => todo!(),
            }
            // if v[0] == "ECHO" {
                
            // } else {
            //     return Vec::new();
            // }
        },
        _ => todo!(),
    }
}
