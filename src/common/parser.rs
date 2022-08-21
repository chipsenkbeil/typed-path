pub type ParseResult<'a, T> = Result<(ParseInput<'a>, T), ParseError>;
pub type ParseInput<'a> = &'a [u8];
pub type ParseError = &'static str;

macro_rules! any_of {
    ($lt:lifetime, $($parser:expr),+ $(,)?) => {
        |input: $crate::parser::ParseInput <$lt>| {
            $(
                if let Ok((input, value)) = $parser(input) {
                    return Ok((input, value));
                }
            )+

            Err("No parser succeeded")
        }
    };
}

/// Map a parser's result
pub fn map<'a, T, U>(
    parser: impl Fn(ParseInput<'a>) -> ParseResult<'a, T>,
    f: impl Fn(T) -> U,
) -> impl Fn(ParseInput<'a>) -> ParseResult<'a, U> {
    move |input: ParseInput| {
        let (input, value) = parser(input)?;
        Ok((input, f(value)))
    }
}

/// Execute three parsers in a row, failing if any fails, and returns first and third parsers' results
pub fn divided<'a, T1, T2, T3>(
    left: impl Fn(ParseInput<'a>) -> ParseResult<'a, T1>,
    middle: impl Fn(ParseInput<'a>) -> ParseResult<'a, T2>,
    right: impl Fn(ParseInput<'a>) -> ParseResult<'a, T3>,
) -> impl Fn(ParseInput<'a>) -> ParseResult<'a, (T1, T3)> {
    move |input: ParseInput| {
        let (input, v1) = left(input)?;
        let (input, _) = middle(input)?;
        let (input, v3) = right(input)?;
        Ok((input, (v1, v3)))
    }
}

/// Execute two parsers in a row, failing if either fails, and returns second parser's result
pub fn prefixed<'a, T1, T2>(
    prefix: impl Fn(ParseInput<'a>) -> ParseResult<'a, T1>,
    parser: impl Fn(ParseInput<'a>) -> ParseResult<'a, T2>,
) -> impl Fn(ParseInput<'a>) -> ParseResult<'a, T2> {
    move |input: ParseInput| {
        let (input, _) = prefix(input)?;
        let (input, value) = parser(input)?;
        Ok((input, value))
    }
}

/// Execute two parsers in a row, failing if either fails, and returns first parser's result
pub fn suffixed<'a, T1, T2>(
    parser: impl Fn(ParseInput<'a>) -> ParseResult<'a, T1>,
    suffix: impl Fn(ParseInput<'a>) -> ParseResult<'a, T2>,
) -> impl Fn(ParseInput<'a>) -> ParseResult<'a, T1> {
    move |input: ParseInput| {
        let (input, value) = parser(input)?;
        let (input, _) = suffix(input)?;
        Ok((input, value))
    }
}

/// Execute a parser, returning Some(value) if succeeds and None if fails
pub fn maybe<'a, T>(
    parser: impl Fn(ParseInput<'a>) -> ParseResult<'a, T>,
) -> impl Fn(ParseInput<'a>) -> ParseResult<'a, Option<T>> {
    move |input: ParseInput| match parser(input) {
        Ok((input, value)) => Ok((input, Some(value))),
        Err(_) => Ok((input, None)),
    }
}

/// Takes while the parser returns true, returning a collection of parser results, or failing if
/// the parser did not succeed at least once
pub fn one_or_more<'a, T>(
    parser: impl Fn(ParseInput<'a>) -> ParseResult<'a, T>,
) -> impl Fn(ParseInput<'a>) -> ParseResult<'a, Vec<T>> {
    move |input: ParseInput| {
        let mut results = Vec::new();
        let mut next = Some(input);
        while let Some(input) = next.take() {
            match parser(input) {
                Ok((input, value)) => {
                    next = Some(input);
                    results.push(value);
                }
                Err(_) => {
                    next = Some(input);
                }
            }
        }

        if results.is_empty() {
            return Err("Parser failed to suceed once");
        }

        Ok((next.unwrap(), results))
    }
}

/// Same as [`one_or_more`], but won't fail if the parser never succeeds
///
/// ### Note
///
/// This will ALWAYS succeed since it will just return an empty collection on failure.
/// Be careful to not get stuck in an infinite loop here!
pub fn zero_or_more<'a, T>(
    parser: impl Fn(ParseInput<'a>) -> ParseResult<'a, T>,
) -> impl Fn(ParseInput<'a>) -> ParseResult<'a, Vec<T>> {
    let parser = maybe(one_or_more(parser));

    move |input: ParseInput| {
        let (input, results) = parser(input)?;
        Ok((input, results.unwrap_or_default()))
    }
}

/// Takes until `predicate` returns true, failing if nothing parsed
pub fn take_until_byte(
    predicate: impl Fn(u8) -> bool,
) -> impl Fn(ParseInput) -> ParseResult<ParseInput> {
    move |input: ParseInput| {
        if input.is_empty() {
            return Err("Empty input");
        }

        let (input, value) = match input.iter().enumerate().find(|(_, b)| predicate(**b)) {
            // Position represents the first character (at boundary) that is not alphanumeric
            Some((i, _)) => (&input[i..], &input[..i]),

            // No position means that the remainder of the str was alphanumeric
            None => (b"".as_slice(), input),
        };

        if value.is_empty() {
            return Err("Predicate immediately returned true");
        }

        Ok((input, value))
    }
}

/// Takes `cnt` bytes, failing if not enough bytes are available
pub fn take(cnt: usize) -> impl Fn(ParseInput) -> ParseResult<ParseInput> {
    move |input: ParseInput| match input.iter().enumerate().nth(cnt) {
        Some((i, _)) => Ok((&input[i..], &input[..i])),
        None => Err("Not enough bytes"),
    }
}

/// Parse multiple bytes, failing if they do not match `bytes` or there are not enough bytes
pub fn bytes<'a>(bytes: &[u8]) -> impl Fn(ParseInput<'a>) -> ParseResult<&'a [u8]> + '_ {
    move |input: ParseInput<'a>| {
        if input.is_empty() {
            return Err("Empty input");
        } else if input.len() < bytes.len() {
            return Err("Not enough bytes");
        }

        if input.starts_with(bytes) {
            Ok((&input[bytes.len()..], &input[..bytes.len()]))
        } else {
            Err("Wrong bytes")
        }
    }
}

/// Parse a single byte, failing if it does not match `byte`
pub fn byte(byte: u8) -> impl Fn(ParseInput) -> ParseResult<u8> {
    move |input: ParseInput| {
        if input.is_empty() {
            return Err("Empty input");
        }

        if input.starts_with(&[byte]) {
            Ok((&input[1..], byte))
        } else {
            Err("Wrong byte")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parsers {
        use super::*;

        fn parse_fail(_: ParseInput) -> ParseResult<ParseInput> {
            Err("bad parser")
        }

        fn take_all(input: ParseInput) -> ParseResult<ParseInput> {
            Ok((b"", input))
        }

        mod prefixed {
            use super::*;

            #[test]
            fn should_fail_if_prefix_parser_fails() {
                let _ = prefixed(parse_fail, take_all)(b"abc").unwrap_err();
            }

            #[test]
            fn should_fail_if_main_parser_fails() {
                let _ = prefixed(take(1), parse_fail)(b"abc").unwrap_err();
            }

            #[test]
            fn should_return_value_of_main_parser_when_succeeds() {
                let (s, value) = prefixed(take(1), take(1))(b"abc").unwrap();
                assert_eq!(s, b"c");
                assert_eq!(value, b"b");
            }
        }

        mod maybe {
            use super::*;

            #[test]
            fn should_return_some_value_if_wrapped_parser_succeeds() {
                let (s, value) = maybe(take(2))(b"abc").unwrap();
                assert_eq!(s, b"c");
                assert_eq!(value, Some(b"ab".as_slice()));
            }

            #[test]
            fn should_return_none_if_wrapped_parser_fails() {
                let (s, value) = maybe(parse_fail)(b"abc").unwrap();
                assert_eq!(s, b"abc");
                assert_eq!(value, None);
            }
        }

        mod take_util {
            use super::*;

            #[test]
            fn should_consume_until_predicate_matches() {
                let (s, text) = take_until_byte(|c| c == b'b')(b"abc").unwrap();
                assert_eq!(s, b"bc");
                assert_eq!(text, b"a");
            }

            #[test]
            fn should_consume_completely_if_predicate_never_matches() {
                let (s, text) = take_until_byte(|c| c == b'z')(b"abc").unwrap();
                assert_eq!(s, b"");
                assert_eq!(text, b"abc");
            }

            #[test]
            fn should_fail_if_nothing_consumed() {
                let _ = take_until_byte(|c| c == b'a')(b"abc").unwrap_err();
            }

            #[test]
            fn should_fail_if_input_is_empty() {
                let _ = take_until_byte(|c| c == b'a')(b"").unwrap_err();
            }
        }

        mod byte {
            use super::*;

            #[test]
            fn should_succeed_if_next_byte_matches() {
                let (s, c) = byte(b'a')(b"abc").unwrap();
                assert_eq!(s, b"bc");
                assert_eq!(c, b'a');
            }

            #[test]
            fn should_fail_if_next_byte_does_not_match() {
                let _ = byte(b'b')(b"abc").unwrap_err();
            }

            #[test]
            fn should_fail_if_input_is_empty() {
                let _ = byte(b'a')(b"").unwrap_err();
            }
        }
    }
}
