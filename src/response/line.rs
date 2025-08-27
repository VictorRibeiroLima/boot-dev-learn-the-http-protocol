use std::io::{self, Write};

use crate::{server::code::StatusCode, SEPARATOR};

const VERSION_BYTES: &[u8; 8] = b"HTTP/1.1";

#[derive(Debug)]
pub struct ResponseLine {
    pub http_version: [u8; 8],
    pub code: StatusCode,
}

impl ResponseLine {
    pub fn new(code: StatusCode) -> Self {
        Self {
            http_version: *VERSION_BYTES,
            code,
        }
    }

    pub fn write_to<W: Write>(&self, mut w: W) -> io::Result<()> {
        w.write_all(&self.http_version)?;
        w.write_all(b" ")?;
        w.write_all(&self.code.bytes())?;
        w.write_all(SEPARATOR)
    }

    pub fn byte_len(&self) -> usize {
        return 8 + self.code.byte_len();
    }
}
