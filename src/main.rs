use rust_image_share::ThreadPool;
use local_ip_address::local_ip;
use std::{
    fs,
    io::prelude::*,
    net::{
        TcpListener,
        TcpStream, 
        SocketAddr
    }, 
    sync::{
        Arc, 
        atomic::{AtomicU32, Ordering}
    },
};


// It's okay, I dont understand it either

fn main() {
    let my_ip = local_ip().unwrap();
    println!("\x1B[1;34mmy ip adress: {:?}\x1B[0m", my_ip);

    let global_page_num = Arc::new(AtomicU32::new(5));

    let listener = TcpListener::bind(SocketAddr::new(my_ip, 7878)).unwrap();
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

fn handle_http_request(mut stream: TcpStream, global_page_num: Arc<AtomicU32>) {
    println!("-----------------------");

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let incoming_request = std::str::from_utf8(&buffer).unwrap();
    
    println!("\x1B[1;34mIncoming: {:?}, from: {:?}\x1B[0m", incoming_request.lines().next().expect("0"), stream.peer_addr());

    let incoming_status_line = std::str::from_utf8(&buffer).unwrap().lines().next().unwrap();

    let (status_line, file_name) = process_incoming_request(incoming_status_line, incoming_request, &global_page_num);
    
    let contents = read_file_contents(&file_name, &global_page_num);

    send_response(&mut stream, &status_line, &contents);

    match status_line {
        status if status.contains("200") => println!("\x1B[1;32mResponse: {:?}\x1B[0m", status),
        status if status.contains("404") => println!("\x1B[1;31mResponse: {:?}\x1B[0m", status),
        _ => println!("Response: {:?}", status_line),
    }
}

fn extract_number_from_http_post(http_post: &str) -> Option<u32> {
    let start_index = http_post.find("{\"number\":")?;
    let end_index = http_post[start_index..].find("}")?;
    let number_str = &http_post[start_index + 10..start_index + end_index];
    number_str.parse::<u32>().ok()
}

fn read_file_contents(file_name: &str, global_page_num: &Arc<AtomicU32>) -> Vec<u8> {
    let file_extension = file_name.rsplitn(2, '.').next();
    
    match file_extension {
        Some("png") => {
            let mut image_data = Vec::new();
            let mut file = std::fs::File::open(file_name).expect("Failed to open image file");
            file.read_to_end(&mut image_data).expect("Failed to read image file");
            image_data
        }
        Some("json") => {
            let contents = fs::read_to_string(file_name).expect("Failed to read JSON file");
            let json_data: serde_json::Value = serde_json::from_str(&contents).expect("Failed to read JSON file");
            serde_json::to_vec(&json_data).expect("Failed to serialize JSON")
        },
        Some("num") => Vec::from(global_page_num.load(Ordering::Relaxed).to_string()),
        Some("none") => Vec::from(b" ".to_owned()),
        _ => {
            let file = fs::read(file_name).expect(&format!("Failed to read the {} file", file_name));
            Vec::from(file)
        },
    }
}

fn process_incoming_request(
    incoming_status_line: &str, 
    incoming_request: &str, 
    global_page_num: &Arc<AtomicU32>
) -> (String, String) {
    match incoming_status_line {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK".to_owned(), "templates/index.html".to_owned()),
        "GET /admin HTTP/1.1" => ("HTTP/1.1 200 OK".to_owned(), "templates/admin.html".to_owned()),
        "GET /contents HTTP/1.1" => ("HTTP/1.1 200 OK".to_owned(), "templates/contents.html".to_owned()),
        "GET /global-variable HTTP/1.1" => ("HTTP/1.1 200 OK".to_owned(), "global-variable.num".to_owned()),
        "POST /emiting-global HTTP/1.1" => {
            global_page_num.store(
                extract_number_from_http_post(incoming_request)
                    .expect(&format!("Trouble extracting the incoming number:\n{}", incoming_request)),
                Ordering::Relaxed
            );
            println!("global num: {}", global_page_num.load(Ordering::Relaxed));
            ("HTTP/1.1 200 OK".to_owned(), "this_really_doesnt_matter_boobs_are_great_though.none".to_owned())
        },
        _ if incoming_request.chars().all(|c| c == '\0') => {
            ("HTTP/1.1 200 OK".to_owned(), "templates/404.html".to_owned())
        },
        message => {
            let file_path = ".".to_owned() + message.get(
                    message.find(' ').unwrap()..message.rfind(' ').unwrap()
                ).unwrap().trim();

            if std::path::Path::new(&file_path).exists() {
                ("HTTP/1.1 200 OK".to_owned(), file_path)
            } else {
                println!("\x1B[1;31mproblem: |{}|\x1B[0m", file_path);
                ("HTTP/1.1 404 NOT FOUND".to_owned(), "templates/404.html".to_owned())
            }
        },
    }
}

fn send_response(stream: &mut TcpStream, status_line: &str, contents: &[u8]) {
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n",
        status_line,
        contents.len()
    );

    let mut response_bytes = response.into_bytes();
    response_bytes.extend_from_slice(contents);

    stream.write_all(&response_bytes).expect("Failed to write response");
    stream.flush().expect("Failed to flush stream");
}