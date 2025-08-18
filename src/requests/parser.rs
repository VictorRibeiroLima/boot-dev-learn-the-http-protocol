use std::{collections::HashMap, result};

use crate::{
    error::Error,
    header::{Headers, ProtoHeader},
    requests::RequestLine,
    Result,
};

#[derive(Debug, Default, PartialEq, Eq)]
enum ParserState {
    Done,
    Initialized,
    #[default]
    Uninitialized,
}

#[derive(Default, Debug)]
pub struct RequestParser {
    pub line: Option<RequestLine>,
    pub headers: Headers,
    pub body: Option<Vec<u8>>,
    state: ParserState,
    headers_done: bool,
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

    pub fn parse(&mut self, data: &[u8]) -> Result<usize> {
        let mut b_read = 0;
        if self.state == ParserState::Done {
            return Err(Error::AlreadyCloseParser);
        }

        if self.state == ParserState::Uninitialized {
            self.state = ParserState::Initialized
        }

        if self.line.is_none() {
            let (n_read, result) = RequestLine::new_from_bytes(data);
            let result = match result {
                Ok(result) => result,
                Err(e) => return Err(e),
            };
            b_read = b_read + n_read;

            if n_read == 0 {
                //No bytes where read we need more data
                return Ok(b_read);
            }

            //SAFETY: If bytes where read than we have a line
            let line = unsafe { result.unwrap_unchecked() };
            self.line = Some(line);
        }

        if !self.headers_done {
            loop {
                let data = &data[b_read..];
                let (n_read, result) = ProtoHeader::new_from_bytes(data);
                let result = match result {
                    Ok(result) => result,
                    Err(e) => return Err(e),
                };

                b_read = b_read + n_read;

                if n_read == 0 {
                    //No bytes where read we need more data
                    return Ok(b_read);
                }

                if n_read == 2 {
                    self.headers_done = true;
                    break;
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

        if self.line.is_some() && self.headers_done {
            self.state = ParserState::Done
        }

        Ok(b_read)
    }
}
