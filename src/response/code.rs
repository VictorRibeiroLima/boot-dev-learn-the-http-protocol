#[derive(Debug)]
pub enum StatusCode {
    OK,
    NotFound,
    InternalServerError,
}

impl StatusCode {
    pub fn bytes(&self) -> &[u8] {
        match &self {
            Self::OK => b"200 OK",
            Self::InternalServerError => b"500 Internal Server Error",
            Self::NotFound => b"404 Not Found",
        }
    }

    pub fn byte_len(&self) -> usize {
        let bytes = self.bytes();
        bytes.len()
    }
}
