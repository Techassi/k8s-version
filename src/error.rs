use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum Error {
    IllegalChar { character: char, index: usize },
    IllegalIdentifier { identifier: String },
    IllegalLength { length: usize },
    IntegerOverflow,
    LeadingZero,
    NonAscii,
    Empty,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IllegalChar { character, index } => {
                write!(f, "unexpected character {character:?} at index {index}")
            }
            Error::IllegalIdentifier { identifier } => write!(
                f,
                "unexpected minor identifier {identifier:?}, expected \"beta\" or \"alpha\""
            ),
            Error::IllegalLength { length } => write!(
                f,
                "expected a string with 63 characters or less, got {length:?}"
            ),
            Error::IntegerOverflow => write!(f, "u64 integer overflow"),
            Error::LeadingZero => write!(f, "unexpected leading zero in version number"),
            Error::NonAscii => write!(f, "unexpected non-ascii character"),
            Error::Empty => write!(f, "empty string, expected a Kubernetes resource version"),
        }
    }
}

impl std::error::Error for Error {}
