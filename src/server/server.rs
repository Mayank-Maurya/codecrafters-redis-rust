use std::collections::HashMap;
use std::{env, vec};
use std::fs::File;
use std::ops::Add;
use std::rc::Rc;
use std::io::{self, BufRead, Error, Read, Write};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use bytes::{buf, Bytes, BytesMut};
use chrono::Utc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio_util::codec::{Decoder, Encoder};
use memchr::memchr;
use std::sync::{LazyLock, Mutex};

use crate::parsers::protocol::parse_and_decode;
use crate::parsers::rdb_file_parser::parse as rdb_file_parser;
use crate::store::store::{map_config_get, GLOBAL_HASHMAP_CONFIG};
use crate::{BufSplit, RESPError, RESPTypes, RedisResult, Value};

pub async fn start_master() -> io::Result<()> {
    // parsing rdb file
    rdb_file_parser();

    // listen as master
    let port = map_config_get(String::from("port")).map_or("6379".to_string(), |s| s.to_string());
    start_listener(port).await
}

pub async fn start_replica() -> io::Result<()> {
    let host = map_config_get(String::from("master_host")).map_or("6379".to_string(), |s| s.to_string());
    let mut port = map_config_get(String::from("master_port")).map_or("6379".to_string(), |s| s.to_string());
    // connect to master
    let master_connection = connect_to_master(String::from(host + ":" + &port)).await;
    match master_connection {
        Ok(stream) => {
            println!("connected to master");
        },
        Err(e) => {
            println!("couldn't to master: {}", e);
        },
    }

    // listen as replica(slave)
    port = map_config_get(String::from("port")).map_or("6379".to_string(), |s| s.to_string());
    start_listener(port).await
    
    // start sync process
}

pub async fn connect_to_master(address: String) -> Result<TcpStream, Error> {
    println!("master addresss: {}", address);
    match TcpStream::connect(address).await {
        Ok(stream) => {
            return Ok(stream);
        }, 
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn start_listener(port: String) -> io::Result<()> {
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
