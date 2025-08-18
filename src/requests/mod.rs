use std::{collections::HashMap, fmt::Display, io::Read, ops::Index, result};

use crate::requests::{error::Error, method::HttpMethod, parser::RequestParser};

pub mod error;
mod method;
mod parser;

type Result<T> = result::Result<T, Error>;

const SEPARATOR: &[u8; 2] = b"\r\n";
const BUFFER_INITIAL_SIZE: usize = 1024;

#[derive(Debug)]
struct RequestLine {
    pub method: HttpMethod,
    pub request_target: String,
    pub http_version: String,
}

impl RequestLine {
    pub fn new_from_bytes(data: &[u8]) -> (usize, Result<Option<Self>>) {
        let b_idx = data
            .windows(SEPARATOR.len())
            .position(|window| window == SEPARATOR);

        let b_idx = match b_idx {
            Some(b_idx) => b_idx,
            None => {
                return (0, Ok(None));
            }
        };

        let total_read = b_idx + SEPARATOR.len();
        let data = &data[..b_idx];

        let parts: Vec<String> = data
            .split(|b| *b == b' ')
            .map(|b| String::from_utf8_lossy(b).to_string())
            .collect();

        let parts_len = parts.len();

        if parts_len != 3 {
            return (0, Err(Error::InvalidHeaderPartSize(parts_len)));
        }

        let method = &parts[0];
        let request_target = parts[1].to_string();
        let http_version = &parts[2];

        let method = match HttpMethod::try_from(method.as_str()) {
            Ok(method) => method,
            Err(e) => {
                return (0, Err(e));
            }
        };
        if http_version != "HTTP/1.1" {
            return (
                0,
                Err(Error::UnsupportedHttpVersion(http_version.to_string())),
            );
        }

        return (
            total_read,
            Ok(Some(Self {
                method,
                request_target,
                http_version: "1.1".to_string(),
            })),
        );
    }
}

impl Display for RequestLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Request line:")?;
        writeln!(f, "- Method: {}", self.method)?;
        writeln!(f, "- Target: {}", self.request_target)?;
        writeln!(f, "- Version: {}", self.http_version)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Request {
    line: RequestLine,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.line)
    }
}

impl Request {
    pub fn new_from_reader<R: Read>(mut r: R) -> Result<Self> {
        let mut parser = RequestParser::new();
        let mut buff: Vec<u8> = Vec::with_capacity(BUFFER_INITIAL_SIZE);
        let mut buff_idx = 0;

        buff.resize(BUFFER_INITIAL_SIZE, 0);

        while !parser.done() {
            let n_read = r.read(&mut buff).map_err(|e| Error::ReaderError(e))?;
            buff_idx = buff_idx + n_read;
            let p_read = parser.parse(&buff[..buff_idx])?;
            let buff_at_the_limit = buff_idx == buff.capacity();
            if p_read != 0 {
                buff.copy_within(p_read..buff_idx, 0);
                buff_idx = buff_idx - p_read;
            } else if buff_at_the_limit {
                let new_len = buff.len() * 2;
                buff.resize(new_len, 0);
            }
        }

        Ok(Self {
            line: parser.line.unwrap(), //see later
            headers: Default::default(),
            body: Default::default(),
        })
    }
}

#[cfg(test)]
mod test {

    use crate::requests::{Error, HttpMethod, Request};

    /// A test utility to simulate reading a variable number of bytes per chunk from a string.
    struct ChunkReader {
        data: Vec<u8>,
        num_bytes_per_read: usize,
        pos: usize,
    }

    impl ChunkReader {
        fn new(data: &str, num_bytes_per_read: usize) -> Self {
            Self {
                data: data.as_bytes().to_vec(),
                num_bytes_per_read,
                pos: 0,
            }
        }
    }

    impl std::io::Read for ChunkReader {
        fn read(&mut self, p: &mut [u8]) -> std::io::Result<usize> {
            if self.pos >= self.data.len() {
                return Ok(0); // EOF
            }
            let end_index = (self.pos + self.num_bytes_per_read).min(self.data.len());
            let n = (end_index - self.pos).min(p.len());
            for i in self.pos..self.pos + n {
                p[i] = self.data[i]
            }
            self.pos += n;
            Ok(n)
        }
    }

    #[test]
    fn test_good_request_line() {
        let request =  "GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let reader = ChunkReader::new(request, 2);
        let request = Request::new_from_reader(reader).unwrap();
        assert_eq!(request.line.method, HttpMethod::GET);
        assert_eq!(request.line.http_version, "1.1");
        assert_eq!(request.line.request_target, "/");
    }

    #[test]
    fn test_good_request_line_with_path() {
        let request =  "GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let reader = ChunkReader::new(request, 1);
        let request = Request::new_from_reader(reader).unwrap();
        assert_eq!(request.line.method, HttpMethod::GET);
        assert_eq!(request.line.http_version, "1.1");
        assert_eq!(request.line.request_target, "/coffee");
    }

    #[test]
    fn test_request_line_invalid_number_of_line_parts() {
        let request =  "/coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let reader = ChunkReader::new(request, 2);
        let err = Request::new_from_reader(reader).unwrap_err();
        assert_eq!(err, Error::InvalidHeaderPartSize(2))
    }

    #[test]
    fn test_request_line_invalid_out_of_order() {
        let request =  "/coffee GET HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let reader = ChunkReader::new(request, 2);
        let err = Request::new_from_reader(reader).unwrap_err();
        assert_eq!(err, Error::UnknownHttpMethod("/coffee".to_string()))
    }

    #[test]
    fn test_request_line_invalid_version() {
        let request =  "GET /coffee HTTP/1.2\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let reader = ChunkReader::new(request, 10);
        let err = Request::new_from_reader(reader).unwrap_err();
        assert_eq!(err, Error::UnsupportedHttpVersion("HTTP/1.2".to_string()))
    }

    #[test]
    fn test_buffer_resizing_on_large_request_line() {
        // 1025 'A's, then a space, then '/' and another space, then 'HTTP/1.1', then CRLF
        let long_method = "A".repeat(1025);
        let request_line = format!(
            "{} / HTTP/1.1\r\nHost: localhost:42069\r\n\r\n",
            long_method
        );
        let reader = ChunkReader::new(&request_line, 2);
        let err = Request::new_from_reader(reader).unwrap_err();
        // Should fail with unknown method, since 1025 'A's is not a valid method
        assert_eq!(err, Error::UnknownHttpMethod(long_method));
    }
}
