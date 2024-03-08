use std::{fmt::Display, str::FromStr};

use lazy_static::lazy_static;
use regex::Regex;
use snafu::{ensure, ResultExt, Snafu};

use crate::{
    util::{consume_digits, consume_start, ConsumeError},
    Level, ParseLevelError,
};

lazy_static! {
    /// This matches one or more DNS labels separated by a dot.
    static ref DNS_SUBDOMAIN_REGEX: Regex =
        Regex::new(r"^(?:\.?[a-z0-9][a-z0-9-]{0,61}[a-z0-9])+$").unwrap();

    /// This matches a single DNS label.
    static ref DNS_LABEL_REGEX: Regex = Regex::new(r"^(?:[a-z0-9][a-z0-9-]{0,61}[a-z0-9])+$").unwrap();
}

#[derive(Debug, PartialEq, Snafu)]
pub enum VersionParseError {
    #[snafu(display("invalid version format. Input is empty, contains non-ASCII characters or contains more than 63 characters"))]
    InvalidFormat,

    #[snafu(display("failed to parse major version"))]
    ParseMajorVersion { source: ConsumeError },

    #[snafu(display("failed to parse version level"))]
    ParseLevel { source: ParseLevelError },
}

/// A Kubernetes resource version with the `v<MAJOR>(beta/alpha<LEVEL>)`
/// format, for example `v1`, `v2beta1` or `v1alpha2`.
///
/// The version must follow the DNS label format defined [here][1].
///
/// ### See
///
/// - <https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#api-conventions>
/// - <https://kubernetes.io/docs/reference/using-api/#api-versioning>
///
/// [1]: https://github.com/kubernetes/design-proposals-archive/blob/main/architecture/identifiers.md#definitions
#[derive(Debug)]
pub struct Version {
    pub major: u64,
    pub level: Option<Level>,
}

impl FromStr for Version {
    type Err = VersionParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if !input.is_ascii() {
            todo!()
        }

        // First we rule out any invalid version string from a format
        // point-of-view. The format defines that the version string must be
        // an alphanumeric (a-z, and 0-9) string, with a maximum length of 63
        // characters, with the '-' character allowed anywhere except the first
        // or last character.
        ensure!(DNS_LABEL_REGEX.is_match(input), InvalidFormatSnafu);

        // Ensure the string starts with a `v`.
        let input = consume_start(input).context(ParseMajorVersionSnafu)?;
        // Consume the major version number
        let (major, input) = consume_digits(&input[1..]).context(ParseMajorVersionSnafu)?;

        if input.is_empty() {
            return Ok(Self { level: None, major });
        }

        let level = Level::from_str(input).context(ParseLevelSnafu)?;

        Ok(Self {
            level: Some(level),
            major,
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

    // #[rstest]
    // #[case("v1gamma12", VersionParseError::ParseLevel { source: ParseLevelError::InvalidLevel })]
    // #[case("v1betä1", VersionParseError::InvalidFormat)]
    // #[case("1beta1", VersionParseError::InvalidStart)]
    // #[case("", VersionParseError::InvalidFormat)]
    // #[case("v0", VersionParseError::LeadingZero)]
    // fn invalid_version(#[case] input: &str, #[case] error: VersionParseError) {
    //     let err = Version::from_str(input).unwrap_err();
    //     assert_eq!(err, error)
    // }
}
