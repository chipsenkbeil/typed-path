mod component;
mod constants;
mod parser;

pub use component::*;
pub use constants::*;

use crate::{CharSeparator, Components, Encoding, Path, PathBuf};

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

    fn components(bytes: &'a [u8]) -> Components<'a, Self> {
        parser::parse(bytes).expect("TODO: Fix this panic")
    }

    fn is_absolute(bytes: &'a [u8]) -> bool {
        Self::has_root(bytes)
    }

    fn has_root(bytes: &'a [u8]) -> bool {
        match Self::components(bytes).next() {
            Some(UnixComponent::RootDir) => true,
            _ => false,
        }
    }
}
