#[derive(Debug)]
pub enum Error {
    AlreadyCloseParser,
    UnknownHttpMethod(String),
    UnsupportedHttpVersion(String),
    InvalidHeaderPartSize(usize),
    ReaderError(std::io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UnknownHttpMethod(l0), Self::UnknownHttpMethod(r0)) => l0 == r0,
            (Self::UnsupportedHttpVersion(l0), Self::UnsupportedHttpVersion(r0)) => l0 == r0,
            (Self::InvalidHeaderPartSize(l0), Self::InvalidHeaderPartSize(r0)) => l0 == r0,
            (Self::ReaderError(_), Self::ReaderError(_)) => true,
            _ => false,
        }
    }
}
