use std::{
    net::{TcpListener, TcpStream},
    result, thread,
};

use crate::{error::Error, requests::Request};

mod error;
mod header;
mod requests;

const SEPARATOR: &[u8; 2] = b"\r\n";
type Result<T> = result::Result<T, Error>;

fn handle_connection(stream: TcpStream) {
    let request = Request::new_from_reader(&stream);
    match request {
        Ok(r) => println!("{}", r),
        Err(e) => {
            println!("ERROR: {:?}", e)
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:42069").expect("To open the tcp");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(err) => {
                eprintln!("{}", err);
                panic!()
            }
        }
    }
}
