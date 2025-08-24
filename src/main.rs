use std::result;

use crate::{error::Error, server::Server};

mod error;
mod header;
mod requests;
mod response;
mod server;

const SEPARATOR: &[u8; 2] = b"\r\n";
type Result<T> = result::Result<T, Error>;

fn main() {
    let server = Server::new(42069).expect("server to open");
    server.list_and_serve();
}
