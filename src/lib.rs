use std::{
    iter::Peekable,
    str::{Bytes, FromStr},
};

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

    #[snafu(display("integer overflow"))]
    IntegerOverflow,

    #[snafu(display("leading zero"))]
    LeadingZero,

    #[snafu(display("expected \"beta\" or \"alpha\" identifier, got {identifier:?}"))]
    IllegalIdentifier { identifier: String },
}

/// A Kubernetes resource version with the `v<MAJOR>(beta/alpha<VERSION>)` format.
#[derive(Debug)]
pub struct Version {
    pub major: u64,
    pub minor: Option<Minor>,
}

#[derive(Debug)]
pub enum Minor {
    Beta(u64),
    Alpha(u64),
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        ensure!(!input.is_empty(), EmptySnafu);

        let length = input.len();
        ensure!(length <= 63, IllegalLengthSnafu { length });

        ensure!(input.is_ascii(), NonAsciiSnafu);

        // TODO (Techassi): Handle this error
        let mut bytes = input.bytes().peekable();
        let first = bytes.next().unwrap();

        ensure!(
            first == b'v',
            IllegalCharSnafu {
                character: first,
                index: 0usize
            }
        );

        let major = extract_digits(&mut bytes)?;

        if bytes.len() == 0 {
            return Ok(Self { major, minor: None });
        }

        let minor = match extract_chars(&mut bytes).as_str() {
            "beta" => Minor::Beta(extract_digits(&mut bytes)?),
            "alpha" => Minor::Alpha(extract_digits(&mut bytes)?),
            identifier => return IllegalIdentifierSnafu { identifier }.fail(),
        };

        Ok(Self {
            major,
            minor: Some(minor),
        })
    }
}

fn extract_digits(bytes: &mut Peekable<Bytes<'_>>) -> Result<u64, Error> {
    let mut at_start = false;
    let mut number = 0u64;

    while let Some(digit) = bytes.next_if(|b| (*b >= b'0' && *b <= b'9')) {
        if at_start && digit == b'0' {
            return LeadingZeroSnafu.fail();
        }
        at_start = false;

        match number
            .checked_mul(10)
            .and_then(|v| v.checked_add((digit - b'0') as u64))
        {
            Some(sum) => number = sum,
            None => return IntegerOverflowSnafu.fail(),
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
    #[case("v1alpha1")]
    #[case("v1beta1")]
    #[case("v1")]
    fn test_from_str(#[case] input: &str) {
        let version = Version::from_str(input).unwrap();
        println!("{version:?}")
    }
}
