#![allow(unused_imports)]
use std::rc::Rc;
use std::io::{self, BufRead, Read, Write};
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
    Array(Vec<RESPTypes>),
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

fn word(buf: &BytesMut, pos: usize) -> Option<(usize, BufSplit)> {
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

#[allow(clippy::unnecessary_wraps)]
fn simple_string(buf: &BytesMut, pos: usize) -> RedisResult {
    Ok(word(buf, pos).map(|(pos, word)| (pos, RESPTypes::String(word))))
}

fn parse(buf: &BytesMut, pos: usize) -> Result<Option<(usize, RESPTypes)>, RESPError> {
    if buf.is_empty() {
        return Ok(None);
    }

    match buf[pos] {
        b'+' => simple_string(buf, pos + 1),
        // b'-' => error(buf, pos + 1),
        // b'$' => bulk_string(buf, pos + 1),
        // b':' => resp_int(buf, pos + 1),
        // b'*' => array(buf, pos + 1),
        _ => Err(RESPError::UnknownStartingByte),
    }

}

pub struct RespParser;

impl Decoder for RespParser {
    type Item = RESPTypes;
    type Error = RESPError;
    fn decode(&mut self, buf: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {

        if buf.is_empty() {
            return Ok(None);
        }

        match parse(buf,0)? {
            Some((pos,value)) => {
                println!("{:?}",value);
                Ok(None)
            },
            None => Ok(None),
        }
    }
}

impl Encoder<RESPTypes> for RespParser {
    type Error = io::Error;

    fn encode(&mut self, item: RESPTypes, dst: &mut BytesMut) -> io::Result<()> {
        write_resp_output(item, dst);
        Ok(())
    }
}

fn write_resp_output(item: RESPTypes, dst: &mut BytesMut) {
    match item {
        RESPTypes::String(s) => {
            // TODO
            dst.extend_from_slice(b"+");
            // dst.extend_from_slice(&s);
            dst.extend_from_slice(b"\r\n");
        },
        _ => todo!()
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf: BytesMut = BytesMut::new();
    
            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        return
                    },
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
                
                // todo parsing here decode and process then encode and write the data to socket
                todo!();
    
                // if let Err(e) = socket.write_all(b"+PONG\r\n").await {
                //     eprintln!("failed to write to socket; err = {:?}", e);
                //     return;
                // }
            }
        });
    }
}
