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
mod path;
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

fn get_by_id(req: Request, writer: &mut ResponseWriter) {
    let id = req.get_path_value("id").unwrap_or("not found");
    let body = format!("The id sended was {}", id);
    let _ = writer.write_code(StatusCode::OK);
    let _ = writer.write_body(body.as_bytes());
}

fn main() {
    let mut server = Server::new(42069).expect("server to open");

    server
        .add_handle_func(method::HttpMethod::GET, "/yourproblem", your_problem)
        .expect("To add endpoint");
    server
        .add_handle_func(method::HttpMethod::GET, "/myproblem", my_problem)
        .expect("To add endpoint");
    server
        .add_handle_func(method::HttpMethod::GET, "/", all_good)
        .expect("To add endpoint");
    server
        .add_handle_func(method::HttpMethod::GET, "/use-nvim", all_good)
        .expect("To add endpoint");

    server
        .add_handle_func(method::HttpMethod::GET, "/user/{id}", get_by_id)
        .expect("To add endpoint");

    server.list_and_serve();
}
