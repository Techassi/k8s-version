use std::{
    fmt::Display,
    iter::Peekable,
    str::{Bytes, FromStr},
};

#[derive(Debug, PartialEq)]
pub enum VersionParseError {
    IllegalIdentifier { identifier: String },
    IllegalPrefix { character: char },
    IllegalLength { length: usize },
    IntegerOverflow,
    LeadingZero,
    NonAscii,
    Empty,
}

impl Display for VersionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionParseError::IllegalPrefix { character } => {
                write!(
                    f,
                    "unexpected prefix character {character:?}, expected \"v\""
                )
            }
            VersionParseError::IllegalIdentifier { identifier } => write!(
                f,
                "unexpected minor identifier {identifier:?}, expected \"beta\" or \"alpha\""
            ),
            VersionParseError::IllegalLength { length } => write!(
                f,
                "expected a string with 63 characters or less, got {length:?}"
            ),
            VersionParseError::IntegerOverflow => write!(f, "u64 integer overflow"),
            VersionParseError::LeadingZero => {
                write!(f, "unexpected leading zero in version number")
            }
            VersionParseError::NonAscii => write!(f, "unexpected non-ascii character"),
            VersionParseError::Empty => {
                write!(f, "empty string, expected a Kubernetes resource version")
            }
        }
    }
}

impl std::error::Error for VersionParseError {}

// A Kubernetes resource version with the `v<MAJOR>(beta/alpha<MINOR>)`
/// format, for example `v1`, `v2beta1` or `v1alpha2`.
#[derive(Debug)]
pub struct Version {
    pub major: u64,
    pub minor: Option<Minor>,
}

impl FromStr for Version {
    type Err = VersionParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.is_empty() {
            return Err(VersionParseError::Empty);
        }

        let length = input.len();
        if length > 63 {
            return Err(VersionParseError::IllegalLength { length });
        }

        if !input.is_ascii() {
            return Err(VersionParseError::NonAscii);
        }

        // This can never error because we validated the input is non-empty
        // and thus contains at least one byte.
        let mut bytes = input.bytes().peekable();
        let first = bytes.next().unwrap();

        if first != b'v' {
            return Err(VersionParseError::IllegalPrefix {
                character: first as char,
            });
        }

        let (major, consumed) = extract_digits(&mut bytes)?;

        if consumed == 0 {
            // TODO (Techassi): Invalid version
        }

        if bytes.len() == 0 {
            return Ok(Self { major, minor: None });
        }

        let minor = match extract_chars(&mut bytes).as_str() {
            "beta" => Minor::Beta(extract_digits(&mut bytes)?.0),
            "alpha" => Minor::Alpha(extract_digits(&mut bytes)?.0),
            identifier => {
                return Err(VersionParseError::IllegalIdentifier {
                    identifier: identifier.into(),
                })
            }
        };

        Ok(Self {
            major,
            minor: Some(minor),
        })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.minor {
            Some(minor) => write!(f, "v{}{}", self.major, minor),
            None => write!(f, "v{}", self.major),
        }
    }
}

impl Version {
    pub fn new(major: u64, minor: Option<Minor>) -> Self {
        Self { major, minor }
    }
}

/// A minor Kubernetes resource version with the `beta/alpha<VERSION>` format.
#[derive(Debug)]
pub enum Minor {
    Beta(u64),
    Alpha(u64),
}

impl Display for Minor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Minor::Beta(beta) => write!(f, "beta{}", beta),
            Minor::Alpha(alpha) => write!(f, "alpha{}", alpha),
        }
    }
}

fn extract_digits(bytes: &mut Peekable<Bytes<'_>>) -> Result<(u64, usize), VersionParseError> {
    let mut at_start = true;
    let mut number = 0u64;
    let length = bytes.len();

    while let Some(digit) = bytes.next_if(|b| (*b >= b'0' && *b <= b'9')) {
        if at_start && digit == b'0' {
            return Err(VersionParseError::LeadingZero);
        }
        at_start = false;

        match number
            .checked_mul(10)
            .and_then(|v| v.checked_add((digit - b'0') as u64))
        {
            Some(sum) => number = sum,
            None => return Err(VersionParseError::IntegerOverflow),
        }
    }

    Ok((number, length - bytes.len()))
}

fn extract_chars(bytes: &mut Peekable<Bytes<'_>>) -> String {
    let mut identifier = String::new();

    while let Some(char) = bytes.next_if(|b| (*b >= b'a' && *b <= b'z')) {
        identifier.push(char as char)
    }

    identifier
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("v1alpha12")]
    #[case("v1alpha1")]
    #[case("v1beta1")]
    #[case("v1")]
    fn valid_version(#[case] input: &str) {
        let version = Version::from_str(input).unwrap();
        assert_eq!(version.to_string(), input);
    }

    #[rstest]
    #[case("v1gamma12", VersionParseError::IllegalIdentifier { identifier: "gamma".into() })]
    #[case("1beta1", VersionParseError::IllegalPrefix { character: '1' })]
    #[case("v1beta0", VersionParseError::LeadingZero)]
    #[case("v1betä1", VersionParseError::NonAscii)]
    #[case("", VersionParseError::Empty)]
    fn invalid_version(#[case] input: &str, #[case] error: VersionParseError) {
        let err = Version::from_str(input).unwrap_err();
        assert_eq!(err, error)
    }
}
