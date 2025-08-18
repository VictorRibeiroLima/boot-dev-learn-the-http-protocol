use std::collections::HashMap;

use crate::requests::{RequestLine, Result};

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
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
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

    pub fn parse(&mut self, data: &[u8]) -> Result<usize> {
        let mut b_read = 0;
        if self.state == ParserState::Done {
            return Err(super::error::Error::AlreadyCloseParser);
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

            if n_read == 0 {
                //No bytes where read we need more data
                return Ok(b_read);
            }

            //SAFETY: If bytes where read than we have a line
            let line = unsafe { result.unwrap_unchecked() };
            self.line = Some(line);
            b_read = n_read;
            //TODO: move this from here latter
            self.state = ParserState::Done;
        }

        Ok(b_read)
    }
}
