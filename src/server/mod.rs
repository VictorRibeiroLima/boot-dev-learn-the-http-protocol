use std::{
    collections::HashMap,
    io::Error,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use crate::{
    method::HttpMethod,
    requests::Request,
    response::Response,
    server::{code::StatusCode, response::ServerResponse},
};

pub mod code;
pub mod response;

pub type HandleFunc =
    fn(req: Request, stream: &TcpStream) -> Result<ServerResponse, ServerResponse>;

type Path = String;
type Endpoint = HashMap<Path, HandleFunc>;

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
    let response: Response = match endpoints.get(&endpoint) {
        Some(func) => {
            let resp = func(request, &stream);
            match resp {
                Ok(res) => Response::new(res.content, res.code),
                Err(res) => {
                    eprint!("Server error {}", res);
                    Response::new(res.content, res.code)
                }
            }
        }
        None => Response::new(Some("Not Found\n".to_string()), StatusCode::NotFound),
    };
    response.write_to(stream).unwrap();
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
