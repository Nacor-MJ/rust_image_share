mod server;
use crate::server::server::ThreadPool;
use rust_image_share::*;

use local_ip_address::local_ip;
use std::{
    net::{
        TcpListener,
        SocketAddr
    }, 
    sync::{
        Arc, 
        atomic::AtomicU32,
    },
};

// It's okay, I dont understand it either

fn main() {
    let ip_with_port = SocketAddr::new(local_ip().unwrap(), 7878);
    println!("\x1B[1;34mIp and port: {:?}\x1B[0m", ip_with_port);

    let global_page_num = Arc::new(AtomicU32::new(5));

    let listener = TcpListener::bind(ip_with_port).unwrap();
    let pool = ThreadPool::new(1);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let global_page_num = Arc::clone(&global_page_num);

        pool.execute(|| {
            handle_http_request(stream, global_page_num);
        });
    }

    println!("Shutting down.");
}