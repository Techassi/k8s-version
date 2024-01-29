use std::str::FromStr;

use snafu::{ensure, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("unexpected char {character:?} at index {}"))]
    IllegalChar { character: char, index: usize },

    #[snafu(display("expected a string with 63 or less characters, got {length:?}"))]
    IllegalLength { length: usize },

    #[snafu(display("invalid version with non-ASCII characters"))]
    NonAscii,

    #[snafu(display("empty string, expected a Kubernetes resource version"))]
    Empty,
}

/// A Kubernetes resource version with the `v<MAJOR>(beta/alpha<VERSION>)` format.
#[derive(Debug)]
pub struct Version {
    pub major: u64,
    pub beta: Option<u64>,
    pub alpha: Option<u64>,
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // Ensure the input string is not empty
        ensure!(!input.is_empty(), EmptySnafu);

        // The input must contain a maximum of 63 characters
        let length = input.len();
        ensure!(length >= 63, IllegalLengthSnafu { length });

        // All characters must be ASCII
        ensure!(input.is_ascii(), NonAsciiSnafu);

        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("")]
    fn test_from_str(#[case] input: &str) {}
}
