use socket_chat::{StreamPool, ThreadPool};
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let mut stream_pool = StreamPool::new(3);
    for stream in listener.incoming().take(3) {
        println!("get a stream");
        let stream = stream.unwrap();
        stream_pool.connect(stream);
    }
    println!("10 in");
    stream_pool.begin();
    println!("Shutting down.");
}
