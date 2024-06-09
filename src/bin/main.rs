use multithreading::ThreadPool;
use std::env::args;
use std::io::{Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream};

struct Address {
    ip: IpAddr,
    port: Port,
}

struct Port {
    port: u16,
}

impl Port {
    fn new(port: String) -> Option<Self> {
        let port = match port.parse() {
            Ok(port) => port,
            Err(_) => {
                println!("Invalid port number");
                return None;
            }
        };
        Some(Port { port })
    }

    fn as_str(&self) -> String {
        format!("{}", self.port)
    }
}

impl Address {
    fn new(ip: String, port: String) -> Option<Self> {
        let ip = match ip.parse() {
            Ok(ip) => ip,
            Err(_) => {
                println!("Invalid IP address");
                return None;
            }
        };

        let port = match Port::new(port) {
            Some(port) => port,
            None => {
                println!("Invalid port number");
                return None;
            }
        };

        Some(Address { ip, port })
    }

    fn as_str(&self) -> String {
        format!("{}:{}", self.ip.to_string(), self.port.as_str())
    }
}

fn main() {
    let address;
    // CLI
    let mut workers = 8;
    {
        let args = args().collect::<Vec<String>>();
        let mut port = String::new();
        let mut ip = String::new();
        for i in 0..args.len() {
            if args[i] == "-p" || args[i] == "--port" {
                port = args[i + 1].clone();
            }
            if args[i] == "-ip" || args[i] == "--address" {
                ip = args[i + 1].clone();
            }
            if args[i] == "-j" || args[i] == "--workers" {
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
                println!("Usage: {} [OPTIONS]\n\
                          \tIf no options it will use 127.0.0.1:80 and half of you threads as default\n\
                          \t-p <port>, --port <port>                -  Port to listen on\n\
                          \t-ip <ip>, --address <ip>                -  IP to listen on\n\
                          \t-j <workers>, --workers <workers>       -  Number of workers to use\n\
                          \t--help, -h                              -  Print this help message",
                         args[0].split("/").collect::<Vec<&str>>().last().unwrap()
                );
                return;
            }
        }
        if port.is_empty() {
            port = "80".to_string();
        }
        if ip.is_empty() {
            ip = "127.0.0.1".to_string();
        }
        address = Address::new(ip, port);
    }

    let address = match address {
        Some(addr) => addr,
        None => {
            println!("Invalid address");
            return;
        }
    };

    let listener = TcpListener::bind(address.as_str()).unwrap();
    println!("Listening on {}", address.as_str());
    println!("Using {} workers", workers);
    let pool = ThreadPool::new(workers);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let address = &stream.peer_addr().unwrap();
        println!(
            "Received request from {} port {}",
            address.ip(),
            address.port()
        );

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    if !buffer.starts_with(b"GET") {
        return;
    }
    let file = String::from_utf8_lossy(buffer.split(|&x| x == b' ').collect::<Vec<&[u8]>>()[1])
        .to_string();
    let file = parse_file(file);
    let file = format!("./{}", &file);
    println!("Serving file: {}", file);
    let file_contents =
        String::from_utf8_lossy(&std::fs::read(file).unwrap_or_default()).to_string();

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    let response;
    if file_contents.is_empty() {
        response = get_response(404, "Not Found", "404 Not Found".to_string());
    } else {
        response = get_response(200, "OK", file_contents);
    }
    stream.write_all(response.as_ref()).unwrap();
}

fn parse_file(file: String) -> String {
    let mut file = file;
    if file.contains("../") {
        return "".to_string();
    }
    if file.ends_with("/") {
        file = format!("{}index.html", file);
    }
    if file.starts_with("/") {
        file = file[1..].parse().unwrap();
    }
    return file;
}

fn get_response(code: u16, status_line_message: &str, message: String) -> String {
    format!(
        "HTTP/1.1 {} {}\r\nContext-Length: {}\r\n\r\n{}",
        code,
        status_line_message,
        message.len(),
        &message
    )
}
