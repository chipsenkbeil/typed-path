use crate::{
    common::parser::*,
    unix::{UnixComponent, UnixComponents, SEPARATOR},
};
use std::collections::VecDeque;

/// Bytes that are not allowed in file or directory names
const DISALLOWED_FILENAME_BYTES: [u8; 2] = [b'/', b'\0'];

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
    let (input, components) = unix_components(input.as_ref())?;

    if !input.is_empty() {
        return Err("Did not fully parse input");
    }

    Ok(components)
}

/// Take multiple [`UnixComponent`]s and map them into [`UnixComponents`]
pub fn unix_components(input: ParseInput) -> ParseResult<UnixComponents> {
    let start = input;

    // Path can possible start with a root directory indicator
    let (input, maybe_root_dir) = maybe(root_dir)(input)?;

    // The get all remaining components in the path
    let (input, components) = one_or_more(inner_unix_component)(input)?;

    let mut components = VecDeque::from(components);
    if let Some(root_dir) = maybe_root_dir {
        components.push_front(root_dir);
    }

    Ok((
        input,
        UnixComponents {
            raw: &start[..(start.len() - input.len())],
            components,
        },
    ))
}

/// Take the next [`UnixComponent`] from arbitrary position in path
///
/// No root directory is accepted here
fn inner_unix_component<'a>(input: ParseInput<'a>) -> ParseResult<UnixComponent> {
    any_of!('a,
        suffixed(parent_dir, zero_or_more(separator)),
        suffixed(cur_dir, zero_or_more(separator)),
        suffixed(normal, zero_or_more(separator)),
    )(input)
}

fn root_dir(input: ParseInput) -> ParseResult<UnixComponent> {
    let (input, _) = separator(input)?;
    Ok((input, UnixComponent::RootDir))
}

fn cur_dir(input: ParseInput) -> ParseResult<UnixComponent> {
    let (input, _) = byte(b'.')(input)?;
    Ok((input, UnixComponent::CurDir))
}

fn parent_dir(input: ParseInput) -> ParseResult<UnixComponent> {
    let (input, _) = bytes(b"..")(input)?;
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
