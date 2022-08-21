use crate::{
    common::parser::*,
    unix::{
        UnixComponent, UnixComponents, CURRENT_DIR, DISALLOWED_FILENAME_BYTES, PARENT_DIR,
        SEPARATOR,
    },
};
use std::collections::VecDeque;

/// Parse input to get [`UnixComponents`]
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
pub fn parse(input: ParseInput) -> Result<UnixComponents, ParseError> {
    let (input, components) = unix_components(input)?;

    if !input.is_empty() {
        return Err("Did not fully parse input");
    }

    Ok(components)
}

/// Take multiple [`UnixComponent`]s and map them into [`UnixComponents`]
///
/// Will normalize occurrences of '.' by removing them except at the beginning of the path
fn unix_components(input: ParseInput) -> ParseResult<UnixComponents> {
    let start = input;

    // Path can possible start with a root directory indicator
    let (input, maybe_root_dir) = maybe(suffixed(root_dir, zero_or_more(separator)))(input)?;

    // Then get all remaining components in the path
    let (input, components) =
        zero_or_more(suffixed(file_or_dir_name, zero_or_more(separator)))(input)?;

    // Normalize by removing any current dir other than at the beginning, and only if there is no
    // root
    let mut components: VecDeque<_> = components
        .into_iter()
        .enumerate()
        .filter_map(|(i, c)| match c {
            UnixComponent::CurDir if i == 0 && maybe_root_dir.is_none() => {
                Some(UnixComponent::CurDir)
            }
            UnixComponent::CurDir => None,
            c => Some(c),
        })
        .collect();

    if let Some(root_dir) = maybe_root_dir {
        components.push_front(root_dir);
    }

    if components.is_empty() {
        return Err("Did not find root dir or any file or dir names");
    }

    Ok((
        input,
        UnixComponents {
            raw: &start[..(start.len() - input.len())],
            components,
        },
    ))
}

/// Take the next [`UnixComponent`] from arbitrary position in path that represents a file or
/// directory name
///
/// Trims off any extra separators
fn file_or_dir_name<'a>(input: ParseInput<'a>) -> ParseResult<UnixComponent> {
    // NOTE: Order is important here! '..' must parse before '.' before any allowed character
    any_of!('a, parent_dir, cur_dir, normal)(input)
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
    fn validate_parse() {
        // Empty input fails
        parse(b"").unwrap_err();

        // Unfinished consumption of input fails
        parse(b"abc\0def").unwrap_err();

        // Supports parsing any component individually
        let mut components = parse(&[SEPARATOR as u8]).unwrap();
        assert_eq!(components.next(), Some(UnixComponent::RootDir));
        assert_eq!(components.next(), None);

        let mut components = parse(CURRENT_DIR).unwrap();
        assert_eq!(components.next(), Some(UnixComponent::CurDir));
        assert_eq!(components.next(), None);

        let mut components = parse(PARENT_DIR).unwrap();
        assert_eq!(components.next(), Some(UnixComponent::ParentDir));
        assert_eq!(components.next(), None);

        let mut components = parse(b"hello").unwrap();
        assert_eq!(components.next(), Some(UnixComponent::Normal(b"hello")));
        assert_eq!(components.next(), None);
    }

    #[test]
    fn validate_unix_components() {
        // Empty input fails
        unix_components(b"").unwrap_err();

        // Fails if starts with a null character
        unix_components(b"\0hello").unwrap_err();

        // Succeeds if finds a root dir
        let (input, mut components) = unix_components(&[SEPARATOR as u8]).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(UnixComponent::RootDir));
        assert_eq!(components.next(), None);

        // Multiple separators still just mean root
        let input = sep(2);
        let (input, mut components) = unix_components(&input).unwrap();
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
        let (input, mut components) = unix_components(input).unwrap();
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
        let (input, mut components) = unix_components(input).unwrap();
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
        let (input, mut components) = unix_components(input).unwrap();
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
        let (input, mut components) = unix_components(input).unwrap();
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
        let (input, mut components) = unix_components(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(UnixComponent::RootDir));
        assert_eq!(components.next(), Some(UnixComponent::Normal(b"a")));
        assert_eq!(components.next(), Some(UnixComponent::ParentDir));
        assert_eq!(components.next(), None);
    }

    #[test]
    fn validate_file_or_dir_name() {
        // Empty input fails
        file_or_dir_name(b"").unwrap_err();

        // Not starting with a file or directory name fails
        file_or_dir_name(&DISALLOWED_FILENAME_BYTES).unwrap_err();

        // Succeeds if parent dir
        let (input, value) = file_or_dir_name(PARENT_DIR).unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, UnixComponent::ParentDir);

        // Succeeds if current dir
        let (input, value) = file_or_dir_name(CURRENT_DIR).unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, UnixComponent::CurDir);

        // Succeeds if normal
        let (input, value) = file_or_dir_name(b"hello").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, UnixComponent::Normal(b"hello"));

        // Succeeds if normal starting with '.'
        let (input, value) = file_or_dir_name(b".hello").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, UnixComponent::Normal(b".hello"));

        // Succeeds if normal starting with '..'
        let (input, value) = file_or_dir_name(b"..hello").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, UnixComponent::Normal(b"..hello"));

        // Succeeds if normal is exactly '...'
        let (input, value) = file_or_dir_name(b"...").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, UnixComponent::Normal(b"..."));
    }

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
