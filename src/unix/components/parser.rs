use crate::{
    common::parser::*,
    unix::{UnixComponent, CURRENT_DIR, DISALLOWED_FILENAME_BYTES, PARENT_DIR, SEPARATOR},
};

/// Parser to get [`UnixComponent`]s
///
/// ### Details
///
/// When parsing the path, there is a small amount of normalization:
///
/// Repeated separators are ignored, so a/b and a//b both have a and b as components.
///
/// Occurrences of . are normalized away, except if they are at the beginning of the path. For
/// example, a/./b, a/b/, a/b/. and a/b all have a and b as components, but ./a/b starts with an
/// additional CurDir component.
///
/// A trailing slash is normalized away, /a/b and /a/b/ are equivalent.
///
/// Note that no other normalization takes place; in particular, a/c and a/b/../c are distinct, to
/// account for the possibility that b is a symbolic link (so its parent isn’t a).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parser<'a> {
    input: &'a [u8],
    state: State,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum State {
    // If input is still at the beginning
    AtBeginning,

    // If input has moved passed the beginning
    NotAtBeginning,
}

impl State {
    #[inline]
    pub fn is_at_beginning(self) -> bool {
        matches!(self, Self::AtBeginning)
    }
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given `input`
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            state: State::AtBeginning,
        }
    }

    /// Returns true if the parser has more to parse
    pub fn has_more(&self) -> bool {
        !self.input.is_empty()
    }

    /// Returns the input remaining for the parser
    pub fn remaining(&self) -> &'a [u8] {
        self.input
    }

    /// Parses next component without advancing internal input pointer or adjusting state
    pub fn peek_front(&self) -> Result<UnixComponent<'a>, ParseError> {
        let (_, component) = parse_front(self.state)(self.input)?;
        Ok(component)
    }

    /// Parses next component, advancing an internal input pointer past the component
    pub fn next_front(&mut self) -> Result<UnixComponent<'a>, ParseError> {
        let (input, component) = parse_front(self.state)(self.input)?;
        self.input = input;
        self.state = State::NotAtBeginning;
        Ok(component)
    }

    /// Parses next component, advancing an internal input pointer past the component, but from the
    /// back of the input instead of the front
    pub fn next_back(&mut self) -> Result<UnixComponent<'a>, ParseError> {
        let (input, component) = parse_back(self.state)(self.input)?;
        self.input = input;
        Ok(component)
    }
}

