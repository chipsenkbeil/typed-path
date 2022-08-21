use crate::{
    common::parser::*,
    windows::{
        WindowsComponent, WindowsComponents, WindowsPrefix, WindowsPrefixComponent, SEPARATOR,
    },
};
use std::collections::VecDeque;

/// Reserved device names
const RESERVED_DEVICE_NAMES: [&[u8]; 22] = [
    b"CON", b"PRN", b"AUX", b"NUL", b"COM1", b"COM2", b"COM3", b"COM4", b"COM5", b"COM6", b"COM7",
    b"COM8", b"COM9", b"LPT1", b"LPT2", b"LPT3", b"LPT4", b"LPT5", b"LPT6", b"LPT7", b"LPT8",
    b"LPT9",
];

/// Bytes that are not allowed in file or directory names
const DISALLOWED_FILENAME_BYTES: [u8; 10] =
    [b'\\', b'/', b':', b'?', b'*', b'"', b'>', b'<', b'|', b'\0'];

/// Parse input to get [`WindowsComponents`]
///
/// ### Details
///
/// When parsing the path, there is a small amount of normalization:
///
/// Repeated separators are ignored, so a\\b and a\\\\b both have a and b as components.
///
/// Occurrences of . are normalized away, except if they are at the beginning of the path. For
/// example, a\\.\\b, a\\b\\, a\\b\\. and a\\b all have a and b as components, but .\\a\\b starts
/// with an additional CurDir component.
///
/// A trailing slash is normalized away, \\a\\b and \\a\\b\\ are equivalent.
///
/// Note that no other normalization takes place; in particular, a/c and a/b/../c are distinct, to
/// account for the possibility that b is a symbolic link (so its parent isn’t a).
pub fn parse(input: ParseInput) -> Result<WindowsComponents, ParseError> {
    let (input, components) = windows_components(input.as_ref())?;

    if !input.is_empty() {
        return Err("Did not fully parse input");
    }

    Ok(components)
}

/// Take multiple [`WindowsComponent`]s and map them into [`WindowsComponents`]
pub fn windows_components(input: ParseInput) -> ParseResult<WindowsComponents> {
    let start = input;

    // Path can potentially have a prefix and/or root directory
    let (input, maybe_prefix) = maybe(prefix)(input)?;
    let (input, maybe_root_dir) = maybe(root_dir)(input)?;

    // The get all remaining components in the path
    let (input, components) = one_or_more(inner_windows_component)(input)?;

    let mut components = VecDeque::from(components);
    if let Some(prefix) = maybe_prefix {
        components.push_front(prefix);
    }
    if let Some(root_dir) = maybe_root_dir {
        components.push_front(root_dir);
    }

    Ok((
        input,
        WindowsComponents {
            raw: &start[..(start.len() - input.len())],
            components,
        },
    ))
}

/// Take the next [`WindowsComponent`] from arbitrary position in path
///
/// No root directory or prefix is accepted here
fn inner_windows_component<'a>(input: ParseInput<'a>) -> ParseResult<WindowsComponent> {
    any_of!('a,
        suffixed(parent_dir, zero_or_more(separator)),
        suffixed(cur_dir, zero_or_more(separator)),
        suffixed(normal, zero_or_more(separator)),
    )(input)
}

fn root_dir(input: ParseInput) -> ParseResult<WindowsComponent> {
    let (input, _) = separator(input)?;
    Ok((input, WindowsComponent::RootDir))
}

fn cur_dir(input: ParseInput) -> ParseResult<WindowsComponent> {
    let (input, _) = byte(b'.')(input)?;
    Ok((input, WindowsComponent::CurDir))
}

fn parent_dir(input: ParseInput) -> ParseResult<WindowsComponent> {
    let (input, _) = bytes(b"..")(input)?;
    Ok((input, WindowsComponent::ParentDir))
}

fn normal(input: ParseInput) -> ParseResult<WindowsComponent> {
    let (input, normal) = normal_bytes(input)?;
    Ok((input, WindowsComponent::Normal(normal)))
}

fn normal_bytes(input: ParseInput) -> ParseResult<&[u8]> {
    let (input, normal) = take_until_byte(|b| DISALLOWED_FILENAME_BYTES.contains(&b))(input)?;
    Ok((input, normal))
}

fn separator(input: ParseInput) -> ParseResult<()> {
    let (input, _) = byte(SEPARATOR as u8)(input)?;
    Ok((input, ()))
}

fn prefix<'a>(input: ParseInput<'a>) -> ParseResult<WindowsComponent> {
    let (new_input, parsed) = any_of!('a,
        prefix_verbatim_unc,
        prefix_verbatim_disk,
        prefix_verbatim,
        prefix_device_ns,
        prefix_unc,
        prefix_disk,
    )(input)?;

    Ok((
        new_input,
        WindowsComponent::Prefix(WindowsPrefixComponent {
            raw: &input[..(input.len() - new_input.len())],
            parsed,
        }),
    ))
}

fn prefix_verbatim_unc(input: ParseInput) -> ParseResult<WindowsPrefix> {
    map(
        prefixed(
            bytes(br"\\?\UNC\"),
            divided(normal_bytes, separator, normal_bytes),
        ),
        |(server, share)| WindowsPrefix::VerbatimUNC(server, share),
    )(input)
}

fn prefix_verbatim(input: ParseInput) -> ParseResult<WindowsPrefix> {
    map(
        prefixed(bytes(br"\\?\"), normal_bytes),
        WindowsPrefix::Verbatim,
    )(input)
}

fn prefix_verbatim_disk(input: ParseInput) -> ParseResult<WindowsPrefix> {
    map(
        prefixed(bytes(br"\\?\"), disk_byte),
        WindowsPrefix::VerbatimDisk,
    )(input)
}

fn prefix_device_ns(input: ParseInput) -> ParseResult<WindowsPrefix> {
    map(
        prefixed(bytes(br"\\.\"), normal_bytes),
        WindowsPrefix::DeviceNS,
    )(input)
}

fn prefix_unc(input: ParseInput) -> ParseResult<WindowsPrefix> {
    map(
        prefixed(
            bytes(br"\\"),
            divided(normal_bytes, separator, normal_bytes),
        ),
        |(server, share)| WindowsPrefix::UNC(server, share),
    )(input)
}

fn prefix_disk(input: ParseInput) -> ParseResult<WindowsPrefix> {
    map(disk_byte, WindowsPrefix::Disk)(input)
}

/// `"C:" -> "C"`
fn disk_byte(input: ParseInput) -> ParseResult<u8> {
    let (input, drive_letter) = take(1)(input)?;
    let (input, _) = byte(b':')(input)?;
    Ok((input, drive_letter[0]))
}
