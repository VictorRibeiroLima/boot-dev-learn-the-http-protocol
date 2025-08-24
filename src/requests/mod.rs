use std::{fmt::Display, io::Read};

use crate::{
    error::Error,
    header::Headers,
    requests::{line::RequestLine, parser::RequestParser},
    Result,
};

mod line;
mod method;
mod parser;

const BUFFER_INITIAL_SIZE: usize = 1024;

#[derive(Debug)]
pub struct Request {
    line: RequestLine,
    headers: Headers,
    body: Vec<u8>,
}

impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.line)?;
        write!(f, "{}", self.headers)?;
        writeln!(f, "Body:")?;
        write!(f, "{}", String::from_utf8_lossy(&self.body))?;
        Ok(())
    }
}

impl Request {
    pub fn new_from_reader<R: Read>(mut r: R) -> Result<Self> {
        let mut parser = RequestParser::new();
        let mut buff: Vec<u8> = Vec::with_capacity(BUFFER_INITIAL_SIZE);
        let mut buff_idx = 0;

        buff.resize(BUFFER_INITIAL_SIZE, 0);

        /*
        I was doing this on a loop that can only break after a "final" read.
        But when the reader is a TCPStream the n_read is only going to be 0 when the client drops the connection.
        The "if n_read == 0" still fine because the client could drop without given me the full body

        but any other conditions that evolve checking if the parser is done after a read blocks the entire thing from properly responding to a real tcp call waiting a response
        */
        while !parser.done() {
            let n_read = r
                .read(&mut buff[buff_idx..])
                .map_err(|e| Error::ReaderError(e))?;
            if n_read == 0 {
                //This means that the content-length was not meet by the parser and we don't have any more body to read
                //The body is smaller then the specified content-length
                return Err(Error::BodySmallerThanContentLength);
            }
            /*
                if n_read > 0 && parser.done() {
                    //This means that the content-length was meet by the parser and we just read more that
                    //The body is bigger than the specified content-length
                    return Err(Error::BodyBiggerThanContentLength);
                }
                if n_read == 0 && parser.done() {
                    break;
                }
            */
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
            headers: parser.headers,
            body: parser.body.unwrap(), //see later
        })
    }
}

#[cfg(test)]
mod test {

    use crate::requests::{method::HttpMethod, Error, Request};

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
            p[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
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
        assert_eq!(err, Error::InvalidLinePartSize(2))
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

    #[test]
    fn test_parsing_headers() {
        let request =  "GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let reader = ChunkReader::new(request, 2);
        let request = Request::new_from_reader(reader).unwrap();
        assert_eq!(
            request.headers.get("host"),
            Some(&"localhost:42069".to_string())
        );
        assert_eq!(
            request.headers.get("user-agent"),
            Some(&"curl/7.81.0".to_string())
        );
        assert_eq!(request.headers.get("accept"), Some(&"*/*".to_string()));
    }

    #[test]
    fn test_parsing_headers_multiple_values() {
        let request =  "GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nSet-Person: lane-loves-go\r\nSet-Person: prime-loves-zig\r\nSet-Person: tj-loves-ocaml\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let reader = ChunkReader::new(request, 2);
        let request = Request::new_from_reader(reader).unwrap();
        assert_eq!(
            request.headers.get("host"),
            Some(&"localhost:42069".to_string())
        );
        assert_eq!(
            request.headers.get("user-agent"),
            Some(&"curl/7.81.0".to_string())
        );
        assert_eq!(
            request.headers.get("set-person"),
            Some(&"lane-loves-go, prime-loves-zig, tj-loves-ocaml".to_string())
        );
        assert_eq!(request.headers.get("accept"), Some(&"*/*".to_string()));
    }

    #[test]
    fn test_parsing_body_good_body() {
        let request = "POST /submit HTTP/1.1\r\n".to_string()
            + "Host: localhost:42069\r\n"
            + "Content-Length: 13\r\n"
            + "\r\n"
            + "hello world!\n";

        let reader = ChunkReader::new(&request, 2);
        let request = Request::new_from_reader(reader).unwrap();
        assert_eq!(
            request.headers.get("host"),
            Some(&"localhost:42069".to_string())
        );
        assert_eq!(
            request.headers.get("content-length"),
            Some(&"13".to_string())
        );
        assert_eq!(
            String::from_utf8_lossy(&request.body).to_string(),
            "hello world!\n".to_string()
        )
    }

    #[test]
    fn test_parsing_body_partial_content() {
        let request = "POST /submit HTTP/1.1\r\n".to_string()
            + "Host: localhost:42069\r\n"
            + "Content-Length: 20\r\n"
            + "\r\n"
            + "partial content";

        let reader = ChunkReader::new(&request, 2);
        let error = Request::new_from_reader(reader).unwrap_err();

        assert_eq!(error, Error::BodySmallerThanContentLength);
    }

    #[test]
    fn test_parsing_body_bigger_than_content_length() {
        let request = "POST /submit HTTP/1.1\r\n".to_string()
            + "Host: localhost:42069\r\n"
            + "Content-Length: 13\r\n"
            + "\r\n"
            + "hello world!\naaaaaaaaaaaaaaaaaaaa";

        let reader = ChunkReader::new(&request, 2);

        //Maybe this is wrong and we should error instead of grabbing just enough of the data
        let request = Request::new_from_reader(reader).unwrap();
        assert_eq!(
            request.headers.get("host"),
            Some(&"localhost:42069".to_string())
        );
        assert_eq!(
            request.headers.get("content-length"),
            Some(&"13".to_string())
        );
        assert_eq!(
            String::from_utf8_lossy(&request.body).to_string(),
            "hello world!\n".to_string()
        )
    }
}
