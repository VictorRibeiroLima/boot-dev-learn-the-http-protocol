use std::{
    collections::HashMap,
    fmt::{self, Display},
    ops::{Deref, DerefMut, Index},
    str::Chars,
};

use crate::{error::Error, Result, SEPARATOR};

#[derive(Debug, Default)]
pub struct Headers(pub HashMap<String, String>);

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

        let data = &data[..b_idx];

        let data = String::from_utf8_lossy(data);
        let data = data.trim();
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
            return (0, Err(Error::MalFormedHeader(data.to_string())));
        }
        if key.chars().last() == Some(' ') {
            return (0, Err(Error::MalFormedHeader(data.to_string())));
        }
        let key = key.trim();
        let value = value.trim();

        for c in key.chars() {
            if !Self::is_token(c) {
                return (0, Err(Error::MalFormedHeader(data.to_string())));
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
