use std::{
    net::{TcpListener, TcpStream},
    thread,
};

use crate::requests::Request;

mod requests;

fn handle_connection(stream: TcpStream) {
    let request = Request::new_from_reader(stream).unwrap();
    println!("{}", request)
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
