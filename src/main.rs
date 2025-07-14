use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use clap::{Command, Arg};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;



#[tokio::main]
async fn main() {
    // Parsing command-line arguments with Clap version 4.x
    let matches = Command::new("CHX A Lightning cache server")
        .version("1.0.0")
        .author("Nux Xader rsayria@gmail.com")
        .about("In-memory key-value cache server")
        .arg(
            Arg::new("h")
                .long("host")
                .value_parser(clap::value_parser!(String))
                .default_value("127.0.0.1")
                .help("The host to bind the server to"),
        )
        .arg(
            Arg::new("p")
                .long("port")
                .value_parser(clap::value_parser!(u16))
                .default_value("3800")
                .help("The port to bind the server to"),
        )
        .arg(
            Arg::new("e")
                .long("expire_time")
                .value_parser(clap::value_parser!(u64))
                .default_value("2592000")
                .help("Expiration time for cache entries (in seconds)"),
        )
        .get_matches();

    // Retrieve argument values using `get_one`
    let host = matches.get_one::<String>("h").unwrap();
    let port = matches.get_one::<u16>("p").unwrap();
    let expire_time: u64 = *matches.get_one::<u64>("e").unwrap();

    // Change std::sync::Mutex to tokio::sync::Mutex
    let listener = TcpListener::bind(format!("{}:{}", host, port)).await.unwrap();
    let store: Arc<RwLock<HashMap<Vec<u8>, (Vec<u8>, Instant)>>> = Arc::new(RwLock::new(HashMap::new()));


    println!("Server listening on {}:{}", host, port);

    loop {
        // Menunggu koneksi dari client
        let (mut socket, _) = listener.accept().await.unwrap();
        let store_clone = Arc::clone(&store);

        // Menangani client secara asinkron
        tokio::spawn(async move {
            loop {
                let mut buffer: [u8; 524288] = [0; 524288];
                let n = socket.read(&mut buffer).await.unwrap();

                if n == 0 {break;}

                let data = &mut buffer[..n].splitn(3, |&byte| byte == 32);
                let mut expired_key: Option<Vec<u8>> = None;
                match data.next() {
                    Some(action) => {
                        match action {
                            // GET
                            &[71] => {
                                let key: Vec<u8> = data.next().unwrap_or_default().trim_ascii().to_vec();
                                if key.is_empty() {break;}
                                if let Some((val, timestamp)) = store_clone.read().await.get(&key) {
                                    if expire_time > 0 && timestamp.elapsed().as_secs() > expire_time {
                                        expired_key = Some(key);
                                        socket.write_all(&[33, 101]).await.unwrap();
                                    } else {
                                        let mut val_ = val.to_owned();
                                        val_.insert(0, 62);
                                        socket.write_all(&val_.as_slice()).await.unwrap();
                                    }
                                } else {
                                    socket.write_all(&[33]).await.unwrap();
                                }
                            }
                            // SET
                            &[83] => {
                                let key = data.next().unwrap_or_default().to_vec();
                                let val = data.next().unwrap_or_default().trim_ascii().to_vec();
                                if key.is_empty() || val.is_empty() {break;}

                                let timestamp = Instant::now();
                                store_clone.write().await.insert(key, (val, timestamp));
                            }
                            // DEL
                            &[68] => {

                            }
                            _ => {}
                        }
                    }
                    None => {}
                }

                if let Some(key) = expired_key {
                    store_clone.write().await.remove(&key);
                }                
            }

        });
    }
}
