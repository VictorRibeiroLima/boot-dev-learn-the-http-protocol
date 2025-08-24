use std::{
    io::Error,
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
};

use crate::requests::Request;

fn handle_connection(stream: TcpStream) {
    let request = Request::new_from_reader(&stream);
    match request {
        Ok(r) => println!("{}", r),
        Err(e) => {
            println!("ERROR: {:?}", e)
        }
    }
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
