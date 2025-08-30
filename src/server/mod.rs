use std::{
    io::Error,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use crate::{
    method::HttpMethod, path::Path, requests::Request, response::writer::ResponseWriter,
    server::code::StatusCode,
};

pub mod code;
pub mod response;

pub type HandleFunc = fn(req: Request, writer: &mut ResponseWriter);

type Endpoints = Vec<(Path, HandleFunc)>;

fn find_endpoint<'a>(p: &Path, endpoints: &'a Endpoints) -> Option<(&'a Path, HandleFunc)> {
    endpoints
        .iter()
        .find(|(ep, _)| ep == p)
        .map(|(ep, func)| (ep, *func))
}

fn insert_endpoint(
    method: HttpMethod,
    path: &str,
    func: HandleFunc,
    endpoints: &mut Endpoints,
) -> Result<(), String> {
    let p = Path::new(method, path)?;
    if find_endpoint(&p, endpoints).is_some() {
        return Err("Endpoint already registred".to_string());
    }
    endpoints.push((p, func));
    Ok(())
}

fn not_found(_req: Request, writer: &mut ResponseWriter) {
    let body = include_bytes!("../../static/not-found.html");
    let _ = writer.write_code(StatusCode::NotFound);
    let _ = writer.write_body(body);
    let _ = writer.write_header("Content-Type", "text/html");
}

fn handle_connection(stream: TcpStream, endpoints: Arc<Endpoints>) {
    let request = Request::new_from_reader(&stream);
    let mut request = match request {
        Ok(r) => r,
        Err(e) => {
            println!("ERROR: {:?}", e);
            return;
        }
    };

    let line = request.line();
    let path = Path::new(line.method, &line.request_target).unwrap(); //see latter
    let mut writer = ResponseWriter::new(&stream);
    let (p, endpoint) = match find_endpoint(&path, &endpoints) {
        Some((p, func)) => (p, func),
        None => (&path, not_found as HandleFunc),
    };
    request.set_matched_path(p);

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
    endpoints: Endpoints,
}

impl Server {
    pub fn new(port: u16) -> Result<Self, Error> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr)?;
        let endpoints: Endpoints = Default::default();
        Ok(Self {
            addr,
            listener,
            endpoints,
        })
    }

    pub fn add_handle_func(
        &mut self,
        method: HttpMethod,
        path: &str,
        func: HandleFunc,
    ) -> Result<(), String> {
        insert_endpoint(method, path, func, &mut self.endpoints)
    }

    pub fn list_and_serve(self) {
        for (p, _) in &self.endpoints {
            println!("{}", p)
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
