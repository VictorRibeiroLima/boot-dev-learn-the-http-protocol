use std::fmt::Display;

use crate::{error::Error, method::HttpMethod, requests::Result, SEPARATOR};

#[derive(Debug)]
pub struct RequestLine {
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
            return (0, Err(Error::InvalidLinePartSize(parts_len)));
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
