use std::net::TcpStream;

use crate::{
    header::Headers,
    response::{error::ResponseWriterError, line::ResponseLine, Response},
    server::code::StatusCode,
};

pub struct ResponseWriter<'a> {
    line: ResponseLine,
    headers: Headers,
    body: Vec<u8>,
    connection: bool,

    stream: &'a TcpStream,
    flushed: bool,
}

impl<'a> ResponseWriter<'a> {
    pub fn new(stream: &'a TcpStream) -> Self {
        let line = ResponseLine::new(StatusCode::OK);
        ResponseWriter {
            line,
            stream,
            flushed: false,
            connection: false,
            body: Default::default(),
            headers: Default::default(),
        }
    }
}

impl ResponseWriter<'_> {
    pub fn write_code(&mut self, code: StatusCode) -> Result<(), ResponseWriterError> {
        if self.flushed {
            return Err(ResponseWriterError::WriterAlreadyFlushed);
        }
        self.line.code = code;
        Ok(())
    }

    pub fn write_header(&mut self, key: &str, value: &str) -> Result<(), ResponseWriterError> {
        if self.flushed {
            return Err(ResponseWriterError::WriterAlreadyFlushed);
        }
        self.headers.insert(key.to_string(), value.to_string());
        Ok(())
    }

    pub fn write_body(&mut self, body: &[u8]) -> Result<(), ResponseWriterError> {
        if self.flushed {
            return Err(ResponseWriterError::WriterAlreadyFlushed);
        }
        self.body = body.to_vec();
        Ok(())
    }

    pub fn append_body(&mut self, body: &[u8]) -> Result<(), ResponseWriterError> {
        if self.flushed {
            return Err(ResponseWriterError::WriterAlreadyFlushed);
        }
        self.body.extend_from_slice(body);
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), ResponseWriterError> {
        if self.flushed {
            return Err(ResponseWriterError::WriterAlreadyFlushed);
        }
        let conn: String;

        if self.connection {
            conn = "open".to_string();
        } else {
            conn = "close".to_string();
        }

        let content_length = self.body.len();
        self.headers
            .overwrite("Content-Length".to_string(), content_length.to_string());
        self.headers.overwrite("Connection".to_string(), conn);
        self.headers
            .insert_if_not_exists("Content-Type".to_string(), "text/plain".to_string());

        let response = Response {
            body: &self.body,
            line: &self.line,
            headers: &self.headers,
        };
        response.write_to(self.stream)?;
        self.flushed = true;
        Ok(())
    }

    pub fn flushed(&self) -> bool {
        self.flushed
    }
}
