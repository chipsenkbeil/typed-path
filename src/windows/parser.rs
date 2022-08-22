use crate::{
    common::parser::*,
    windows::{
        WindowsComponent, WindowsComponents, WindowsPrefix, WindowsPrefixComponent, ALT_SEPARATOR,
        CURRENT_DIR, PARENT_DIR, SEPARATOR,
    },
};
use std::collections::VecDeque;

/// Parse input to get [`WindowsComponents`]
///
/// ### Details
///
/// When parsing the path, there is a small amount of normalization:
///
/// Repeated separators are ignored, so a\\b and a\\\\b both have a and b as components.
///
/// Occurrences of '.' are normalized away, except if they are at the beginning of the path. For
/// example, a\\.\\b, a\\b\\, a\\b\\. and a\\b all have a and b as components, but .\\a\\b starts
/// with an additional CurDir component.
///
/// A trailing slash is normalized away, \\a\\b and \\a\\b\\ are equivalent.
///
/// Note that no other normalization takes place; in particular, a\c and a\b\..\c are distinct, to
/// account for the possibility that b is a symbolic link (so its parent isn’t a).
///
/// ### Verbatim disabling of normalization
///
/// When using `\\?\`, [normalization is skipped](https://docs.microsoft.com/en-us/dotnet/standard/io/file-path-formats#skip-normalization).
///
/// This means that from the above rules:
///
/// * '.' is not normalized away
/// * '/' is not used as a separator
///
/// Note that repeat separators are still removed and trailing slashes are still not included
pub fn parse(input: ParseInput) -> Result<WindowsComponents, ParseError> {
    let (input, components) = windows_components(input)?;

    if !input.is_empty() {
        return Err("Did not fully parse input");
    }

    Ok(components)
}

