use std::io::{self, Write};

use crate::{header::Headers, response::line::ResponseLine, server::code::StatusCode};

pub mod error;
mod line;

pub struct Response {
    line: ResponseLine,
    headers: Headers,
    body: Vec<u8>,
}

impl Response {
    pub fn new(body: Option<String>, code: StatusCode) -> Self {
        let body = match body {
            Some(b) => b.as_bytes().to_vec(),
            None => Vec::new(),
        };
        let content_length = body.len();

        let mut headers = Headers::default();
        headers.insert("Content-Length".to_string(), content_length.to_string());
        headers.insert("Connection".to_string(), "close".to_string());
        headers.insert("Content-Type".to_string(), "text/plain".to_string());

        let line = ResponseLine::new(code);

        Self {
            line,
            headers,
            body,
        }
    }

    pub fn write_to<W: Write>(&self, mut w: W) -> io::Result<()> {
        self.line.write_to(&mut w)?;
        self.headers.write_to(&mut w)?;
        w.write_all(&self.body)
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let line_len = self.line.byte_len();
        let headers_len = self.headers.byte_len();
        let body_len = self.body.len();
        let total_len = line_len + headers_len + body_len;
        let mut resp = Vec::with_capacity(total_len);
        self.write_to(&mut resp).unwrap(); //For now
        return resp;
    }
}
