use std::{io::Write, net::TcpStream};

use crate::{
    header::Headers,
    response::{error::ResponseWriterError, line::ResponseLine, Response},
    server::code::StatusCode,
    SEPARATOR,
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

    pub fn chunked_writer(mut self) -> ChunkedResponseWriter<'a> {
        if self.flushed {
            let line = ResponseLine::new(StatusCode::OK);
            return ChunkedResponseWriter {
                line,
                headers_flushed: false,
                closed: false,
                headers: Default::default(),
                trailers: Default::default(),

                stream: self.stream,
            };
        }

        self.flushed = true;
        let mut headers = self.headers.clone();
        headers.overwrite("Transfer-Encoding".to_string(), "chunked".to_string());
        headers.remove("Content-Length");

        return ChunkedResponseWriter {
            line: self.line,
            headers_flushed: false,
            closed: false,
            headers,
            trailers: Default::default(),

            stream: self.stream,
        };
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

impl Drop for ResponseWriter<'_> {
    fn drop(&mut self) {
        //Trying to guarantee a flush,so that the handlers can get ownership of the writer
        if self.flushed() {
            return;
        }

        match self.flush() {
            Err(e) => {
                eprintln!("error flushing response writer on drop: {}", e)
            }
            Ok(()) => {}
        }
    }
}

pub struct ChunkedResponseWriter<'a> {
    line: ResponseLine,
    headers: Headers,
    trailers: Headers,

    stream: &'a TcpStream,
    headers_flushed: bool,
    closed: bool,
}

impl ChunkedResponseWriter<'_> {
    pub fn write_code(&mut self, code: StatusCode) -> Result<(), ResponseWriterError> {
        if self.headers_flushed {
            return Err(ResponseWriterError::WriterAlreadyFlushed);
        }
        if self.closed {
            return Err(ResponseWriterError::WriterAlreadyClosed);
        }
        self.line.code = code;
        Ok(())
    }

    pub fn write_header(&mut self, key: &str, value: &str) -> Result<(), ResponseWriterError> {
        if self.headers_flushed {
            return Err(ResponseWriterError::WriterAlreadyFlushed);
        }
        if self.closed {
            return Err(ResponseWriterError::WriterAlreadyClosed);
        }
        self.headers.insert(key.to_string(), value.to_string());
        Ok(())
    }

    pub fn write_trailer(&mut self, key: &str, value: &str) -> Result<(), ResponseWriterError> {
        if self.closed {
            return Err(ResponseWriterError::WriterAlreadyClosed);
        }
        self.trailers.insert(key.to_string(), value.to_string());
        Ok(())
    }

    pub fn flush_headers(&mut self) -> Result<usize, ResponseWriterError> {
        let line_bytes = self.line.byte_len();
        let header_bytes = self.headers.byte_len();
        let body = Vec::new();
        self.headers
            .insert_if_not_exists("Content-Type".to_string(), "text/plain".to_string());
        let response = Response {
            body: &body,
            line: &self.line,
            headers: &self.headers,
        };
        response.write_to(self.stream)?;
        self.headers_flushed = true;
        return Ok(line_bytes + header_bytes);
    }

    pub fn write(&mut self, p: &[u8]) -> Result<usize, ResponseWriterError> {
        let mut total = 0;
        if self.closed {
            return Err(ResponseWriterError::WriterAlreadyClosed);
        }
        if !self.headers_flushed {
            total += self.flush_headers()?;
        }

        let len = p.len();
        let hex_upper_str = format!("{:X}", len);
        let hex_upper = hex_upper_str.as_bytes();
        self.stream.write_all(hex_upper)?;
        self.stream.write_all(SEPARATOR)?;
        self.stream.write_all(p)?;

        self.stream.write_all(SEPARATOR)?;

        total += hex_upper.len();
        total += SEPARATOR.len();
        total += p.len();
        total += SEPARATOR.len();

        Ok(total)
    }

    pub fn close(&mut self) -> Result<(), ResponseWriterError> {
        if self.closed {
            return Err(ResponseWriterError::WriterAlreadyClosed);
        }
        self.stream.write_all(b"0")?;
        self.stream.write_all(SEPARATOR)?;
        self.trailers.write_to(self.stream)?;
        self.stream.write_all(SEPARATOR)?;
        self.closed = true;
        Ok(())
    }
}

impl Drop for ChunkedResponseWriter<'_> {
    fn drop(&mut self) {
        if self.closed {
            return;
        }

        match self.close() {
            Err(e) => {
                eprintln!("error flushing chunked response writer on drop: {}", e)
            }
            Ok(()) => {}
        }
    }
}
