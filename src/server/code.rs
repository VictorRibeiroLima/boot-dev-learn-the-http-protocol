use std::fmt::Display;

#[derive(Debug)]
pub enum StatusCode {
    OK,
    BadRequest,
    NotFound,
    InternalServerError,
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b = self.bytes();
        let message = String::from_utf8_lossy(b);
        write!(f, "{}", message)
    }
}

impl StatusCode {
    pub fn bytes(&self) -> &[u8] {
        match &self {
            Self::OK => b"200 OK",
            Self::BadRequest => b"400 Bad Request",
            Self::NotFound => b"404 Not Found",
            Self::InternalServerError => b"500 Internal Server Error",
        }
    }

    pub fn byte_len(&self) -> usize {
        let bytes = self.bytes();
        bytes.len()
    }
}
