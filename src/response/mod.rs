use std::io::{self, Write};

use crate::{header::Headers, response::line::ResponseLine};

pub mod error;
mod line;
pub mod writer;

pub struct Response<'a> {
    line: &'a ResponseLine,
    headers: &'a Headers,
    body: &'a Vec<u8>,
}

impl Response<'_> {
    pub fn write_to<W: Write>(&self, mut w: W) -> io::Result<()> {
        self.line.write_to(&mut w)?;
        self.headers.write_to(&mut w)?;
        w.write_all(&self.body)
    }

    #[allow(dead_code)]
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
