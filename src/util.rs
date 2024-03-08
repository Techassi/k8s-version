use snafu::{ensure, OptionExt, Snafu};

#[derive(Debug, PartialEq, Snafu)]
pub enum ConsumeError {
    #[snafu(display("unexpected character"))]
    UnexpectedCharacter { char: String },

    #[snafu(display("version number cannot start with a leading zero"))]
    LeadingZero,

    #[snafu(display("integer overflow"))]
    IntegerOverflow,
}

pub(crate) fn consume_start(input: &str) -> Result<&str, ConsumeError> {
    ensure!(
        input.starts_with('v'),
        UnexpectedCharacterSnafu {
            char: input[..1].to_string()
        }
    );
    Ok(&input[1..])
}

pub(crate) fn consume_digits(input: &str) -> Result<(u64, &str), ConsumeError> {
    let mut iter = input.bytes().enumerate().peekable();
    let mut number = 0u64;
    let mut consumed = 0;

    while let Some((index, digit)) = iter.next_if(|(_, b)| (*b >= b'0' && *b <= b'9')) {
        ensure!(!(index == 0 && digit == b'0'), LeadingZeroSnafu);

        number = number
            .checked_mul(10)
            .and_then(|v| v.checked_add((digit - b'0') as u64))
            .context(IntegerOverflowSnafu)?;

        consumed += 1;
    }

    if consumed > 0 {
        Ok((number, &input[consumed..]))
    } else {
        // Unexpected end
        todo!()
    }
}

pub(crate) fn consume_chars(input: &str) -> Result<(String, &str), ConsumeError> {
    let mut iter = input.bytes().peekable();
    let mut string = String::new();
    let mut consumed = 0;

    while let Some(char) = iter.next_if(|b| (*b >= b'a' && *b <= b'z')) {
        string.push(char as char);
        consumed += 1;
    }

    Ok((string, &input[consumed..]))
}
