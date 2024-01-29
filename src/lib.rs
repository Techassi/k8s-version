use std::{
    fmt::Display,
    iter::Peekable,
    str::{Bytes, FromStr},
};

mod error;

use error::Error;

pub struct ApiVersion {
    pub group: Option<String>,
    pub version: Version,
}

impl FromStr for ApiVersion {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.group {
            Some(group) => write!(f, "{}/{}", group, self.version),
            None => write!(f, "{}", self.version),
        }
    }
}

/// A Kubernetes resource version with the `v<MAJOR>(beta/alpha<MINOR>)`
/// format, for example `v1`, `v2beta1` or `v1alpha2`.
#[derive(Debug)]
pub struct Version {
    pub major: u64,
    pub minor: Option<Minor>,
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.is_empty() {
            return Err(Error::Empty);
        }

        let length = input.len();
        if length > 63 {
            return Err(Error::IllegalLength { length });
        }

        if !input.is_ascii() {
            return Err(Error::NonAscii);
        }

        // TODO (Techassi): Handle this error
        let mut bytes = input.bytes().peekable();
        let first = bytes.next().unwrap();

        if first != b'v' {
            return Err(Error::IllegalChar {
                character: first as char,
                index: 0,
            });
        }

        let major = extract_digits(&mut bytes)?;

        if bytes.len() == 0 {
            return Ok(Self { major, minor: None });
        }

        let minor = match extract_chars(&mut bytes).as_str() {
            "beta" => Minor::Beta(extract_digits(&mut bytes)?),
            "alpha" => Minor::Alpha(extract_digits(&mut bytes)?),
            identifier => {
                return Err(Error::IllegalIdentifier {
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

fn extract_digits(bytes: &mut Peekable<Bytes<'_>>) -> Result<u64, Error> {
    let mut at_start = true;
    let mut number = 0u64;

    while let Some(digit) = bytes.next_if(|b| (*b >= b'0' && *b <= b'9')) {
        if at_start && digit == b'0' {
            return Err(Error::LeadingZero);
        }
        at_start = false;

        match number
            .checked_mul(10)
            .and_then(|v| v.checked_add((digit - b'0') as u64))
        {
            Some(sum) => number = sum,
            None => return Err(Error::IntegerOverflow),
        }
    }

    Ok(number)
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
    fn valid_from_str(#[case] input: &str) {
        let version = Version::from_str(input).unwrap();
        assert_eq!(version.to_string(), input);
    }

    #[rstest]
    #[case("v1gamma12", Error::IllegalIdentifier { identifier: "gamma".into() })]
    #[case("1beta1", Error::IllegalChar { character: '1', index: 0 })]
    #[case("v1beta0", Error::LeadingZero)]
    #[case("v1betä1", Error::NonAscii)]
    #[case("", Error::Empty)]
    fn invalid_from_str(#[case] input: &str, #[case] error: Error) {
        let err = Version::from_str(input).unwrap_err();
        assert_eq!(err, error)
    }
}
