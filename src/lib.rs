mod server;

use std::{
    fs,
    io::prelude::*,
    net::{
        TcpStream,
    }, 
    sync::{
        Arc, 
        atomic::{AtomicU32, Ordering}
    },
};

#[derive(PartialEq, Debug)]
enum Status {
    Get,
    Post,
    Another,
}
#[derive(PartialEq, Debug)]
struct HttpRequest {
    status: Status,
    path: String,
    headers: String,
    body: Option<String>,
}
impl HttpRequest {
    fn new(request: &str) -> Self {
        let request = request.to_owned() + "\n";
        let mut whitespace_iter = request.split_ascii_whitespace();
        let status = match whitespace_iter
            .next()
            .expect("The buffer is empty") {
            "GET" => Status::Get,
            "POST" => Status::Post,
            _ => Status::Another,
        };
        let path = whitespace_iter
            .next()
            .to_owned()
            .unwrap_or(" ")
            .to_owned();
        
        let lines_vec = request.lines().collect::<Vec<_>>();

        let separator_index = lines_vec.iter().position(|&l| l.is_empty()).unwrap_or(0);

        let (_, bottom) = lines_vec.split_at(1); // http request without the first line

        let (headers, body) = bottom.split_at(separator_index);

        let headers = headers.join("\n");
        let body = body.join("\n");
        let body = if body.is_empty() {
            None
        } else {
            Some(body)
        };
        HttpRequest {
            status,
            path,
            headers,
            body,
        }
    }
}

#[derive(PartialEq, Debug)]
enum HttpResponse {
    Succes(String), // here the status code is always 200
    Failed(u32), // the u32 is the status code 
}

pub fn handle_http_request(mut stream: TcpStream, global_page_num: Arc<AtomicU32>) {
    let incoming_request = read_tcp_stream(&stream);

    let response = process_incoming_request(incoming_request, &global_page_num);
    
    let contents = read_file_contents(&response, &global_page_num);

    send_response(&mut stream, &response, &contents);
}

fn read_tcp_stream(mut stream: &TcpStream) -> HttpRequest {
    println!("-----------------------");

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let incoming_request_buffer= std::str::from_utf8(&buffer).unwrap_or("GET /problem Http/1.1\r\n\r\n\r\n");
    
    println!(
        "\x1B[1;34mIncoming: {:?}, from: {:?}\x1B[0m", 
        incoming_request_buffer.lines().next()
            .expect("failed to read the incoming status line"), 
        stream.peer_addr()
    );

    let incoming_request = 
        HttpRequest::new(incoming_request_buffer);
    
    incoming_request
}

fn extract_number_from_http_post(http_post: &str) -> Option<u32> {
    let start_index = http_post.find("{\"number\":")?;
    let end_index = http_post[start_index..].find("}")?;
    let number_str = &http_post[start_index + 10..start_index + end_index];
    number_str.parse::<u32>().ok()
}

fn process_incoming_request(
    incoming_request: HttpRequest, 
    global_page_num: &Arc<AtomicU32>
) -> HttpResponse {
    match incoming_request.status {
        Status::Get => match incoming_request.path.as_str() {
            "/" => HttpResponse::Succes("templates/index.html".to_owned()),
            "/admin" => HttpResponse::Succes("templates/admin.html".to_owned()),
            "/contents" => HttpResponse::Succes("templates/contents.html".to_owned()),
            "/global-variable" => HttpResponse::Succes("global-variable.num".to_owned()),
            file_path => {
                // no f*cking clue wth this is, worked before, now it needs this...
                let file_path = &(".".to_owned() + file_path); 
                if std::path::Path::new(file_path).exists() {
                    HttpResponse::Succes(file_path.to_owned())
                } else {
                    println!("\x1B[1;31mproblem: \"{}\"\x1B[0m", file_path);
                    HttpResponse::Failed(404)
                }
            },     
        },
        Status::Post => { // could be improved with a custom error type
            if incoming_request.path.as_str()
                .eq_ignore_ascii_case("/emiting-global") &&
                incoming_request.body.is_some() 
            {
                let option_page_num = extract_number_from_http_post(
                    &incoming_request.body.expect("extracting num from http Post #1")
                );
                if option_page_num.is_some() {
                    let valid_page_num = option_page_num.expect("extracting num from http Post #2");
                    global_page_num.store(valid_page_num, Ordering::Relaxed);
                    println!("global num: {}", valid_page_num);
                    return HttpResponse::Succes("this_really_doesnt_matter.none".to_owned())
                }
            }
            HttpResponse::Failed(404)
        },
        Status::Another => {
            println!("400 bad request: {:?}", incoming_request);
            HttpResponse::Failed(400)
        }
    }
}

