use lzma::LzmaReader;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use clap::{Command, Arg};
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use tokio::sync::{RwLock};
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
        let (socket, _) = listener.accept().await.unwrap();
        println!("New client connected!");

        // Menangani client secara asinkron
        tokio::spawn(handle_client(socket));
    }
}


async fn handler(mut socket: TcpStream, store_clone: Arc<RwLock<HashMap<Vec<u8>, (Vec<u8>, Instant)>>>, expire_time: u64) {
    loop {
        let mut buffer = Vec::with_capacity(10000000);
        println!("incoming data");
        let n = socket.read(&mut buffer).await.unwrap();
        println!("Received {} bytes", n);

        if n == 0 {break;}

        let data = &mut buffer[..n].splitn(2, |&byte| byte == 32);
        let mut expired_key: Option<Vec<u8>> = None;          
        match data.next() {
            Some(action) => {
                match action {
                    // GET
                    [71] => {
                        let store = store_clone.read().await;
                        let key = data.next().unwrap_or_default().to_vec();
                        if let Some((value, timestamp)) = store.get(&key) {
                            if expire_time > 0 && timestamp.elapsed().as_secs() > expire_time {
                                expired_key = Some(key);
                                socket.write_all(&[]).await.unwrap();
                            } else {
                                let mut decompressed_data = Vec::new();
                                {
                                    let mut reader = LzmaReader::new_decompressor(value.as_slice()).unwrap();
                                    reader.read_to_end(&mut decompressed_data).unwrap();
                                }
                                socket.write_all(decompressed_data.as_slice()).await.unwrap();
                            }
                        } else {
                            socket.write_all(&[]).await.unwrap();
                        }                                            
                    }
                    // SET
                    [83] => {
                        let mut store = store_clone.write().await;
                        let key = data.next().unwrap_or_default().to_vec();
                        let val = data.next().unwrap_or_default();
                        if key.is_empty() || val.is_empty() {
                            return;
                        }

                        let mut compressed_data = Vec::new();
                        {
                            let mut reader = LzmaReader::new_compressor(val, 5).unwrap();
                            reader.read_to_end(&mut compressed_data).unwrap();
                        }
                        let timestamp = Instant::now();
                        store.insert(key, (compressed_data, timestamp));
                        socket.write_all(&[]).await.unwrap();
                    }
                    // DEL
                    [68] => {

                    }
                    _ => {}
                }
            }
            None => {}
        }

        if let Some(key) = expired_key {
            let mut store = store_clone.write().await;
            store.remove(&key);
        }                
    }

}

async fn handle_client(mut stream: TcpStream) {

    loop {
        let mut buffer = Vec::with_capacity(1024);
        // Membaca data dari stream (ncat yang mengirimkan data)
        let bytes_read = stream.read(&mut buffer).await.unwrap();

        if bytes_read == 0 {
            // Tidak ada data lagi, keluar dari loop
            break;
        }

        // Menampilkan data yang diterima dari ncat
        let received_text = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received from client: {}", received_text);
    }

}

#[tokio::main]
async fn mainx() {
    let address = "127.0.0.1:8080"; // Alamat dan port untuk server
    let listener = TcpListener::bind(address).await.unwrap();

    println!("Server listening on {}", address);

    loop {
        // Menunggu koneksi dari client
        let (socket, _) = listener.accept().await.unwrap();
        println!("New client connected!");

        // Menangani client secara asinkron
        tokio::spawn(handle_client(socket));
    }
}
