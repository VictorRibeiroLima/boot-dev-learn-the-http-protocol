use std::{
    collections::HashMap,
    fmt::{self, Display},
    io::{self, Write},
    ops::{Deref, DerefMut},
};

use crate::{error::Error, Result, SEPARATOR};

const JOIN_HEADER: &[u8; 2] = b": ";

#[derive(Debug, Default)]
pub struct Headers(HashMap<String, String>);

impl Display for Headers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Headers:")?;
        for (k, v) in &self.0 {
            writeln!(f, "- {}: {}", k, v)?;
        }
        Ok(())
    }
}

impl Deref for Headers {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Headers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Headers {
    pub fn write_to<W: Write>(&self, mut w: W) -> io::Result<()> {
        for (key, value) in self.iter() {
            w.write_all(key.as_bytes())?;
            w.write_all(JOIN_HEADER)?;
            w.write_all(value.as_bytes())?;
            w.write_all(SEPARATOR)?;
        }
        w.write_all(SEPARATOR)
    }

    pub fn byte_len(&self) -> usize {
        //It always has the final "/r/n"
        let mut result = 2;
        for (key, value) in self.iter() {
            result += key.as_bytes().len();
            result += value.as_bytes().len();
            //Accounting for ": " and "/r/n"
            result += 4;
        }
        return result;
    }

    pub fn push_from_proto(&mut self, ph: ProtoHeader) {
        let key = ph.key;
        let value = ph.value;
        self.insert(key, value);
    }

    pub fn insert(&mut self, key: String, value: String) {
        match self.get_mut(&key) {
            Some(v) => v.push_str(format!(", {}", value).as_str()),
            None => {
                self.0.insert(key, value);
            }
        };
    }

    pub fn overwrite(&mut self, key: String, value: String) {
        match self.get_mut(&key) {
            Some(v) => *v = value,
            None => {
                self.0.insert(key, value);
            }
        };
    }

    pub fn insert_if_not_exists(&mut self, key: String, value: String) {
        match self.get_mut(&key) {
            Some(_) => {}
            None => {
                self.0.insert(key, value);
            }
        };
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProtoHeader {
    pub key: String,
    pub value: String,
}

impl ProtoHeader {
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

        if total_read == 2 {
            //This is the end of the header section
            return (2, Ok(None));
        }

        let original_data = &data[..b_idx];

        let original_data = String::from_utf8_lossy(original_data);
        let data = original_data.trim();
        let split_index = match data.find(':') {
            Some(idx) => idx,
            None => {
                return (0, Err(Error::MalFormedHeader(data.to_string())));
            }
        };

        let (key, value) = data.split_at(split_index);
        //Remove ":"
        let value = &value[1..];

        if value.len() < 1 || key.len() < 1 {
            return (0, Err(Error::MalFormedHeader(original_data.to_string())));
        }
        if key.chars().last() == Some(' ') {
            return (0, Err(Error::MalFormedHeader(original_data.to_string())));
        }
        let key = key.trim();
        let value = value.trim();

        for c in key.chars() {
            if !Self::is_token(c) {
                return (0, Err(Error::MalFormedHeader(original_data.to_string())));
            }
        }

        return (
            total_read,
            Ok(Some(Self {
                key: key.to_lowercase(),
                value: value.to_string(),
            })),
        );
    }

    fn is_token(c: char) -> bool {
        let is_alpha_numeric = c.is_alphanumeric();
        if !is_alpha_numeric
            && !(c == '!')
            && !(c == '#')
            && !(c == '$')
            && !(c == '%')
            && !(c == '&')
            && !(c == '\'')
            && !(c == '*')
            && !(c == '+')
            && !(c == '-')
            && !(c == '.')
            && !(c == '^')
            && !(c == '_')
            && !(c == '`')
            && !(c == '|')
            && !(c == '~')
        {
            return false;
        }
        return true;
    }
}

#[cfg(test)]
mod test {
    use crate::{error::Error, header::ProtoHeader};

    #[test]
    fn test_valid_header() {
        let data = "Host: localhost:42069\r\n\r\n";
        let (b_read, result) = ProtoHeader::new_from_bytes(data.as_bytes());
        assert_eq!(b_read, 23);
        assert!(result.is_ok());
        let proto_header = result.unwrap();
        assert!(proto_header.is_some());
        let proto_header = proto_header.unwrap();
        assert_eq!(proto_header.key, "host");
        assert_eq!(proto_header.value, "localhost:42069");
    }

    #[test]
    fn test_invalid_spacing_header() {
        let data = "       Host : localhost:42069       \r\n\r\n";
        let (b_read, result) = ProtoHeader::new_from_bytes(data.as_bytes());
        assert_eq!(b_read, 0);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err,
            Error::MalFormedHeader("       Host : localhost:42069       ".to_string())
        );
    }

    #[test]
    fn test_invalid_token_on_key_header() {
        let data = "H©st: localhost:42069\r\n\r\n";
        let (b_read, result) = ProtoHeader::new_from_bytes(data.as_bytes());
        assert_eq!(b_read, 0);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err,
            Error::MalFormedHeader("H©st: localhost:42069".to_string())
        );
    }
}
