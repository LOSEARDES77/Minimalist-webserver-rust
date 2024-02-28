use std::net::{TcpListener, TcpStream};
use std::env::args;
use std::io::{BufRead, BufReader, Write};

fn main() {
let mut address;
    {
        let args = args().collect::<Vec<String>>();
        let mut port = String::new();
        let mut ip= String::new();
        for i in 0..args.len() {
            if args[i] == "-p" {
                port = args[i + 1].clone();
            }
            if args[i] == "-ip" {
                ip = args[i + 1].clone();
            }
        }
        address = format!("{}:{}", ip, port);
    }

    if address == ":" {
        address = "127.0.0.1:80".to_string();
    }
    println!("Server started at {}", address);
    let listener = TcpListener::bind(address.as_str()).unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let address = stream.peer_addr().unwrap();
        println!("{}:", address);
        // request handle ^^
        let mut reader = BufReader::new(&mut stream);
        let mut request_lines = String::new();
        loop {
            let mut request = String::new();
            reader.read_line(&mut request).unwrap();
            request_lines += &*request;
            if request.trim().is_empty() { break; }
        }


        for line in request_lines.split("\n") {
            if line.starts_with("GET") {
                handle_get_request(&line, &mut stream);
            }
        }

    }

    fn handle_get_request(request: &str, mut stream: &TcpStream) {
        match request.trim().split(" ").collect::<Vec<&str>>().as_slice() {
            ["GET", resource, "HTTP/1.1"] => {
                println!("Requested: {}", resource);
                let mut document = std::path::PathBuf::new();
                // document.push("htdocs");
                document.push(resource.trim_start_matches("/"));
                if resource.ends_with("/") || resource.is_empty() { document.push("index.html"); }
                println!("Document: {:?}", document);
                let file_contents = std::fs::read(document).unwrap_or_else(|_| Vec::from("404 Not Found"));
                if file_contents == Vec::from("404 Not Found") {
                    stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\n").unwrap();
                    stream.write_all(b"<h1>404 Not Found</h1>").unwrap();
                    return;
                }
                stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                stream.write_all(&file_contents).unwrap();
            }
            _ => {

            }
        }
    }
}
