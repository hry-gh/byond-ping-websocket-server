use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;

fn main() {
    let mut builder =
        openssl::ssl::SslAcceptor::mozilla_intermediate(openssl::ssl::SslMethod::tls()).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();
    builder
        .set_private_key_file("key.pem", openssl::ssl::SslFiletype::PEM)
        .unwrap();
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
