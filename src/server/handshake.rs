use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::codec::encoder::encode_array;

pub async fn handshake(mut stream: TcpStream, port: String) {
    // send PING command
    let mut args = Vec::new();
    args.push("PING".to_string());
    let mut response = encode_array(args);
    stream = send_command(stream,response).await;
    println!("sends command PING");

    args = Vec::new();
    args.push("REPLCONF".to_string());
    args.push("listening-port".to_string());
    args.push(port);
    response = encode_array(args);
    stream = send_command(stream,response).await;
    println!("sends command ReplConf");

    args = Vec::new();
    args.push("REPLCONF".to_string());
    args.push("capa".to_string());
    args.push("psync2".to_string());
    response = encode_array(args);
    stream = send_command(stream,response).await;
    println!("sends command ReplConf psync2");

}

pub async fn send_command(mut stream: TcpStream, messages: Vec<u8>) -> TcpStream {
    if let Err(e) = stream.write_all(&messages).await {
        eprintln!("Failed to write to stream: {}", e);
    }
    return stream;
}