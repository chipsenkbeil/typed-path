mod components;
mod constants;

pub use components::*;
pub use constants::*;

use crate::{private, Components, Encoding, Path, PathBuf};
use std::{fmt, hash::Hasher};

/// Represents a Unix-specific [`Path`]
pub type UnixPath = Path<UnixEncoding>;

/// Represents a Unix-specific [`PathBuf`]
pub type UnixPathBuf = PathBuf<UnixEncoding>;

/// Represents a Unix-specific [`Encoding`]
pub struct UnixEncoding;

impl private::Sealed for UnixEncoding {}

impl<'a> Encoding<'a> for UnixEncoding {
    type Components = UnixComponents<'a>;

    fn components(path: &'a [u8]) -> Self::Components {
        UnixComponents::new(path)
    }

    fn hash<H: Hasher>(path: &[u8], h: &mut H) {
        let mut component_start = 0;
        let mut bytes_hashed = 0;

        for i in 0..path.len() {
            let is_sep = path[i] == SEPARATOR as u8;
            if is_sep {
                if i > component_start {
                    let to_hash = &path[component_start..i];
                    h.write(to_hash);
                    bytes_hashed += to_hash.len();
                }

                // skip over separator and optionally a following CurDir item
                // since components() would normalize these away.
                component_start = i + 1;

                let tail = &path[component_start..];

                component_start += match tail {
                    [b'.'] => 1,
                    [b'.', sep, ..] if *sep == SEPARATOR as u8 => 1,
                    _ => 0,
                };
            }
        }

        if component_start < path.len() {
            let to_hash = &path[component_start..];
            h.write(to_hash);
            bytes_hashed += to_hash.len();
        }

        h.write_usize(bytes_hashed);
    }

    fn push(current_path: &mut Vec<u8>, path: &[u8]) {
        if path.is_empty() {
            return;
        }

        // Absolute path will replace entirely, otherwise check if we need to add our separator,
        // and add it if the separator is missing
        if Self::components(path).is_absolute() {
            current_path.clear();
        } else if !current_path.ends_with(&[SEPARATOR as u8]) {
            current_path.push(SEPARATOR as u8);
        }

        current_path.extend_from_slice(path);
    }
}

impl fmt::Debug for UnixEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnixEncoding").finish()
    }
}

impl fmt::Display for UnixEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UnixEncoding")
    }
}
