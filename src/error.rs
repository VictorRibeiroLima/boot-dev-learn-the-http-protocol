#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    AlreadyCloseParser,
    UnknownHttpMethod(String),
    UnsupportedHttpVersion(String),
    InvalidLinePartSize(usize),
    ReaderError(std::io::Error),
    MalFormedHeader(String),
    MalFormedContentLengthHeader(String),
    BodyBiggerThanContentLength,
    BodySmallerThanContentLength,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UnknownHttpMethod(l0), Self::UnknownHttpMethod(r0)) => l0 == r0,
            (Self::UnsupportedHttpVersion(l0), Self::UnsupportedHttpVersion(r0)) => l0 == r0,
            (Self::InvalidLinePartSize(l0), Self::InvalidLinePartSize(r0)) => l0 == r0,
            (Self::ReaderError(_), Self::ReaderError(_)) => true,
            (Self::MalFormedHeader(l0), Self::MalFormedHeader(r0)) => l0 == r0,
            (Self::MalFormedContentLengthHeader(l0), Self::MalFormedContentLengthHeader(r0)) => {
                l0 == r0
            }
            (Self::BodyBiggerThanContentLength, Self::BodyBiggerThanContentLength) => true,
            (Self::BodySmallerThanContentLength, Self::BodySmallerThanContentLength) => true,
            _ => false,
        }
    }
}
