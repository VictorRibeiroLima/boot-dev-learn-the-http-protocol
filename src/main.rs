use std::result;

use crate::{
    error::Error,
    requests::Request,
    response::writer::ResponseWriter,
    server::{code::StatusCode, Server},
};

mod error;
mod header;
mod method;
mod requests;
mod response;
mod server;

const SEPARATOR: &[u8; 2] = b"\r\n";
type Result<T> = result::Result<T, Error>;

fn your_problem(_req: Request, writer: &mut ResponseWriter) {
    let body = include_bytes!("../static/bad-request.html");
    let _ = writer.write_code(StatusCode::BadRequest);
    let _ = writer.write_body(body);
    let _ = writer.write_header("Content-Type", "text/html");
}

fn my_problem(_req: Request, writer: &mut ResponseWriter) {
    let body = include_bytes!("../static/internal-server-error.html");
    let _ = writer.write_code(StatusCode::InternalServerError);
    let _ = writer.write_body(body);
    let _ = writer.write_header("Content-Type", "text/html");
}

fn all_good(_req: Request, writer: &mut ResponseWriter) {
    let body = include_bytes!("../static/ok.html");
    let _ = writer.write_code(StatusCode::OK);
    let _ = writer.write_body(body);
    let _ = writer.write_header("Content-Type", "text/html");
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
    server.add_handle_func(method::HttpMethod::GET, "/".to_string(), all_good);
    server.add_handle_func(method::HttpMethod::GET, "/use-nvim".to_string(), all_good);

    server.list_and_serve();
}
