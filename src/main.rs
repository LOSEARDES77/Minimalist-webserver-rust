use clap::Parser;
use multithreading::ThreadPool;
use std::env::args;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const FILE_EXPLORER_SKELETON: &str = "
<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta http-equiv=\"X-UA-Compatible\" content=\"IE=edge\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>File Explorer</title>
    </head>
    <body>
        <h1>File Explorer</h1>
        <ul>
            <FILES>
        </ul>
        <a href=\"<ONE_DIRECTORY_BACK>\"><button>Back</button></a>
    </body>
</html>
";

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
    let mut buffer = [0; 8192];
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

    let path = std::path::Path::new(&file);
    if path.is_dir() {
        let file_explorer_content = use_file_explorer(&file);
        let response = get_response(200, "OK", file_explorer_content);
        stream.write_all(response.as_ref()).unwrap();
        stream.flush().unwrap();
        return;
    }

    let file_contents = std::fs::read(&file);

    println!("Request: {}", &buffer);
    match file_contents {
        Ok(contents) => {
            let content_type = get_content_type(&file);
            let response = get_response_with_content_type(200, "OK", contents, &content_type);
            stream.write_all(response.as_ref()).unwrap();
        }
        Err(_) => {
            let file_explorer_content = use_file_explorer(".");
            let response = get_response(200, "OK", file_explorer_content);
            stream.write_all(response.as_ref()).unwrap();
        }
    }
    stream.flush().unwrap();
}

fn get_content_type(file: &str) -> String {
    let extension = std::path::Path::new(file)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("");

    match extension {
        "html" | "htm" => "text/html".to_string(),
        "css" => "text/css".to_string(),
        "js" => "application/javascript".to_string(),
        "png" => "image/png".to_string(),
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "gif" => "image/gif".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

fn get_response_with_content_type(
    code: u16,
    status_line_message: &str,
    message: Vec<u8>,
    content_type: &str,
) -> Vec<u8> {
    let headers = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        code,
        status_line_message,
        content_type,
        message.len()
    );
    let mut response = headers.into_bytes();
    response.extend_from_slice(&message);
    response
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

fn use_file_explorer(address: &str) -> String {
    let path = std::path::Path::new(address);
    let mut files = String::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_string_lossy();
            let relative_path = path.strip_prefix(".").unwrap_or(&path);
            files.push_str(&format!(
                "<li><a href=\"{}\">{}</a></li>\n",
                relative_path.display(),
                file_name
            ));
        }
    }

    let one_directory_back = path.parent().unwrap_or(path);

    FILE_EXPLORER_SKELETON.replace("<FILES>", &files).replace(
        "<ONE_DIRECTORY_BACK>",
        one_directory_back.display().to_string().as_str(),
    )
}
