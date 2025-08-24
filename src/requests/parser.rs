use std::cmp::min;

use crate::{
    error::Error,
    header::{Headers, ProtoHeader},
    requests::RequestLine,
    Result,
};

#[derive(Debug, Default, PartialEq, Eq)]
enum ParserState {
    Done,
    ParsingLine,
    ParsingHeaders,
    ParsingBody,
    #[default]
    Uninitialized,
}

#[derive(Default, Debug)]
pub struct RequestParser {
    pub line: Option<RequestLine>,
    pub headers: Headers,
    pub body: Option<Vec<u8>>,
    content_length: usize,
    state: ParserState,
}

impl RequestParser {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn done(&self) -> bool {
        self.state == ParserState::Done
    }

    pub fn content_length(&self) -> Result<usize> {
        if self.content_length > 0 {
            return Ok(self.content_length);
        }

        //If content_length == 0 this may mean that we need to find on the headers

        let opt_content_length = self.headers.get("content-length");
        let raw_content_length = match opt_content_length {
            None => return Ok(0),
            Some(rcl) => rcl,
        };

        let content_length = raw_content_length
            .parse::<usize>()
            .map_err(|_| Error::MalFormedContentLengthHeader(raw_content_length.to_string()))?;

        return Ok(content_length);
    }

    pub fn parse(&mut self, data: &[u8]) -> Result<usize> {
        let mut b_read = 0;
        if self.state == ParserState::Done {
            return Err(Error::AlreadyCloseParser);
        }

        if self.state == ParserState::Uninitialized {
            self.state = ParserState::ParsingLine
        }

        if self.state == ParserState::ParsingLine {
            let n_read = self.parse_line(data)?;
            b_read += n_read;
        }

        if self.state == ParserState::ParsingHeaders {
            let n_read = self.parse_headers(b_read, data)?;
            b_read += n_read;
        }

        if self.state == ParserState::ParsingBody {
            let n_read = self.parse_body(b_read, data)?;
            b_read += n_read;
        }
        Ok(b_read)
    }

    fn parse_line(&mut self, data: &[u8]) -> Result<usize> {
        let (n_read, result) = RequestLine::new_from_bytes(data);
        let result = match result {
            Ok(result) => result,
            Err(e) => return Err(e),
        };

        if n_read == 0 {
            //No bytes where read we need more data
            return Ok(n_read);
        }

        //SAFETY: If bytes where read than we have a line
        let line = unsafe { result.unwrap_unchecked() };
        self.line = Some(line);
        self.state = ParserState::ParsingHeaders;
        Ok(n_read)
    }

    fn parse_headers(&mut self, init_read: usize, data: &[u8]) -> Result<usize> {
        let mut b_read = 0;
        loop {
            let data = &data[(b_read + init_read)..];
            let (n_read, result) = ProtoHeader::new_from_bytes(data);
            let result = match result {
                Ok(result) => result,
                Err(e) => return Err(e),
            };

            b_read += n_read;

            if n_read == 0 {
                //No bytes where read we need more data
                return Ok(b_read);
            }

            if n_read == 2 {
                self.state = ParserState::ParsingBody;
                return Ok(b_read);
            }

            //SAFETY: If bytes where read than we have a line
            let result = unsafe { result.unwrap_unchecked() };
            let key = result.key;
            let value = result.value;

            match self.headers.get_mut(&key) {
                Some(v) => v.push_str(format!(", {}", value).as_str()),
                None => {
                    self.headers.insert(key, value);
                }
            }
        }
    }

    fn parse_body(&mut self, init_read: usize, data: &[u8]) -> Result<usize> {
        let data = &data[init_read..];
        //Everything is going to the body
        let read_data = data.len();
        let content_length = self.content_length()?;
        self.content_length = content_length;

        let body = match self.body.as_mut() {
            None => {
                let b = Vec::with_capacity(content_length);
                self.body = Some(b);
                //SAFETY: We just set this
                unsafe { self.body.as_mut().unwrap_unchecked() }
            }
            Some(b) => b,
        };

        //I really don't know if should error when the data is bigger the the "content_length" our just read until there
        //the leader seems the "safer option" to refactor

        //If body already has some data we need to grab less data
        let remaining_length = content_length - body.len();
        let end = min(remaining_length, read_data);

        let data = &data[..end];

        body.extend_from_slice(data);
        if body.len() == content_length {
            self.state = ParserState::Done;
        }
        return Ok(read_data);
    }
}
