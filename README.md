# Minimalist Webserver
___
A minimalist multithreaded webserver written in rust

### Compiling
```bash
git clone https://github.com/LOSEARDES77/Minimalist-webserver-rust.git # Clone project
cd Minimalist-webserver-rust # Change directory to project
cargo build --release # Compile project
sudo setcap cap_net_bind_service=+ep target/release/minimalist_webserver # Give the binary the ability to bind to ports
```

### Running
```bash
http_server # Run the server on port 80
```

### Usage
```txt
http_server [OPTIONS]\n\
    If no options it will use 127.0.0.1:80 and half of you threads as default
    -p <port>, --port <port>                -  Port to listen on
    -ip <ip>, --address <ip>                -  IP to listen on
    -j <workers>, --workers <workers>       -  Number of workers to use
    --help, -h                              -  Print this help message
```

### Installing
this wil install to .cargo/bin
```bash
cargo install --path .
```

