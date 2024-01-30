use std::{fmt::Display, str::FromStr};

use crate::{Version, VersionParseError};

#[derive(Debug, PartialEq)]
pub enum ApiVersionParseError {
    VersionParseError { source: VersionParseError },
}

impl Display for ApiVersionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for ApiVersionParseError {}

/// A Kubernetes API version with the `(<GROUP>/)<VERSION>` format, for example
/// `certificates.k8s.io/v1beta1`, `extensions/v1beta1` or `v1`.
pub struct ApiVersion {
    pub group: Option<String>,
    pub version: Version,
}

impl FromStr for ApiVersion {
    type Err = ApiVersionParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.split_once('/') {
            Some((group, version)) => {
                let version = Version::from_str(version)
                    .map_err(|err| ApiVersionParseError::VersionParseError { source: err })?;

                // TODO (Techassi): Validate group
                Ok(Self {
                    group: Some(group.to_string()),
                    version,
                })
            }
            None => {
                let version = Version::from_str(input)
                    .map_err(|err| ApiVersionParseError::VersionParseError { source: err })?;

                Ok(Self {
                    group: None,
                    version,
                })
            }
        }
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
