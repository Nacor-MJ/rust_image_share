use rust_image_share::ThreadPool;
use std::{
    fs,
    io::prelude::*,
    net::{
        TcpListener,
        TcpStream, 
        SocketAddr
    },
};
use local_ip_address::local_ip;

fn main() {
    let my_ip = local_ip().unwrap();
    println!("my ip adress: {:?}", my_ip);

    let listener = TcpListener::bind(SocketAddr::new(my_ip, 7878)).unwrap();
    let pool = ThreadPool::new(1);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_http_request(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_http_request(mut stream: TcpStream) {
    println!("-----------------------");

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let incoming_request = std::str::from_utf8(&buffer).unwrap();
    
    println!("This: {:?}, from: {:?}", incoming_request.lines().next().expect("0"), stream.peer_addr());

    let incoming_status_line = std::str::from_utf8(&buffer).unwrap().lines().next().unwrap();

    let (status_line, file_name) = match incoming_status_line {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "templates/index.html".to_owned()),
        "GET /admin HTTP/1.1" => ("HTTP/1.1 200 OK", "templates/admin.html".to_owned()),
        "GET /contents HTTP/1.1" => ("HTTP/1.1 200 OK", "templates/contents.html".to_owned()),
        message => {
            let file_path = ".".to_owned() + message.get(message.find(' ').unwrap()..message.rfind(' ').unwrap()).unwrap().trim();

            if std::path::Path::new(&file_path).exists() {
                ("HTTP/1.1 200 OK", file_path)
            } else {
                println!("|{file_path}|");
                ("HTTP/1.1 404 NOT FOUND", "templates/404.html".to_owned())
            }
        },
    };

    let contents = if file_name.ends_with(".png") {
        let mut image_data = Vec::new();
        let mut file = std::fs::File::open(file_name).expect("Failed to open image file");
        file.read_to_end(&mut image_data).expect("Failed to read image file");
        image_data
    } else if file_name.ends_with(".json"){
        let contents = fs::read_to_string(file_name).expect("Failed to read JSON file");
        let json_data: serde_json::Value = serde_json::from_str(&contents).expect("Failed to read JSON file_");
        serde_json::to_vec(&json_data).expect("Failed to serialize JSON")
    } else {
        fs::read_to_string(file_name).unwrap().into_bytes()
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n",
        status_line,
        contents.len()
    );

    let mut response_bytes = response.into_bytes();
    response_bytes.extend_from_slice(&contents);

    stream.write_all(&response_bytes).expect("Failed to write response");
    stream.flush().expect("Failed to flush stream");

    println!("This: {:?}", status_line);
}