fn read_file_contents(
    response: &HttpResponse, 
    global_page_num: &Arc<AtomicU32>
) -> Vec<u8> {
    match response {
        HttpResponse::Succes(file_name) => {
            let file_extension = file_name.rsplitn(2, '.').next();
            
            match file_extension {
                Some("png") => {
                    let mut image_data = Vec::new();
                    let mut file = std::fs::File::open(file_name)
                        .expect("Failed to open image file");
                    file.read_to_end(&mut image_data).expect("Failed to read image file");
                    image_data
                }
                Some("json") => {
                    let contents = fs::read_to_string(file_name)
                        .expect("Failed to read JSON file");
                    let json_data: serde_json::Value = serde_json::from_str(&contents)
                        .expect("Failed to read JSON file");
                    serde_json::to_vec(&json_data).expect("Failed to serialize JSON")
                },
                Some("num") => Vec::from(global_page_num.load(Ordering::Relaxed).to_string()),
                Some("none") => Vec::from(fs::read("templates/404.html".to_owned()).unwrap()),
                _ => {
                    let file = fs::read(file_name)
                        .expect(&format!("Failed to read the {} file", file_name));
                    Vec::from(file)
                },
            }
        },
        HttpResponse::Failed(_) => Vec::from(fs::read("templates/404.html".to_owned()).unwrap()), 
    }
}

fn send_response(stream: &mut TcpStream, http_response: &HttpResponse, contents: &[u8]) {
    let status_line = match http_response {
        HttpResponse::Succes(_) => "HTTP/1.1 200 OK".to_owned(),
        HttpResponse::Failed(error_num) => match error_num {
            400 => "HTTP/1.1 400 Bad Request".to_owned(),
            404 => "HTTP/1.1 404 Not Found".to_owned(),
            uknown_num => format!("HTTP/1.1 {}", uknown_num) // dont know if this works
        }
    };
    match http_response {
        HttpResponse::Succes(_) => println!("\x1B[1;32mResponse: {:?}\x1B[0m", status_line),
        HttpResponse::Failed(_) => println!("\x1B[1;31mResponse: {:?}\x1B[0m", status_line),
    }
    
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_request_new() {
        let http_str = "POST /emiting-global HTTP/1.1
Host: 420.069.0.000:7878

{{\"number\":8}}".to_owned();

        let request = HttpRequest::new(&http_str);

        assert_eq!(
            request,
            HttpRequest {
                status: Status::Post,
                path: "/emiting-global".to_owned(),
                headers: "Host: 420.069.0.000:7878\n".to_owned(),
                body: Some("{{\"number\":8}}".to_owned()),
            },
        )
    }
    #[test]
    fn test_proccesing_request() {
        let global_page_num = Arc::new(AtomicU32::new(5));
        let incoming_request = HttpRequest { 
            status: Status::Get, 
            path: "/".to_owned(), 
            headers: "Host: 420.069.0.000:7878".to_owned(), 
            body: Some("{{\"number\":8}}".to_owned()),
        };
        assert_eq!(
            process_incoming_request(incoming_request, &global_page_num),
            HttpResponse::Succes("templates/index.html".to_owned()),
        );
    }
    #[test]
    fn test_file_read() {
        let global_page_num = Arc::new(AtomicU32::new(5));

        read_file_contents(&HttpResponse::Succes("templates/index.html".to_owned()), &global_page_num);
        read_file_contents(&HttpResponse::Failed(404), &global_page_num);
    }

    #[test]
    fn test_extract_number_from_http_post() {
        let post_template = "POST /emiting-global HTTP/1.1
        Host: 192.168.0.105:7878
        Connection: keep-alive
        Content-Length: 12
        User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36 OPR/99.0.0.0
        Content-Type: application/json
        Accept: */*
        Origin: http://192.168.0.105:7878
        Referer: http://192.168.0.105:7878/admin
        Accept-Encoding: gzip, deflate
        Accept-Language: en-US,en;q=0.9".to_owned();
        
        assert_eq!(extract_number_from_http_post(
            &format!("{} \n {{\"number\":8}}", post_template)), 
            Some(8)
        );
        assert_eq!(extract_number_from_http_post(
            &format!("{} \n {{\"number\":-6}}", post_template)),
            None
        );
        assert_eq!(extract_number_from_http_post(
            &format!("{} \n {{\"number\":s}}", post_template)),
            None
        );
        assert_eq!(extract_number_from_http_post(
            &format!("{} \n", post_template)),
            None
        );
    }
}