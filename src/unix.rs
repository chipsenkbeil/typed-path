mod component;
mod constants;
mod parser;

pub use component::*;
pub use constants::*;

use crate::{CharSeparator, Components, Encoding, Path, PathBuf, Separator};

/// Represents a Unix-specific [`Path`]
pub type UnixPath = Path<UnixEncoding>;

/// Represents a Unix-specific [`PathBuf`]
pub type UnixPathBuf = PathBuf<UnixEncoding>;

/// Represents a Unix-specific [`Components`]
pub type UnixComponents<'a> = Components<'a, UnixEncoding>;

/// Represents a Unix-specific [`Encoding`]
#[derive(Copy, Clone)]
pub struct UnixEncoding;

impl<'a> Encoding<'a> for UnixEncoding {
    type Component = UnixComponent<'a>;
    type Separator = CharSeparator<SEPARATOR>;

    fn push(bytes: &mut Vec<u8>, path: &[u8]) {
        if path.is_empty() {
            return;
        }

        // Absolute path will replace entirely, otherwise check if we need to add our separator,
        // and add it if the separator is missing
        if Self::is_absolute(path) {
            bytes.clear();
        } else if !Self::Separator::is_at_end_of(bytes) {
            bytes.extend_from_slice(Self::Separator::as_bytes());
        }

        bytes.extend_from_slice(path);
    }

    fn components(bytes: &'a [u8]) -> Components<'a, Self> {
        parser::parse(bytes).expect("TODO: Fix this panic")
    }

    fn is_absolute(bytes: &[u8]) -> bool {
        Self::has_root(bytes)
    }

    fn has_root(bytes: &[u8]) -> bool {
        matches!(Self::components(bytes).next(), Some(UnixComponent::RootDir))
    }
}
