use clap::Parser;
use multithreading::ThreadPool;
use std::env::args;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value_t = get_ip_address())]
    address: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(short, long, default_value_t = num_cpus::get() / 2)]
    workers: usize,
}

fn get_ip_address() -> String {
    if cfg!(debug_assertions) {
        "127.0.0.1".parse().unwrap()
    } else {
        "0.0.0.0".parse().unwrap()
    }
}
fn main() {
    let parsed_args = Args::parse();

    let address = format!("{}:{}", parsed_args.address, parsed_args.port);

    let workers = parsed_args.workers;

    let listener = match TcpListener::bind(address.as_str()) {
        Ok(listener) => listener,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                println!(
                    "{}\nError: Could not open port {} due to insufficient permission",
                    parsed_args.port, e
                );
                #[cfg(target_os = "linux")]
                println!("tip: try running \"sudo setcap cap_net_bind_service=+ep {}\" to add permission or run it as sudo", args().collect::<Vec<String>>()[0]);
            } else {
                println!("Error: {}", e);
            }
            panic!();
        }
    };

    println!("Listening on {}", address);
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
    let bytes_read = stream.read(&mut buffer).unwrap();
    let buffer = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

    if !buffer.starts_with("GET") {
        return;
    }
    let file = buffer.split_whitespace().nth(1).unwrap_or_default();
    let file = parse_file(file);
    if file == "error_path_in_reverse" {
        let response = get_response(403, "Forbidden", "403 Forbidden".to_string());
        stream.write_all(response.as_ref()).unwrap();
        stream.flush().unwrap();
        return;
    }
    let file = format!("./{}", &file);
    println!("Serving file: {}", file);
    let file_contents =
        String::from_utf8_lossy(&std::fs::read(file).unwrap_or_default()).to_string();

    println!("Request: {}", &buffer);
    if file_contents.is_empty() {
        let response = get_response(404, "Not Found", "404 Not Found".to_string());
        stream.write_all(response.as_ref()).unwrap();
        stream.flush().unwrap();
        return;
    }
    let response = get_response(200, "OK", file_contents);
    stream.write_all(response.as_ref()).unwrap();
}

fn parse_file(file: &str) -> String {
    let mut file = file.to_string();
    if file.contains("../") {
        return "error_path_in_reverse".to_string();
    }
    if file.ends_with("/") {
        file = format!("{}index.html", file);
    }
    if file.starts_with("/") {
        file = file[1..].parse().unwrap();
    }
    file
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
