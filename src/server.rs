use std::{
    io::{Error, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
};

use crate::{
    requests::Request,
    response::{code::StatusCode, Response},
};

fn handle_connection(mut stream: TcpStream) {
    let request = Request::new_from_reader(&stream);
    let request = match request {
        Ok(r) => r,
        Err(e) => {
            println!("ERROR: {:?}", e);
            return;
        }
    };

    _ = request;
    let response = Response::new("Hello World!\n".to_string(), StatusCode::OK);
    response.write_to(stream).unwrap();
}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(port: u16) -> Result<Self, Error> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr)?;
        Ok(Self { listener })
    }

    pub fn list_and_serve(self) {
        for stream in self.listener.incoming() {
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
}
