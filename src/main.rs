use std::{
    fs::File,
    io::{BufReader, Read},
    result,
};

use sha2::{Digest, Sha256};

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

fn your_problem(_req: Request, mut writer: ResponseWriter) {
    let body = include_bytes!("../static/bad-request.html");
    let _ = writer.write_code(StatusCode::BadRequest);
    let _ = writer.write_body(body);
    let _ = writer.write_header("Content-Type", "text/html");
}

fn my_problem(_req: Request, mut writer: ResponseWriter) {
    let body = include_bytes!("../static/internal-server-error.html");
    let _ = writer.write_code(StatusCode::InternalServerError);
    let _ = writer.write_body(body);
    let _ = writer.write_header("Content-Type", "text/html");
}

fn all_good(_req: Request, mut writer: ResponseWriter) {
    let body = include_bytes!("../static/ok.html");
    let _ = writer.write_code(StatusCode::OK);
    let _ = writer.write_body(body);
    let _ = writer.write_header("Content-Type", "text/html");
}

fn get_by_id(req: Request, mut writer: ResponseWriter) {
    let id = req.get_path_value("id").unwrap_or("not found");
    let body = format!("The id sended was {}", id);
    let _ = writer.write_code(StatusCode::OK);
    let _ = writer.write_body(body.as_bytes());
}

fn stream_data(req: Request, writer: ResponseWriter) {
    let mut full_body: Vec<u8> = Vec::new();
    let proxy = match req.get_path_value("proxy") {
        Some(proxy) => proxy,
        None => {
            return your_problem(req, writer);
        }
    };
    let url = format!("https://www.httpbin.org/{}", proxy);
    let client = reqwest::blocking::Client::new();
    let mut resp = match client.get(url).send() {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("stream data 1: {}", e);
            return my_problem(req, writer);
        }
    };

    let mut writer = writer.chunked_writer();
    let _ = writer.write_code(StatusCode::OK);
    let _ = writer.write_header("Trailer", "X-Content-Sha256");
    let _ = writer.write_header("Trailer", "X-Content-Length");
    let _ = writer.flush_headers();
    let mut buffer: [u8; 1024] = [0; 1024];
    loop {
        let i = match resp.read(&mut buffer) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("stream data 2: {}", e);
                return;
            }
        };

        if i == 0 {
            break;
        }
        let buff = &buffer[..i];
        full_body.extend_from_slice(buff);

        let _ = writer.write(buff);
    }

    let hash: [u8; 32] = Sha256::digest(&full_body).into();
    let hex = hex::encode(hash);

    let _ = writer.write_trailer("X-Content-Length", &full_body.len().to_string());
    let _ = writer.write_trailer("X-Content-Sha256", &hex);
}

fn stream_video(req: Request, writer: ResponseWriter) {
    let f = match File::open("./assets/vim.mp4") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("stream_video 1 :{}", e);
            return my_problem(req, writer);
        }
    };
    let mut buffer_reader = BufReader::new(f);
    let mut writer = writer.chunked_writer();
    let _ = writer.write_code(StatusCode::OK);
    let _ = writer.write_header("Content-Type", "video/mp4");
    let _ = writer.flush_headers();
    let mut buffer: [u8; 1024] = [0; 1024];
    loop {
        let i = match buffer_reader.read(&mut buffer) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("stream_video 2: {}", e);
                return;
            }
        };

        if i == 0 {
            break;
        }
        let buff = &buffer[..i];

        let _ = writer.write(buff);
    }
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

    server
        .add_handle_func(method::HttpMethod::GET, "/httpbin/{proxy}", stream_data)
        .expect("To add endpoint");

    server
        .add_handle_func(method::HttpMethod::GET, "/video", stream_video)
        .expect("To add endpoint");

    server.list_and_serve();
}
