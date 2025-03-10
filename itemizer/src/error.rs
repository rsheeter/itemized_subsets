use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    TBD,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::TBD => f.write_str("TBD"),
        }
    }
}