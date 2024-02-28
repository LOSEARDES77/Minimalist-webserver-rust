use std::net::{TcpListener, TcpStream};
use std::env::args;
use std::io::{Read, Write};
use http_server::ThreadPool;

fn main() {
let mut address;
    // CLI
    let mut workers = num_cpus::get() / 2;
    {
        let args = args().collect::<Vec<String>>();
        let mut port = String::new();
        let mut ip= String::new();
        for i in 0..args.len() {
            if args[i] == "-p" || args[i] == "--port" {
                port = args[i + 1].clone();
            }
            if args[i] == "-ip" || args[i] == "--address"{
                ip = args[i + 1].clone();
            }
            if args[i] == "-j" || args[i] == "--workers"{
                workers = args[i + 1].parse::<usize>().unwrap();
            }
            if args[i] == "--help" || args[i] == "-h" {
                #[cfg(windows)]
                println!("Usage: {} [OPTIONS]\n\
                          \tIf no options it will use 127.0.0.1:80 and half of you threads as default\n\
                          \t-p <port>, --port <port>                -  Port to listen on\n\
                          \t-ip <ip>, --address <ip>                -  IP to listen on\n\
                          \t-j <workers>, --workers <workers>       -  Number of workers to use\n\
                          \t--help, -h                              -  Print this help message",
                         args[0].split("\\").collect::<Vec<&str>>().last().unwrap()
                );
                #[cfg(unix)]
                println!("Usage: {} [OPTIONS]\n    If no option it will use 127.0.0.1:80 as default\n    -p <port>      -  Port to listen on\n    -ip <ip>       -  IP to listen on\n    --help, -h     -  Print this help message", args[0].split("/").collect::<Vec<&str>>().last().unwrap());
                return;
            }
        }
        if port.is_empty() {
            port = "80".to_string();
        }
        if ip.is_empty() {
            ip = "127.0.0.1".to_string();
        }
        address = format!("{}:{}", ip, port);
    }
    println!("Server started at {}", address);

    let listener = TcpListener::bind(address.as_str()).unwrap();
    let pool = ThreadPool::new(workers);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let address = &stream.peer_addr().unwrap();
        println!("Received request from {} port {}", address.ip(), address.port());

        pool.execute(|| {
            handle_connection(stream);
        });

    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    if !buffer.starts_with(b"GET") { return; }
    let file = buffer.split(|&x| x == b' ').collect::<Vec<&[u8]>>()[1];
    let mut file: &str = String::from_utf8_lossy(file).to_string().as_str();
    parse_file(&mut file);
    let file = &*format!("./{}", &file);
    let file_contents = std::fs::read(&file).unwrap_or_else(|_| Vec::from(""));


    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    let response = get_response(200, "OK", file_contents.as_slice());
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn parse_file(file: &mut &str) {
    if file.ends_with("/") {
        *file = &*format!("{}index.html", file);
    }
    if file.contains("../") {
        *file = "";
    }
    if file.starts_with("/") {
        *file = &file[1..];
    }
}

fn get_response(code: u8, status_line_message: &str, message: &[u8]) -> String {
    format!("HTTP/1.1 {} {}\r\nContext-Length: {}\r\n\r\n{:?}", code, status_line_message, message.len(), message)
}
