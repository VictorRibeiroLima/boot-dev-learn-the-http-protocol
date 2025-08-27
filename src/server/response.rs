use std::{error::Error, fmt::Display};

use crate::server::code::StatusCode;

#[derive(Debug)]
pub struct ServerResponse {
    pub code: StatusCode,
    pub content: Option<String>,
}

impl Display for ServerResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self.content {
            Some(c) => format!("{}:{}", self.code, c),
            None => self.code.to_string(),
        };
        write!(f, "{}", message)
    }
}

//A server response can be an error
impl Error for ServerResponse {}
