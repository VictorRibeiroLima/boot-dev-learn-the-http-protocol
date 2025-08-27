use std::{net::TcpStream, result};

use crate::{
    error::Error,
    requests::Request,
    server::{code::StatusCode, response::ServerResponse, Server},
};

mod error;
mod header;
mod method;
mod requests;
mod response;
mod server;

const SEPARATOR: &[u8; 2] = b"\r\n";
type Result<T> = result::Result<T, Error>;

fn your_problem(
    _req: Request,
    _stream: &TcpStream,
) -> result::Result<ServerResponse, ServerResponse> {
    Err(ServerResponse {
        code: StatusCode::BadRequest,
        content: Some("Your problem is not my problem\n".to_string()),
    })
}

fn my_problem(
    _req: Request,
    _stream: &TcpStream,
) -> result::Result<ServerResponse, ServerResponse> {
    Err(ServerResponse {
        code: StatusCode::InternalServerError,
        content: Some("Woopsie, my bad\n".to_string()),
    })
}

fn all_good(_req: Request, _stream: &TcpStream) -> result::Result<ServerResponse, ServerResponse> {
    Ok(ServerResponse {
        code: StatusCode::OK,
        content: Some("All good, frfr\n".to_string()),
    })
}

fn main() {
    let mut server = Server::new(42069).expect("server to open");
    server.add_handle_func(
        method::HttpMethod::GET,
        "/yourproblem".to_string(),
        your_problem,
    );
    server.add_handle_func(
        method::HttpMethod::GET,
        "/myproblem".to_string(),
        my_problem,
    );
    server.add_handle_func(method::HttpMethod::GET, "/use-nvim".to_string(), all_good);
    server.list_and_serve();
}