/// Take multiple [`WindowsComponent`]s and map them into [`WindowsComponents`]
fn windows_components(input: ParseInput) -> ParseResult<WindowsComponents> {
    let start = input;

    // Path can potentially have a prefix and/or root directory
    let (input, maybe_prefix) = maybe(prefix_component)(input)?;

    // Normalize unless the path starts exactly with \\?\
    let normalize = maybe_prefix
        .map(|p| !p.raw.starts_with(br"\\?\"))
        .unwrap_or(true);

    // If normalizing, we treat all forward slashes as standard Windows separators,
    // otherwise we only treat '\' as a separator
    let sep = if normalize {
        separator
    } else {
        verbatim_separator
    };

    let (input, maybe_root_dir) = maybe(suffixed(root_dir(sep), zero_or_more(sep)))(input)?;

    // Check if we have a physical root or an implicit root
    let has_root = maybe_root_dir.is_some()
        || maybe_prefix
            .map(|p| !matches!(p.kind(), WindowsPrefix::Disk(_)))
            .unwrap_or(false);

    // Then get all remaining components in the path
    let (input, components) =
        zero_or_more(suffixed(file_or_dir_name(sep), zero_or_more(sep)))(input)?;

    // Normalize by removing any current dir other than at the beginning, and only if there is no
    // root; this will do nothing to current dir if we are not normalizing
    let mut components: VecDeque<_> = components
        .into_iter()
        .enumerate()
        .filter_map(|(i, c)| match c {
            WindowsComponent::CurDir if i == 0 && !has_root => Some(WindowsComponent::CurDir),
            WindowsComponent::CurDir if normalize => None,
            c => Some(c),
        })
        .collect();

    // Place root dir in front of path
    if let Some(root_dir) = maybe_root_dir {
        components.push_front(root_dir);
    }

    // Place prefix in front of path & root dir
    if let Some(prefix) = maybe_prefix {
        components.push_front(WindowsComponent::Prefix(prefix));
    }

    if components.is_empty() {
        return Err("Did not find prefix, root dir, or any file or dir names");
    }

    Ok((
        input,
        WindowsComponents {
            raw: &start[..(start.len() - input.len())],
            components,
        },
    ))
}

/// Take the next [`WindowsComponent`] from arbitrary position in path that represents a file or
/// directory name
///
/// Trims off any extra separators
fn file_or_dir_name<'a>(
    sep: impl Fn(ParseInput) -> ParseResult<()> + Copy,
) -> impl Fn(ParseInput<'a>) -> ParseResult<WindowsComponent> {
    let parent_dir = parent_dir(sep);
    let cur_dir = cur_dir(sep);
    let normal = normal(sep);

    move |input: ParseInput| {
        // NOTE: Order is important here! '..' must parse before '.' before any allowed character
        any_of!('a, parent_dir, cur_dir, normal)(input)
    }
}

fn root_dir(
    sep: impl Fn(ParseInput) -> ParseResult<()> + Copy,
) -> impl Fn(ParseInput) -> ParseResult<WindowsComponent> {
    move |input: ParseInput| {
        let (input, _) = sep(input)?;
        Ok((input, WindowsComponent::RootDir))
    }
}

fn cur_dir(
    sep: impl Fn(ParseInput) -> ParseResult<()> + Copy,
) -> impl Fn(ParseInput) -> ParseResult<WindowsComponent> {
    move |input: ParseInput| {
        let (input, _) = suffixed(bytes(CURRENT_DIR), any_of!('_, empty, peek(sep)))(input)?;
        Ok((input, WindowsComponent::CurDir))
    }
}

fn parent_dir(
    sep: impl Fn(ParseInput) -> ParseResult<()> + Copy,
) -> impl Fn(ParseInput) -> ParseResult<WindowsComponent> {
    move |input: ParseInput| {
        let (input, _) = suffixed(bytes(PARENT_DIR), any_of!('_, empty, peek(sep)))(input)?;
        Ok((input, WindowsComponent::ParentDir))
    }
}

fn normal(
    sep: impl Fn(ParseInput) -> ParseResult<()> + Copy,
) -> impl Fn(ParseInput) -> ParseResult<WindowsComponent> {
    move |input: ParseInput| {
        let (input, normal) = normal_bytes(sep)(input)?;
        Ok((input, WindowsComponent::Normal(normal)))
    }
}

/// Parse normal bytes
///
/// NOTE: If we were really nice, we'd parse up to the disallowed filenames, but Rust and other
///       implementations don't appear to do that and instead just jump to the next separator
fn normal_bytes(
    sep: impl Fn(ParseInput) -> ParseResult<()> + Copy,
) -> impl Fn(ParseInput) -> ParseResult<ParseInput> {
    move |input: ParseInput| {
        let (input, normal) = take_while(not(sep))(input)?;
        Ok((input, normal))
    }
}

fn separator<'a>(input: ParseInput<'a>) -> ParseResult<()> {
    let (input, _) = any_of!('a, byte(SEPARATOR as u8), byte(ALT_SEPARATOR as u8))(input)?;
    Ok((input, ()))
}

/// For Windows, verbatim paths only use '\' as a separator
fn verbatim_separator(input: ParseInput) -> ParseResult<()> {
    let (input, _) = byte(SEPARATOR as u8)(input)?;
    Ok((input, ()))
}

pub fn prefix_component(input: ParseInput) -> ParseResult<WindowsPrefixComponent> {
    let (new_input, parsed) = prefix(input)?;

    Ok((
        new_input,
        WindowsPrefixComponent {
            raw: &input[..(input.len() - new_input.len())],
            parsed,
        },
    ))
}

fn prefix<'a>(input: ParseInput<'a>) -> ParseResult<WindowsPrefix> {
    any_of!('a,
        prefix_verbatim_unc,
        prefix_verbatim_disk,
        prefix_verbatim,
        prefix_device_ns,
        prefix_unc,
        prefix_disk,
    )(input)
}

/// Format is `\\?\UNC\SERVER\SHARE` where the backslash is interchangeable with a forward slash
fn prefix_verbatim_unc(input: ParseInput) -> ParseResult<WindowsPrefix> {
    // Based on verbatim, if it is EXACTLY \\?\, then we want to use the verbatim separator
    let sep = if input.starts_with(br"\\?\") {
        verbatim_separator
    } else {
        separator
    };

    let (input, _) = verbatim(input)?;
    let (input, _) = bytes(b"UNC")(input)?;
    let (input, _) = sep(input)?;
    let (input, server) = normal_bytes(sep)(input)?;
    let (input, _) = maybe(sep)(input)?;
    let (input, maybe_share) = maybe(normal_bytes(sep))(input)?;

    Ok((
        input,
        WindowsPrefix::VerbatimUNC(server, maybe_share.unwrap_or(b"")),
    ))
}

/// Format is `\\?\PICTURES:` where the backslash is interchangeable with a forward slash
fn prefix_verbatim<'a>(input: ParseInput<'a>) -> ParseResult<WindowsPrefix> {
    let (input, _) = not(prefix_verbatim_disk)(input)?;
    let (input, _) = not(prefix_verbatim_unc)(input)?;

    // Based on verbatim, if it is EXACTLY \\?\, then we want to use the verbatim separator
    let sep = if input.starts_with(br"\\?\") {
        verbatim_separator
    } else {
        separator
    };

    let (input, _) = verbatim(input)?;

    // NOTE: Support a blank verbatim because this is what Rust's stdlib does
    //
    // e.g. `\\?\\hello\world` -> [Verbatim(""), RootDir, Normal("hello"), Normal("world")]
    let (input, value) = any_of!('a, normal_bytes(sep), map(peek(sep), |_| b""))(input)?;

    Ok((input, WindowsPrefix::Verbatim(value)))
}

/// Format is `\\?\DISK:` where the backslash is interchangeable with a forward slash
fn prefix_verbatim_disk(input: ParseInput) -> ParseResult<WindowsPrefix> {
    map(prefixed(verbatim, disk_byte), WindowsPrefix::VerbatimDisk)(input)
}

/// Matches `\\?\` where the backslash is interchangeable with a forward slash
fn verbatim(input: ParseInput) -> ParseResult<()> {
    let (input, _) = separator(input)?;
    let (input, _) = separator(input)?;
    let (input, _) = byte(b'?')(input)?;
    let (input, _) = separator(input)?;
    Ok((input, ()))
}

/// Format is `\\.\DEVICE` where the backslash is interchangeable with a forward slash
fn prefix_device_ns(input: ParseInput) -> ParseResult<WindowsPrefix> {
    let (input, _) = separator(input)?;
    let (input, _) = separator(input)?;
    let (input, _) = byte(b'.')(input)?;
    let (input, _) = separator(input)?;

    map(normal_bytes(separator), WindowsPrefix::DeviceNS)(input)
}

/// Format is `\\SERVER\SHARE` where the backslash is interchangeable with a forward slash
fn prefix_unc(input: ParseInput) -> ParseResult<WindowsPrefix> {
    let (input, _) = separator(input)?;
    let (input, _) = separator(input)?;

    let (input, server) = normal_bytes(separator)(input)?;
    let (input, _) = maybe(separator)(input)?;
    let (input, maybe_share) = maybe(normal_bytes(separator))(input)?;

    Ok((
        input,
        WindowsPrefix::UNC(server, maybe_share.unwrap_or(b"")),
    ))
}

/// Format is `C:`
fn prefix_disk(input: ParseInput) -> ParseResult<WindowsPrefix> {
    map(disk_byte, WindowsPrefix::Disk)(input)
}

/// `"C:" -> "C"`
fn disk_byte(input: ParseInput) -> ParseResult<u8> {
    let (input, drive_letter) = drive_letter(input)?;

    let (input, _) = byte(b':')(input)?;
    Ok((input, drive_letter))
}

/// `"C:" -> "C"`
fn drive_letter(input: ParseInput) -> ParseResult<u8> {
    let (input, drive_letter) = take(1)(input)?;

    // Drive letter should ONLY be a-zA-Z
    if !(drive_letter[0] as char).is_alphabetic() {
        return Err("drive not alphabetic");
    }

    Ok((input, drive_letter[0]))
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

    fn get_prefix<'a>(component: impl Into<Option<WindowsComponent<'a>>>) -> WindowsPrefix<'a> {
        match component.into() {
            Some(WindowsComponent::Prefix(p)) => p.kind(),
            Some(_) => panic!("not a prefix"),
            None => panic!("component is none"),
        }
    }

    #[test]
    fn validate_parse() {
        // Empty input fails
        parse(b"").unwrap_err();

        // Supports parsing any component individually
        let mut components = parse(&[SEPARATOR as u8]).unwrap();
        assert_eq!(components.next(), Some(WindowsComponent::RootDir));
        assert_eq!(components.next(), None);

        let mut components = parse(&[ALT_SEPARATOR as u8]).unwrap();
        assert_eq!(components.next(), Some(WindowsComponent::RootDir));
        assert_eq!(components.next(), None);

        let mut components = parse(CURRENT_DIR).unwrap();
        assert_eq!(components.next(), Some(WindowsComponent::CurDir));
        assert_eq!(components.next(), None);

        let mut components = parse(PARENT_DIR).unwrap();
        assert_eq!(components.next(), Some(WindowsComponent::ParentDir));
        assert_eq!(components.next(), None);

        let mut components = parse(b"hello").unwrap();
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"hello")));
        assert_eq!(components.next(), None);

        let mut components = parse(b"C:").unwrap();
        assert_eq!(get_prefix(components.next()), WindowsPrefix::Disk(b'C'));
    }

    #[test]
    #[ignore]
    fn validate_rust_variation_1() {
        // Verbatim disk with something other than a trailing slash following is just verbatim
        //
        // CASE: \\?\C:. -> [Verbatim("C:.")]
        //
        // NOTE: This seems wrong to me as I would expect this to be verbatim with a current dir;
        //       so, I am not going to worry about this right now
        let mut components = parse(br"\\?\C:.").unwrap();
        assert_eq!(
            get_prefix(components.next()),
            WindowsPrefix::Verbatim(b"C:.")
        );
        assert_eq!(components.next(), None);
    }

    #[test]
    #[ignore]
    fn validate_rust_variation_2() {
        // Disk with a '.' following is just disk, not disk and current dir
        //
        // CASE: C:. -> [Disk('C')]
        //
        // NOTE: This seems wrong to me as I would expect this to include current dir since there
        //       is no root at the beginning; so, I am not going to worry about this right now
        let mut components = parse(b"C:.").unwrap();
        assert_eq!(get_prefix(components.next()), WindowsPrefix::Disk(b'C'));
        assert_eq!(components.next(), None);
    }

    #[test]
    #[ignore]
    fn validate_rust_variation_3() {
        // Disk with '.' at beginning of path has the current dir removed
        //
        // NOTE: This seems wrong to me as I would expect this to include current dir since there
        //       is no root at the beginning
        let mut components = parse(br"C:.\hello").unwrap();
        assert_eq!(get_prefix(components.next()), WindowsPrefix::Disk(b'C'));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"hello")));
        assert_eq!(components.next(), None);
    }

    #[test]
    fn validate_rust_variation_4() {
        // Verbatim mode can have an empty verbatim field if separators at the beginning
        //
        // NOTE: I'm not sure about this one. It seems like something incorrect, but the current
        //       parser implementation is outputting UNC when this fails
        let mut components = parse(br"\\?\\hello").unwrap();
        assert_eq!(get_prefix(components.next()), WindowsPrefix::Verbatim(b""));
        assert_eq!(components.next(), Some(WindowsComponent::RootDir));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"hello")));
        assert_eq!(components.next(), None);
    }

    #[test]
    fn validate_arbitrary_rules_to_match_std_lib() {
        // Support verbatim disk on its own
        let mut components = parse(br"\\?\C:").unwrap();
        assert_eq!(
            get_prefix(components.next()),
            WindowsPrefix::VerbatimDisk(b'C')
        );
        assert_eq!(components.next(), None);

        // Support verbatim disk with root
        let mut components = parse(br"\\?\C:\").unwrap();
        assert_eq!(
            get_prefix(components.next()),
            WindowsPrefix::VerbatimDisk(b'C')
        );
        assert_eq!(components.next(), Some(WindowsComponent::RootDir));
        assert_eq!(components.next(), None);

        // Verbatim can have '.' as value
        let mut components = parse(br"\\?\.\hello\.\again").unwrap();
        assert_eq!(get_prefix(components.next()), WindowsPrefix::Verbatim(b"."));
        assert_eq!(components.next(), Some(WindowsComponent::RootDir));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"hello")));
        assert_eq!(components.next(), Some(WindowsComponent::CurDir));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"again")));
        assert_eq!(components.next(), None);

        // Verbatim mode still removes extra separators
        let mut components = parse(br"\\?\.\\\hello\\\.\\\again").unwrap();
        assert_eq!(get_prefix(components.next()), WindowsPrefix::Verbatim(b"."));
        assert_eq!(components.next(), Some(WindowsComponent::RootDir));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"hello")));
        assert_eq!(components.next(), Some(WindowsComponent::CurDir));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"again")));
        assert_eq!(components.next(), None);

        // Verbatim UNC can have an optional share
        let mut components = parse(br"\\?\UNC\server").unwrap();
        assert_eq!(
            get_prefix(components.next()),
            WindowsPrefix::VerbatimUNC(b"server", b"")
        );
        assert_eq!(components.next(), None);

        // Verbatim will include '/' as part of name
        let mut components = parse(br"\\?\some/name").unwrap();
        assert_eq!(
            get_prefix(components.next()),
            WindowsPrefix::Verbatim(b"some/name")
        );
        assert_eq!(components.next(), None);
    }

    #[test]
    fn validate_windows_components() {
        // Empty input fails
        windows_components(b"").unwrap_err();

        // Succeeds if finds a prefix
        let (input, mut components) = windows_components(br"\\server\share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(
            get_prefix(components.next()),
            WindowsPrefix::UNC(b"server", b"share")
        );
        assert_eq!(components.next(), None);

        // Succeeds if finds a root dir
        let (input, mut components) = windows_components(&[SEPARATOR as u8]).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(WindowsComponent::RootDir));
        assert_eq!(components.next(), None);

        // Multiple separators still just mean root
        let input = sep(2);
        let (input, mut components) = windows_components(&input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(WindowsComponent::RootDir));
        assert_eq!(components.next(), None);

        // Succeeds even if there isn't a root or prefix
        //
        // E.g. a\b\c
        let input = &[
            &[b'a'],
            sep(1).as_slice(),
            &[b'b'],
            sep(1).as_slice(),
            &[b'c'],
        ]
        .concat();
        let (input, mut components) = windows_components(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"a")));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"b")));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"c")));
        assert_eq!(components.next(), None);

        // Should support '.' at beginning of path
        //
        // E.g. .\b\c
        let input = &[
            CURRENT_DIR,
            sep(1).as_slice(),
            &[b'b'],
            sep(1).as_slice(),
            &[b'c'],
        ]
        .concat();
        let (input, mut components) = windows_components(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(WindowsComponent::CurDir));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"b")));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"c")));
        assert_eq!(components.next(), None);

        // Should support '.' at beginning of path after a prefix as long as the
        // prefix does not imply that there is an implicit root
        //
        // implicit root is essentially every prefix except the normal drive
        //
        // E.g. C:. and C:.\ are okay to keep
        // E.g. \\?\C:. is verbatim mode, so the dot counts as well
        let input = &[b"C:", CURRENT_DIR].concat();
        let (input, mut components) = windows_components(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(get_prefix(components.next()), WindowsPrefix::Disk(b'C'));
        assert_eq!(components.next(), Some(WindowsComponent::CurDir));
        assert_eq!(components.next(), None);

        let input = &[b"C:", CURRENT_DIR, sep(1).as_slice()].concat();
        let (input, mut components) = windows_components(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(get_prefix(components.next()), WindowsPrefix::Disk(b'C'));
        assert_eq!(components.next(), Some(WindowsComponent::CurDir));
        assert_eq!(components.next(), None);

        let input = &[br"\\?\C:", CURRENT_DIR].concat();
        let (input, mut components) = windows_components(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(
            get_prefix(components.next()),
            WindowsPrefix::VerbatimDisk(b'C')
        );
        assert_eq!(components.next(), Some(WindowsComponent::CurDir));
        assert_eq!(components.next(), None);

        // Should remove current dir from anywhere if not at beginning
        //
        // E.g. \.\b\c -> \b\c
        // E.g. a\.\c -> a\c
        let input = &[
            sep(1).as_slice(),
            CURRENT_DIR,
            sep(1).as_slice(),
            &[b'b'],
            sep(1).as_slice(),
            &[b'c'],
        ]
        .concat();
        let (input, mut components) = windows_components(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(WindowsComponent::RootDir));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"b")));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"c")));
        assert_eq!(components.next(), None);

        let input = &[
            &[b'a'],
            sep(1).as_slice(),
            CURRENT_DIR,
            sep(1).as_slice(),
            &[b'c'],
        ]
        .concat();
        let (input, mut components) = windows_components(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"a")));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"c")));
        assert_eq!(components.next(), None);

        // Should strip multiple separators and normalize '.' if not starting with \\?\
        //
        // E.g. \\\\\a\\\.\\..\\\ -> [ROOT, "a", CURRENT_DIR, PARENT_DIR]
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
        let (input, mut components) = windows_components(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(components.next(), Some(WindowsComponent::RootDir));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"a")));
        assert_eq!(components.next(), Some(WindowsComponent::ParentDir));
        assert_eq!(components.next(), None);

        // Prefix should come before root should come before path
        //
        // E.g. \\\\\a\\\.\\..\\\ -> [ROOT, "a", CURRENT_DIR, PARENT_DIR]
        let input = &[
            b"C:",
            sep(5).as_slice(),
            &[b'a'],
            sep(3).as_slice(),
            CURRENT_DIR,
            sep(2).as_slice(),
            PARENT_DIR,
            sep(3).as_slice(),
        ]
        .concat();
        let (input, mut components) = windows_components(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(get_prefix(components.next()), WindowsPrefix::Disk(b'C'));
        assert_eq!(components.next(), Some(WindowsComponent::RootDir));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"a")));
        assert_eq!(components.next(), Some(WindowsComponent::ParentDir));
        assert_eq!(components.next(), None);

        // Verbatim path starting with \\?\ should retain . throughout path
        //
        // E.g. \\?\pictures\a\.\..\. -> [PREFIX, ROOT, "a", CURRENT_DIR, PARENT_DIR, CURRENT_DIR]
        let input = &[
            br"\\?\pictures",
            sep(5).as_slice(),
            &[b'a'],
            sep(3).as_slice(),
            CURRENT_DIR,
            sep(2).as_slice(),
            PARENT_DIR,
            sep(3).as_slice(),
            CURRENT_DIR,
        ]
        .concat();
        let (input, mut components) = windows_components(input).unwrap();
        assert_eq!(input, b"");
        assert_eq!(
            get_prefix(components.next()),
            WindowsPrefix::Verbatim(b"pictures")
        );
        assert_eq!(components.next(), Some(WindowsComponent::RootDir));
        assert_eq!(components.next(), Some(WindowsComponent::Normal(b"a")));
        assert_eq!(components.next(), Some(WindowsComponent::CurDir));
        assert_eq!(components.next(), Some(WindowsComponent::ParentDir));
        assert_eq!(components.next(), Some(WindowsComponent::CurDir));
        assert_eq!(components.next(), None);
    }

    #[test]
    fn validate_prefix_component() {
        // Empty input fails
        prefix_component(b"").unwrap_err();

        // Not starting with a prefix fails
        prefix_component(&[SEPARATOR as u8]).unwrap_err();

        // Should succeed if a disk
        let (input, value) = prefix_component(b"C:").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value.raw, b"C:");
        assert_eq!(value.parsed, WindowsPrefix::Disk(b'C'));

        // Should succeed if verbatim
        let (input, value) = prefix_component(br"\\?\pictures").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value.raw, br"\\?\pictures");
        assert_eq!(value.parsed, WindowsPrefix::Verbatim(b"pictures"));

        // Should succeed if verbatim UNC
        let (input, value) = prefix_component(br"\\?\UNC\server\share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value.raw, br"\\?\UNC\server\share");
        assert_eq!(
            value.parsed,
            WindowsPrefix::VerbatimUNC(b"server", b"share")
        );

        // Should succeed if verbatim disk
        let (input, value) = prefix_component(br"\\?\C:").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value.raw, br"\\?\C:");
        assert_eq!(value.parsed, WindowsPrefix::VerbatimDisk(b'C'));

        // Should succeed if Device NS
        let (input, value) = prefix_component(br"\\.\BrainInterface").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value.raw, br"\\.\BrainInterface");
        assert_eq!(value.parsed, WindowsPrefix::DeviceNS(b"BrainInterface"));

        // Should succeed if UNC
        let (input, value) = prefix_component(br"\\server\share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value.raw, br"\\server\share");
        assert_eq!(value.parsed, WindowsPrefix::UNC(b"server", b"share"));

        // Should set raw to only consumed portion of input
        let (input, value) = prefix_component(br"C:\path").unwrap();
        assert_eq!(input, br"\path");
        assert_eq!(value.raw, b"C:");
        assert_eq!(value.parsed, WindowsPrefix::Disk(b'C'));
    }

    #[test]
    fn validate_prefix() {
        // Empty input fails
        prefix(b"").unwrap_err();

        // Not starting with a prefix fails
        prefix(&[SEPARATOR as u8]).unwrap_err();

        // Should succeed if a disk
        let (input, value) = prefix(b"C:").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::Disk(b'C'));

        // Should succeed if verbatim
        let (input, value) = prefix(br"\\?\pictures").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::Verbatim(b"pictures"));

        // Should succeed if verbatim UNC
        let (input, value) = prefix(br"\\?\UNC\server\share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::VerbatimUNC(b"server", b"share"));

        // Should succeed if verbatim disk
        let (input, value) = prefix(br"\\?\C:").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::VerbatimDisk(b'C'));

        // Should succeed if device NS
        let (input, value) = prefix(br"\\.\BrainInterface").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::DeviceNS(b"BrainInterface"));

        // Should succeed if UNC
        let (input, value) = prefix(br"\\server\share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::UNC(b"server", b"share"));
    }

    #[test]
    fn validate_prefix_verbatim_unc() {
        // Empty input fails
        prefix_verbatim_unc(b"").unwrap_err();

        // Not starting with appropriate character set
        prefix_verbatim_unc(br"server\share").unwrap_err();
        prefix_verbatim_unc(br"\server\share").unwrap_err();
        prefix_verbatim_unc(br"\\server\share").unwrap_err();
        prefix_verbatim_unc(br"\\?server\share").unwrap_err();
        prefix_verbatim_unc(br"\?\server\share").unwrap_err();
        prefix_verbatim_unc(br"?\\server\share").unwrap_err();
        prefix_verbatim_unc(br"?\\?\server\share").unwrap_err();
        prefix_verbatim_unc(br"\\?\UNCserver\share").unwrap_err();
        prefix_verbatim_unc(br"\?\UNC\server\share").unwrap_err();
        prefix_verbatim_unc(br"\\.\UNC\server\share").unwrap_err();
        prefix_verbatim_unc(br"\\?\UN\server\share").unwrap_err();

        // Fails if not verbatim type (other forms of verbatim)
        prefix_verbatim_unc(br"\\?\C:").unwrap_err();
        prefix_verbatim_unc(br"\\?\pictures").unwrap_err();

        // Supports both primary and alternate separators UNLESS prefixed with EXACTLY \\?\
        // in which case it only parses '\' as the separator
        let (input, value) = prefix_verbatim_unc(br"\\?/UNC\server\share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::VerbatimUNC(b"server", b"share"));

        let (input, value) = prefix_verbatim_unc(br"\/?\UNC\server\share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::VerbatimUNC(b"server", b"share"));

        let (input, value) = prefix_verbatim_unc(br"/\?\UNC\server\share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::VerbatimUNC(b"server", b"share"));

        // Verify that \\?\ causes the exact match requirement
        let (input, value) = prefix_verbatim_unc(br"\\?\UNC\server\share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::VerbatimUNC(b"server", b"share"));

        let (input, value) = prefix_verbatim_unc(br"\\?\UNC\server/share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::VerbatimUNC(b"server/share", b""));

        let (input, value) = prefix_verbatim_unc(br"\\?\UNC\server/share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::VerbatimUNC(b"server/share", b""));

        // Share is optional
        let (input, value) = prefix_verbatim_unc(br"\\?\UNC\server").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::VerbatimUNC(b"server", b""));

        // These will NOT parse correctly because of the \\?\ prefix forcing UNC\
        prefix_verbatim_unc(br"\\?\UNC/server\share").unwrap_err();
        prefix_verbatim_unc(br"\\?\UNC/server/share").unwrap_err();

        // Consumes up to the next separator
        let (input, value) = prefix_verbatim_unc(br"\\?\UNC\server\share\hello").unwrap();
        assert_eq!(input, br"\hello");
        assert_eq!(value, WindowsPrefix::VerbatimUNC(b"server", b"share"));
    }

    #[test]
    fn validate_prefix_verbatim() {
        // Empty input fails
        prefix_verbatim(b"").unwrap_err();

        // Not starting with appropriate character set
        prefix_verbatim(br"pictures").unwrap_err();
        prefix_verbatim(br"\pictures").unwrap_err();
        prefix_verbatim(br"\\pictures").unwrap_err();
        prefix_verbatim(br"\\?pictures").unwrap_err();
        prefix_verbatim(br"\?\pictures").unwrap_err();
        prefix_verbatim(br"?\\pictures").unwrap_err();
        prefix_verbatim(br"?\\?\pictures").unwrap_err();

        // Fails if not verbatim type (other forms of verbatim)
        prefix_verbatim(br"\\?\C:").unwrap_err();
        prefix_verbatim(br"\\?\UNC\server\share").unwrap_err();

        // Supports both primary and alternate separators
        let (input, value) = prefix_verbatim(br"\\?\pictures").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::Verbatim(b"pictures"));

        let (input, value) = prefix_verbatim(br"\\?/pictures").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::Verbatim(b"pictures"));

        let (input, value) = prefix_verbatim(br"\/?\pictures").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::Verbatim(b"pictures"));

        let (input, value) = prefix_verbatim(br"/\?\pictures").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::Verbatim(b"pictures"));

        // Consumes up to the next separator
        let (input, value) = prefix_verbatim(br"\\?\pictures\hello").unwrap();
        assert_eq!(input, br"\hello");
        assert_eq!(value, WindowsPrefix::Verbatim(b"pictures"));
    }

    #[test]
    fn validate_prefix_verbatim_disk() {
        // Empty input fails
        prefix_verbatim_disk(b"").unwrap_err();

        // Not starting with appropriate character set
        prefix_verbatim_disk(br"C:").unwrap_err();
        prefix_verbatim_disk(br"\C:").unwrap_err();
        prefix_verbatim_disk(br"\\C:").unwrap_err();
        prefix_verbatim_disk(br"\\?C:").unwrap_err();
        prefix_verbatim_disk(br"\?\C:").unwrap_err();
        prefix_verbatim_disk(br"?\\C:").unwrap_err();
        prefix_verbatim_disk(br"?\\?\C:").unwrap_err();

        // Fails if not a drive letter (other forms of verbatim)
        prefix_verbatim_disk(br"\\?\pictures").unwrap_err();
        prefix_verbatim_disk(br"\\?\UNC\server\share").unwrap_err();

        // Supports both primary and alternate separators
        let (input, value) = prefix_verbatim_disk(br"\\?\C:").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::VerbatimDisk(b'C'));

        let (input, value) = prefix_verbatim_disk(br"\\?/C:").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::VerbatimDisk(b'C'));

        let (input, value) = prefix_verbatim_disk(br"\/?\C:").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::VerbatimDisk(b'C'));

        let (input, value) = prefix_verbatim_disk(br"/\?\C:").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::VerbatimDisk(b'C'));

        // Consumes up to the drive and :
        let (input, value) = prefix_verbatim_disk(br"\\?\C:.").unwrap();
        assert_eq!(input, b".");
        assert_eq!(value, WindowsPrefix::VerbatimDisk(b'C'));

        let (input, value) = prefix_verbatim_disk(br"\\?\C:\").unwrap();
        assert_eq!(input, br"\");
        assert_eq!(value, WindowsPrefix::VerbatimDisk(b'C'));
    }

    #[test]
    fn validate_prefix_device_ns() {
        // Empty input fails
        prefix_device_ns(b"").unwrap_err();

        // Not starting with appropriate character set
        prefix_device_ns(br"BrainInterface").unwrap_err();
        prefix_device_ns(br"\BrainInterface").unwrap_err();
        prefix_device_ns(br"\\BrainInterface").unwrap_err();
        prefix_device_ns(br"\\.BrainInterface").unwrap_err();
        prefix_device_ns(br"\.\BrainInterface").unwrap_err();
        prefix_device_ns(br".\\BrainInterface").unwrap_err();
        prefix_device_ns(br".\\.\BrainInterface").unwrap_err();

        // Supports both primary and alternate separators
        let (input, value) = prefix_device_ns(br"\\.\BrainInterface").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::DeviceNS(b"BrainInterface"));

        let (input, value) = prefix_device_ns(br"\\./BrainInterface").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::DeviceNS(b"BrainInterface"));

        let (input, value) = prefix_device_ns(br"\/.\BrainInterface").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::DeviceNS(b"BrainInterface"));

        let (input, value) = prefix_device_ns(br"/\.\BrainInterface").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::DeviceNS(b"BrainInterface"));

        // Consumes up to the next separator
        let (input, value) = prefix_device_ns(br"\\.\BrainInterface\hello").unwrap();
        assert_eq!(input, br"\hello");
        assert_eq!(value, WindowsPrefix::DeviceNS(b"BrainInterface"));
    }

    #[test]
    fn validate_prefix_unc() {
        // Empty input fails
        prefix_unc(b"").unwrap_err();

        // Not starting with two separators ('\\', '\/', '//', '/\')
        prefix_unc(br"server\share").unwrap_err();
        prefix_unc(br"\server\share").unwrap_err();

        // Supports parsing just the server and not the share
        let (input, value) = prefix_unc(br"\\server").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::UNC(b"server", b""));

        let (input, value) = prefix_unc(br"\\server\").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::UNC(b"server", b""));

        // Supports both primary and alternate separators
        let (input, value) = prefix_unc(br"\\server\share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::UNC(b"server", b"share"));

        let (input, value) = prefix_unc(br"\\server/share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::UNC(b"server", b"share"));

        let (input, value) = prefix_unc(br"\/server\share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::UNC(b"server", b"share"));

        let (input, value) = prefix_unc(br"/\server\share").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::UNC(b"server", b"share"));

        // Consumes up to the next separator
        let (input, value) = prefix_unc(br"\\server\share\hello").unwrap();
        assert_eq!(input, br"\hello");
        assert_eq!(value, WindowsPrefix::UNC(b"server", b"share"));
    }

    #[test]
    fn validate_prefix_disk() {
        // Empty input fails
        prefix_disk(b"").unwrap_err();

        // Not starting with a drive letter fails
        prefix_disk(b"1:").unwrap_err();

        // Not ending with : fails
        prefix_disk(b"C").unwrap_err();
        prefix_disk(b"CC").unwrap_err();

        // Supports uppercase alphabet
        let (input, value) = prefix_disk(b"C:").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::Disk(b'C'));

        // Supports lowercase alphabet
        let (input, value) = prefix_disk(b"c:").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsPrefix::Disk(b'c'));

        // Consumes only drive letter and :
        let (input, value) = prefix_disk(br"C:\path").unwrap();
        assert_eq!(input, br"\path");
        assert_eq!(value, WindowsPrefix::Disk(b'C'));
    }

    #[test]
    fn validate_file_or_dir_name() {
        let file_or_dir_name = file_or_dir_name(separator);

        // Empty input fails
        file_or_dir_name(b"").unwrap_err();

        // Not starting with a file or directory name fails
        file_or_dir_name(&[SEPARATOR as u8]).unwrap_err();

        // Succeeds if parent dir
        let (input, value) = file_or_dir_name(PARENT_DIR).unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsComponent::ParentDir);

        // Succeeds if current dir
        let (input, value) = file_or_dir_name(CURRENT_DIR).unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsComponent::CurDir);

        // Succeeds if normal
        let (input, value) = file_or_dir_name(b"hello").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsComponent::Normal(b"hello"));

        // Succeeds if normal starting with '.'
        let (input, value) = file_or_dir_name(b".hello").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsComponent::Normal(b".hello"));

        // Succeeds if normal starting with '..'
        let (input, value) = file_or_dir_name(b"..hello").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsComponent::Normal(b"..hello"));

        // Succeeds if normal is exactly '...'
        let (input, value) = file_or_dir_name(b"...").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsComponent::Normal(b"..."));
    }

    #[test]
    fn validate_root_dir() {
        let root_dir = root_dir(separator);

        // Empty input fails
        root_dir(b"").unwrap_err();

        // Not starting with root dir fails
        root_dir(&[b'a', SEPARATOR as u8]).unwrap_err();

        // Succeeds just on its own
        let (input, value) = root_dir(&[SEPARATOR as u8]).unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsComponent::RootDir);

        // Succeeds, taking only what it matches
        let (input, value) = root_dir(&[SEPARATOR as u8, b'a', SEPARATOR as u8]).unwrap();
        assert_eq!(input, &[b'a', SEPARATOR as u8]);
        assert_eq!(value, WindowsComponent::RootDir);
    }

    #[test]
    fn validate_cur_dir() {
        let cur_dir = cur_dir(separator);

        // Empty input fails
        cur_dir(b"").unwrap_err();

        // Not starting with current dir fails
        cur_dir(&[&[b'a'], CURRENT_DIR].concat()).unwrap_err();

        // Succeeds just on its own
        let (input, value) = cur_dir(CURRENT_DIR).unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsComponent::CurDir);

        // Fails if more content after itself that is not a separator
        // E.g. .. will fail, .a will fail
        cur_dir(&[CURRENT_DIR, &[b'.']].concat()).unwrap_err();
        cur_dir(&[CURRENT_DIR, &[b'a']].concat()).unwrap_err();

        // Succeeds, taking only what it matches
        let input = &[CURRENT_DIR, &sep(1), CURRENT_DIR].concat();
        let (input, value) = cur_dir(input).unwrap();
        assert_eq!(input, &[&sep(1), CURRENT_DIR].concat());
        assert_eq!(value, WindowsComponent::CurDir);
    }

    #[test]
    fn validate_parent_dir() {
        let parent_dir = parent_dir(separator);

        // Empty input fails
        parent_dir(b"").unwrap_err();

        // Not starting with parent dir fails
        parent_dir(&[&[b'a'], PARENT_DIR].concat()).unwrap_err();

        // Succeeds just on its own
        let (input, value) = parent_dir(PARENT_DIR).unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsComponent::ParentDir);

        // Fails if more content after itself that is not a separator
        // E.g. ... will fail, ..a will fail
        parent_dir(&[PARENT_DIR, &[b'.']].concat()).unwrap_err();
        parent_dir(&[PARENT_DIR, &[b'a']].concat()).unwrap_err();

        // Succeeds, taking only what it matches
        let input = &[PARENT_DIR, &sep(1), PARENT_DIR].concat();
        let (input, value) = parent_dir(input).unwrap();
        assert_eq!(input, &[&sep(1), PARENT_DIR].concat());
        assert_eq!(value, WindowsComponent::ParentDir);
    }

    #[test]
    fn validate_normal() {
        // Empty input fails
        normal(separator)(b"").unwrap_err();

        // Fails if takes nothing
        normal(separator)(br"\a").unwrap_err();

        // Succeeds just on its own
        let (input, value) = normal(separator)(b"hello").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsComponent::Normal(b"hello"));

        // Succeeds, taking up to the next separator
        let (input, value) = normal(separator)(br"abc\").unwrap();
        assert_eq!(input, br"\");
        assert_eq!(value, WindowsComponent::Normal(b"abc"));

        let (input, value) = normal(separator)(br"abc/").unwrap();
        assert_eq!(input, br"/");
        assert_eq!(value, WindowsComponent::Normal(b"abc"));

        let (input, value) = normal(verbatim_separator)(br"abc\").unwrap();
        assert_eq!(input, br"\");
        assert_eq!(value, WindowsComponent::Normal(b"abc"));

        let (input, value) = normal(verbatim_separator)(br"abc/").unwrap();
        assert_eq!(input, b"");
        assert_eq!(value, WindowsComponent::Normal(b"abc/"));
    }

    #[test]
    fn validate_separator() {
        // Empty input fails
        separator(b"").unwrap_err();

        // Not starting with separator fails
        separator(&[b'a', SEPARATOR as u8]).unwrap_err();

        // Succeeds just on its own with primary
        let (input, _) = separator(&[SEPARATOR as u8]).unwrap();
        assert_eq!(input, b"");

        // Succeeds just on its own with alternate
        let (input, _) = separator(&[ALT_SEPARATOR as u8]).unwrap();
        assert_eq!(input, b"");

        // Succeeds, taking only one separator
        let input = &[SEPARATOR as u8, SEPARATOR as u8];
        let (input, _) = separator(input).unwrap();
        assert_eq!(input, &[SEPARATOR as u8]);

        // Succeeds, taking only one alternate separator
        let input = &[ALT_SEPARATOR as u8, ALT_SEPARATOR as u8];
        let (input, _) = separator(input).unwrap();
        assert_eq!(input, &[ALT_SEPARATOR as u8]);
    }
}