fn parse_front(state: State) -> impl FnMut(ParseInput) -> ParseResult<UnixComponent> {
    move |input: ParseInput| match state {
        // If we are at the beginning, we want to allow for root directory and '.'
        State::AtBeginning => suffixed(
            any_of!('_, root_dir, parent_dir, cur_dir, normal),
            zero_or_more(separator),
        )(input),

        // If we are not at the beginning, then we only want to allow for '..' and file names
        State::NotAtBeginning => {
            // Skip any '.' and separators we encounter
            let (input, _) = take_while(any_of!('_, cur_dir, separator))(input)?;

            // Get the next component if we have one left
            suffixed(any_of!('_, parent_dir, normal), zero_or_more(separator))(input)
        }
    }
}

fn parse_back(state: State) -> impl FnMut(ParseInput) -> ParseResult<UnixComponent> {
    move |input: ParseInput| {
        let original_input = input;

        let is_sep = |b: u8| b == SEPARATOR as u8;

        // Keep track of whether we've already seen the current directory '.' before,
        // to avoid consuming '..'
        let mut last_seen_byte = b'\0';
        let mut is_cur_dir = |b: u8| {
            let cur_dir_byte = CURRENT_DIR[0];
            let valid = b == cur_dir_byte && last_seen_byte != cur_dir_byte;
            last_seen_byte = b;
            valid
        };

        // Skip any '.' and trailing separators we encounter
        let (input, _) = rtake_until_byte(|b| !is_sep(b) && !is_cur_dir(b))(input)?;

        // If at beginning and our resulting input is empty, this means that we only had '.' and
        // separators remaining, which means that we want to check the front instead for our
        // component since we are supporting '.' and root directory (which is our separator)
        if state.is_at_beginning() && input.is_empty() {
            let (_, component) = parse_front(state)(original_input)?;

            return Ok((b"", component));
        }

        // Otherwise, look for next separator in reverse so we can parse everything after it
        let (input, after_sep) = rtake_until_byte_1(|b| !is_sep(b))(input)?;

        // Parse the component, failing if we don't fully parse it
        let (_, component) = fully_consumed(any_of!('_, parent_dir, normal))(after_sep)?;

        Ok((input, component))
    }
}

fn root_dir(input: ParseInput) -> ParseResult<UnixComponent> {
    let (input, _) = separator(input)?;
    Ok((input, UnixComponent::RootDir))
}

fn cur_dir(input: ParseInput) -> ParseResult<UnixComponent> {
    let (input, _) = suffixed(bytes(CURRENT_DIR), any_of!('_, empty, peek(separator)))(input)?;
    Ok((input, UnixComponent::CurDir))
}

fn parent_dir(input: ParseInput) -> ParseResult<UnixComponent> {
    let (input, _) = suffixed(bytes(PARENT_DIR), any_of!('_, empty, peek(separator)))(input)?;
    Ok((input, UnixComponent::ParentDir))
}

fn normal(input: ParseInput) -> ParseResult<UnixComponent> {
    let (input, normal) = take_until_byte(|b| DISALLOWED_FILENAME_BYTES.contains(&b))(input)?;
    Ok((input, UnixComponent::Normal(normal)))
}

fn separator(input: ParseInput) -> ParseResult<()> {
    let (input, _) = byte(SEPARATOR as u8)(input)?;
    Ok((input, ()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sep(cnt: usize) -> Vec<u8> {
        let mut v = Vec::new();
        for _ in 0..cnt {
            v.push(SEPARATOR as u8);
        }
        v
    }

    #[test]
    fn should_support_parsing_single_component_from_front() {
        // Empty input fails
        Parser::new(b"").next_front().unwrap_err();

        // Unfinished consumption of input fails
        Parser::new(b"abc\0def").next_front().unwrap_err();

        // Supports parsing any component individually
        let mut parser = Parser::new(&[SEPARATOR as u8]);
        assert_eq!(parser.next_front(), Ok(UnixComponent::RootDir));
        assert_eq!(parser.remaining(), b"");
        assert!(parser.next_front().is_err());

        let mut parser = Parser::new(CURRENT_DIR);
        assert_eq!(parser.next_front(), Ok(UnixComponent::CurDir));
        assert_eq!(parser.remaining(), b"");
        assert!(parser.next_front().is_err());
        let mut parser = Parser::new(PARENT_DIR);
        assert_eq!(parser.next_front(), Ok(UnixComponent::ParentDir));
        assert_eq!(parser.remaining(), b"");
        assert!(parser.next_front().is_err());

        let mut parser = Parser::new(b"hello");
        assert_eq!(parser.next_front(), Ok(UnixComponent::Normal(b"hello")));
        assert_eq!(parser.remaining(), b"");
        assert!(parser.next_front().is_err());
    }

    #[test]
    fn should_support_parsing_from_multiple_components_from_front() {
        // Empty input fails
        parse_front(b"").unwrap_err();

        // Fails if starts with a null character
        parse_front(b"\0hello").unwrap_err();

        // Succeeds if finds a root dir
        let (input, mut components) = parse_front(&[SEPARATOR as u8]).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(UnixComponent::RootDir));
        assert_eq!(components.next(), None);

        // Multiple separators still just mean root
        let input = sep(2);
        let (input, mut components) = parse_front(&input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(UnixComponent::RootDir));
        assert_eq!(components.next(), None);

        // Succeeds even if there isn't a root
        //
        // E.g. a/b/c
        let input = &[
            &[b'a'],
            sep(1).as_slice(),
            &[b'b'],
            sep(1).as_slice(),
            &[b'c'],
        ]
        .concat();
        let (input, mut components) = parse_front(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(UnixComponent::Normal(b"a")));
        assert_eq!(components.next(), Some(UnixComponent::Normal(b"b")));
        assert_eq!(components.next(), Some(UnixComponent::Normal(b"c")));
        assert_eq!(components.next(), None);

        // Should support '.' at beginning of path
        //
        // E.g. ./b/c
        let input = &[
            CURRENT_DIR,
            sep(1).as_slice(),
            &[b'b'],
            sep(1).as_slice(),
            &[b'c'],
        ]
        .concat();
        let (input, mut components) = parse_front(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(UnixComponent::CurDir));
        assert_eq!(components.next(), Some(UnixComponent::Normal(b"b")));
        assert_eq!(components.next(), Some(UnixComponent::Normal(b"c")));
        assert_eq!(components.next(), None);

        // Should remove current dir from anywhere if not at beginning
        //
        // E.g. /./b/c -> /b/c
        // E.g. a/./c -> a/c
        let input = &[
            sep(1).as_slice(),
            CURRENT_DIR,
            sep(1).as_slice(),
            &[b'b'],
            sep(1).as_slice(),
            &[b'c'],
        ]
        .concat();
        let (input, mut components) = parse_front(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(UnixComponent::RootDir));
        assert_eq!(components.next(), Some(UnixComponent::Normal(b"b")));
        assert_eq!(components.next(), Some(UnixComponent::Normal(b"c")));
        assert_eq!(components.next(), None);

        let input = &[
            &[b'a'],
            sep(1).as_slice(),
            CURRENT_DIR,
            sep(1).as_slice(),
            &[b'c'],
        ]
        .concat();
        let (input, mut components) = parse_front(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(UnixComponent::Normal(b"a")));
        assert_eq!(components.next(), Some(UnixComponent::Normal(b"c")));
        assert_eq!(components.next(), None);

        // Should strip multiple separators and normalize '.'
        //
        // E.g. /////a///.//../// -> [ROOT, "a", CURRENT_DIR, PARENT_DIR]
        let input = &[
            sep(5).as_slice(),
            &[b'a'],
            sep(3).as_slice(),
            CURRENT_DIR,
            sep(2).as_slice(),
            PARENT_DIR,
            sep(3).as_slice(),
        ]
        .concat();
        let (input, mut components) = parse_front(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(UnixComponent::RootDir));
        assert_eq!(components.next(), Some(UnixComponent::Normal(b"a")));
        assert_eq!(components.next(), Some(UnixComponent::ParentDir));
        assert_eq!(components.next(), None);
    }

    mod helpers {
        use super::*;

        #[test]
        fn validate_root_dir() {
            // Empty input fails
            root_dir(b"").unwrap_err();

            // Not starting with root dir fails
            root_dir(&[b'a', SEPARATOR as u8]).unwrap_err();

            // Succeeds just on its own
            let (input, value) = root_dir(&[SEPARATOR as u8]).unwrap();
            assert_eq!(input, b"");
            assert_eq!(value, UnixComponent::RootDir);

            // Succeeds, taking only what it matches
            let (input, value) = root_dir(&[SEPARATOR as u8, b'a', SEPARATOR as u8]).unwrap();
            assert_eq!(input, &[b'a', SEPARATOR as u8]);
            assert_eq!(value, UnixComponent::RootDir);
        }

        #[test]
        fn validate_cur_dir() {
            // Empty input fails
            cur_dir(b"").unwrap_err();

            // Not starting with current dir fails
            cur_dir(&[&[b'a'], CURRENT_DIR].concat()).unwrap_err();

            // Succeeds just on its own
            let (input, value) = cur_dir(CURRENT_DIR).unwrap();
            assert_eq!(input, b"");
            assert_eq!(value, UnixComponent::CurDir);

            // Fails if more content after itself that is not a separator
            // E.g. .. will fail, .a will fail
            cur_dir(&[CURRENT_DIR, &[b'.']].concat()).unwrap_err();
            cur_dir(&[CURRENT_DIR, &[b'a']].concat()).unwrap_err();

            // Succeeds, taking only what it matches
            let input = &[CURRENT_DIR, &sep(1), CURRENT_DIR].concat();
            let (input, value) = cur_dir(input).unwrap();
            assert_eq!(input, &[&sep(1), CURRENT_DIR].concat());
            assert_eq!(value, UnixComponent::CurDir);
        }

        #[test]
        fn validate_parent_dir() {
            // Empty input fails
            parent_dir(b"").unwrap_err();

            // Not starting with parent dir fails
            parent_dir(&[&[b'a'], PARENT_DIR].concat()).unwrap_err();

            // Succeeds just on its own
            let (input, value) = parent_dir(PARENT_DIR).unwrap();
            assert_eq!(input, b"");
            assert_eq!(value, UnixComponent::ParentDir);

            // Fails if more content after itself that is not a separator
            // E.g. ... will fail, ..a will fail
            parent_dir(&[PARENT_DIR, &[b'.']].concat()).unwrap_err();
            parent_dir(&[PARENT_DIR, &[b'a']].concat()).unwrap_err();

            // Succeeds, taking only what it matches
            let input = &[PARENT_DIR, &sep(1), PARENT_DIR].concat();
            let (input, value) = parent_dir(input).unwrap();
            assert_eq!(input, &[&sep(1), PARENT_DIR].concat());
            assert_eq!(value, UnixComponent::ParentDir);
        }

        #[test]
        fn validate_normal() {
            // Empty input fails
            normal(b"").unwrap_err();

            // Fails if takes nothing
            normal(&[DISALLOWED_FILENAME_BYTES[0], b'a']).unwrap_err();

            // Succeeds just on its own
            let (input, value) = normal(b"hello").unwrap();
            assert_eq!(input, b"");
            assert_eq!(value, UnixComponent::Normal(b"hello"));

            // Succeeds, taking up to a disallowed filename byte
            for byte in DISALLOWED_FILENAME_BYTES {
                let input = [b'h', b'e', b'l', b'l', b'o', byte, b'm', b'o', b'r', b'e'];
                let (input, value) = normal(&input).unwrap();
                assert_eq!(input, &[byte, b'm', b'o', b'r', b'e']);
                assert_eq!(value, UnixComponent::Normal(b"hello"));
            }
        }

        #[test]
        fn validate_separator() {
            // Empty input fails
            separator(b"").unwrap_err();

            // Not starting with separator fails
            separator(&[b'a', SEPARATOR as u8]).unwrap_err();

            // Succeeds just on its own
            let (input, _) = separator(&[SEPARATOR as u8]).unwrap();
            assert_eq!(input, b"");

            // Succeeds, taking only what it matches
            let (input, _) = separator(&[
                SEPARATOR as u8,
                DISALLOWED_FILENAME_BYTES[0],
                SEPARATOR as u8,
            ])
            .unwrap();
            assert_eq!(input, &[DISALLOWED_FILENAME_BYTES[0], SEPARATOR as u8]);
        }
    }
}
