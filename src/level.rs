use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
    str::FromStr,
};

use snafu::{ResultExt, Snafu};

use crate::util::{consume_chars, consume_digits, ConsumeError};

#[derive(Debug, PartialEq, Snafu)]
pub enum ParseLevelError {
    #[snafu(display("failed to parse level identifier"))]
    ParseIdentifier { source: ConsumeError },

    #[snafu(display("failed to parse level version"))]
    ParseVersion { source: ConsumeError },

    #[snafu(display("unknown level identifier"))]
    UnknownIdentifier,
}

/// A minor Kubernetes resource version with the `beta/alpha<VERSION>` format.
#[derive(Debug, PartialEq)]
pub enum Level {
    Beta(u64),
    Alpha(u64),
}

impl FromStr for Level {
    type Err = ParseLevelError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (identifier, input) = consume_chars(input).context(ParseIdentifierSnafu)?;

        let level = match identifier.as_str() {
            "beta" => Level::Beta(consume_digits(input).context(ParseVersionSnafu)?.0),
            "alpha" => Level::Alpha(consume_digits(input).context(ParseVersionSnafu)?.0),
            _ => return UnknownIdentifierSnafu.fail(),
        };

        Ok(level)
    }
}

impl PartialOrd for Level {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Level::Beta(sb) => match other {
                Level::Beta(ob) => sb.partial_cmp(ob),
                Level::Alpha(_) => Some(Ordering::Greater),
            },
            Level::Alpha(sa) => match other {
                Level::Beta(_) => Some(Ordering::Less),
                Level::Alpha(oa) => sa.partial_cmp(oa),
            },
        }
    }
}

impl<T> Add<T> for Level
where
    T: Into<u64>,
{
    type Output = Level;

    fn add(self, rhs: T) -> Self::Output {
        match self {
            Level::Beta(b) => Level::Beta(b + rhs.into()),
            Level::Alpha(a) => Level::Alpha(a + rhs.into()),
        }
    }
}

impl<T> AddAssign<T> for Level
where
    T: Into<u64>,
{
    fn add_assign(&mut self, rhs: T) {
        match self {
            Level::Beta(b) => *b + rhs.into(),
            Level::Alpha(a) => *a + rhs.into(),
        };
    }
}

impl<T> Sub<T> for Level
where
    T: Into<u64>,
{
    type Output = Level;

    fn sub(self, rhs: T) -> Self::Output {
        match self {
            Level::Beta(b) => Level::Beta(b - rhs.into()),
            Level::Alpha(a) => Level::Alpha(a - rhs.into()),
        }
    }
}

impl<T> SubAssign<T> for Level
where
    T: Into<u64>,
{
    fn sub_assign(&mut self, rhs: T) {
        match self {
            Level::Beta(b) => *b - rhs.into(),
            Level::Alpha(a) => *a - rhs.into(),
        };
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Beta(beta) => write!(f, "beta{}", beta),
            Level::Alpha(alpha) => write!(f, "alpha{}", alpha),
        }
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Level::Beta(1), Level::Alpha(1), Ordering::Greater)]
    #[case(Level::Alpha(1), Level::Beta(1), Ordering::Less)]
    #[case(Level::Alpha(2), Level::Alpha(1), Ordering::Greater)]
    #[case(Level::Alpha(2), Level::Alpha(2), Ordering::Equal)]
    #[case(Level::Alpha(1), Level::Alpha(2), Ordering::Less)]
    #[case(Level::Beta(2), Level::Beta(1), Ordering::Greater)]
    #[case(Level::Beta(2), Level::Beta(2), Ordering::Equal)]
    #[case(Level::Beta(1), Level::Beta(2), Ordering::Less)]
    fn partial_ord_level(#[case] input: Level, #[case] other: Level, #[case] expected: Ordering) {
        assert_eq!(input.partial_cmp(&other), Some(expected))
    }
}
