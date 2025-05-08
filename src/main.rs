#![allow(unused_imports)]
// custom modules
// mod
mod parsers;
mod utils;
mod codec;
mod store;
mod server;

// crates
use chrono::{DateTime, TimeDelta, Utc};
use codec::encoder::{encode_array, encode_bulk_string};
use parsers::args_parse::parse as args_parse;
use parsers::rdb_file_parser::parse as rdb_file_parse;
use server::server::{start_listener, start_master, start_replica};
use utils::utils::{generate_random_string, get_key_value_pair_string, map_get as map_get_generic};
use store::store::{map_config_get, map_get, map_insert, GLOBAL_HASHMAP, GLOBAL_HASHMAP_CONFIG};

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

// some definations
#[derive(Debug)]
pub struct BufSplit(usize, usize);

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

    // be a master or slave
    let is_slave;
    match map_config_get(String::from("master_host")) {
        Some(value) => {
            is_slave = value.len() > 0;
        },
        None => {
            is_slave = false;
        }
    }

    // start listening as master or slave
    if is_slave {
        match start_replica().await {
            Ok(result) => {
                
            },
            Err(e) => {
                println!("error at slave: {}", e);
            }
        }
    } else {
        match start_master().await {
            Ok(result) => {
                
            },
            Err(e) => {
                println!("error at master: {}", e);
            }
        }
    }

    Ok(())
}
