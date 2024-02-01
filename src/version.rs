use std::{
    fmt::Display,
    iter::Peekable,
    str::{Bytes, FromStr},
};

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// This matches one or more DNS labels separated by a dot.
    static ref DNS_SUBDOMAIN_REGEX: Regex =
        Regex::new(r"^(?:\.?[a-z0-9][a-z0-9-]{0,61}[a-z0-9])+$").unwrap();

    /// This matches a single DNS label.
    static ref DNS_LABEL_REGEX: Regex = Regex::new(r"^(?:[a-z0-9][a-z0-9-]{0,61}[a-z0-9])+$").unwrap();
}

pub const ERROR_REGEX_FAILED_MESSAGE: &'static str =
    "string is empty, contains non-ASCII characters or contains more than 63 characters";
pub const ERROR_ILLEGAL_PREFIX_MESSAGE: &'static str = "version must start with \"v\"";

#[derive(Debug, PartialEq)]
pub enum VersionParseError {
    IllegalFormat { message: &'static str },
    IllegalLevel { level: String },
    IntegerOverflow,
    LeadingZero,
}

impl Display for VersionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionParseError::IllegalLevel { level } => write!(
                f,
                "unexpected level identifier {level:?}, expected \"beta\" or \"alpha\""
            ),
            VersionParseError::IntegerOverflow => write!(f, "u64 integer overflow"),
            VersionParseError::LeadingZero => {
                write!(f, "unexpected leading zero in version number")
            }
            VersionParseError::IllegalFormat { message } => {
                write!(f, "illegal version format: {message}")
            }
        }
    }
}

impl std::error::Error for VersionParseError {}

/// A Kubernetes resource version with the `v<MAJOR>(beta/alpha<LEVEL>)`
/// format, for example `v1`, `v2beta1` or `v1alpha2`.
///
/// The `<VERSION>` string must follow the DNS label format defined
/// [here](https://github.com/kubernetes/design-proposals-archive/blob/main/architecture/identifiers.md#definitions).
///
/// ### See
///
/// - <https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#api-conventions>
/// - <https://kubernetes.io/docs/reference/using-api/#api-versioning>
#[derive(Debug)]
pub struct Version {
    pub major: u64,
    pub level: Option<Level>,
}

impl FromStr for Version {
    type Err = VersionParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // First we rule out any invalid version string from a format
        // point-of-view. The format defines that the version string must be
        // an alphanumeric (a-z, and 0-9) string, with a maximum length of 63
        // characters, with the '-' character allowed anywhere except the first
        // or last character.
        if !DNS_LABEL_REGEX.is_match(input) {
            return Err(VersionParseError::IllegalFormat {
                message: ERROR_REGEX_FAILED_MESSAGE,
            });
        }

        // This can never error because we validated the input is non-empty
        // and thus contains at least one byte.
        let mut bytes = input.bytes().peekable();
        let first = bytes.next().unwrap();

        // Each version string must start with a lower case 'v'.
        if first != b'v' {
            return Err(VersionParseError::IllegalFormat {
                message: ERROR_ILLEGAL_PREFIX_MESSAGE,
            });
        }

        // Next, consume the major version number.
        let (major, consumed) = extract_digits(&mut bytes)?;

        // If we consumed no characters, we know that the version string
        // only contained a single 'v' without any major version number.
        if consumed == 0 {
            return Err(VersionParseError::IllegalFormat {
                message: "failed to parse major version number".into(),
            });
        }

        // If we are already at the end, the version string only contains a
        // major version (without a beta or alpha level).
        if bytes.len() == 0 {
            return Ok(Self { major, level: None });
        }

        // Extract the optional beta or alpha level with the version number.
        // TODO (Techassi): Verify we consumed the level version number
        let level = match extract_chars(&mut bytes).as_str() {
            "beta" => Level::Beta(extract_digits(&mut bytes)?.0),
            "alpha" => Level::Alpha(extract_digits(&mut bytes)?.0),
            level => {
                return Err(VersionParseError::IllegalLevel {
                    level: level.into(),
                })
            }
        };

        Ok(Self {
            major,
            level: Some(level),
        })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.level {
            Some(minor) => write!(f, "v{}{}", self.major, minor),
            None => write!(f, "v{}", self.major),
        }
    }
}

impl Version {
    pub fn new(major: u64, minor: Option<Level>) -> Self {
        Self {
            major,
            level: minor,
        }
    }
}

/// A minor Kubernetes resource version with the `beta/alpha<VERSION>` format.
#[derive(Debug)]
pub enum Level {
    Beta(u64),
    Alpha(u64),
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Beta(beta) => write!(f, "beta{}", beta),
            Level::Alpha(alpha) => write!(f, "alpha{}", alpha),
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
    #[case("v1betä1", VersionParseError::IllegalFormat { message: ERROR_REGEX_FAILED_MESSAGE })]
    #[case("v1gamma12", VersionParseError::IllegalLevel { level: "gamma".into() })]
    #[case("1beta1", VersionParseError::IllegalFormat { message: "" })]
    #[case("", VersionParseError::IllegalFormat { message: "" })]
    #[case("v0", VersionParseError::LeadingZero)]
    fn invalid_version(#[case] input: &str, #[case] error: VersionParseError) {
        let err = Version::from_str(input).unwrap_err();
        assert_eq!(err, error)
    }
}
