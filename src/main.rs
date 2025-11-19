use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;

fn main() {
    let mut cert = "cert.pem".to_string();
    let mut key = "key.pem".to_string();

    let mut args = std::env::args();
    let prog = args
        .next()
        .unwrap_or_else(|| "byond-ping-websocket-server".into());
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-c" | "--cert" => {
                if let Some(v) = args.next() {
                    cert = v;
                } else {
                    eprintln!("{}: missing value for {}", prog, arg);
                    std::process::exit(2);
                }
            }
            "-k" | "--key" => {
                if let Some(v) = args.next() {
                    key = v;
                } else {
                    eprintln!("{}: missing value for {}", prog, arg);
                    std::process::exit(2);
                }
            }
            "-h" | "--help" => {
                eprintln!(
                    "Usage: {} [-c|--cert <cert.pem>] [-k|--key <key.pem>]",
                    prog
                );
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                eprintln!(
                    "Usage: {} [-c|--cert <cert.pem>] [-k|--key <key.pem>]",
                    prog
                );
                std::process::exit(2);
            }
        }
    }

    let mut builder =
        openssl::ssl::SslAcceptor::mozilla_intermediate(openssl::ssl::SslMethod::tls()).unwrap();
    builder
        .set_certificate_chain_file(&cert)
        .unwrap_or_else(|e| {
            eprintln!("Failed to load certificate chain from {}: {}", cert, e);
            std::process::exit(1);
        });
    builder
        .set_private_key_file(&key, openssl::ssl::SslFiletype::PEM)
        .unwrap_or_else(|e| {
            eprintln!("Failed to load private key from {}: {}", key, e);
            std::process::exit(1);
        });
    let acceptor = builder.build();

    let server = TcpListener::bind("0.0.0.0:9001").unwrap();
    for stream in server.incoming() {
        let acceptor = acceptor.clone();

        spawn(move || {
            let mut websocket = match stream {
                Ok(tcp_stream) => match acceptor.accept(tcp_stream) {
                    Ok(ssl_stream) => match accept(ssl_stream) {
                        Ok(ws) => ws,
                        Err(e) => {
                            eprintln!("WebSocket accept error: {}", e);
                            return;
                        }
                    },
                    Err(e) => {
                        eprintln!("TLS accept error: {}", e);
                        return;
                    }
                },
                Err(e) => {
                    eprintln!("TCP accept error: {}", e);
                    return;
                }
            };

            loop {
                let Ok(msg) = websocket.read() else {
                    return;
                };

                if msg.is_binary() || msg.is_text() {
                    websocket.send(msg).unwrap();
                }
            }
        });
    }
}
