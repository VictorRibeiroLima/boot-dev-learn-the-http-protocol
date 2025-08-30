use std::{
    collections::HashMap,
    io::Error,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use crate::{
    method::HttpMethod, requests::Request, response::writer::ResponseWriter,
    server::code::StatusCode,
};

pub mod code;
pub mod response;

pub type HandleFunc = fn(req: Request, writer: &mut ResponseWriter);

type Path = String;
type Endpoint = HashMap<Path, HandleFunc>;

fn not_found(_req: Request, writer: &mut ResponseWriter) {
    let body = include_bytes!("../../static/not-found.html");
    let _ = writer.write_code(StatusCode::NotFound);
    let _ = writer.write_body(body);
    let _ = writer.write_header("Content-Type", "text/html");
}

fn handle_connection(stream: TcpStream, endpoints: Arc<Endpoint>) {
    let request = Request::new_from_reader(&stream);
    let request = match request {
        Ok(r) => r,
        Err(e) => {
            println!("ERROR: {:?}", e);
            return;
        }
    };

    let line = request.line();
    let endpoint = format!("{} {}", line.method, line.request_target);
    let mut writer = ResponseWriter::new(&stream);
    let endpoint = match endpoints.get(&endpoint) {
        Some(func) => *func,
        None => not_found,
    };

    endpoint(request, &mut writer);
    if writer.flushed() {
        return;
    }

    match writer.flush() {
        Err(e) => {
            eprintln!("{}", e)
        }
        Ok(()) => {}
    }
}

pub struct Server {
    addr: SocketAddr,
    listener: TcpListener,
    endpoints: Endpoint,
}

impl Server {
    pub fn new(port: u16) -> Result<Self, Error> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr)?;
        let endpoints: Endpoint = Default::default();
        Ok(Self {
            addr,
            listener,
            endpoints,
        })
    }

    pub fn add_handle_func(&mut self, method: HttpMethod, path: String, func: HandleFunc) {
        let endpoint = format!("{} {}", method, path);
        self.endpoints.insert(endpoint, func);
    }

    pub fn list_and_serve(self) {
        for endpoint in self.endpoints.keys() {
            println!("{}", endpoint)
        }
        println!("Server listening at {}", self.addr);
        //Someday i want to comeback to this and be able to not use a arc
        let endpoints = Arc::new(self.endpoints);
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let endpoints = Arc::clone(&endpoints);
                    thread::spawn(move || {
                        handle_connection(stream, endpoints);
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
