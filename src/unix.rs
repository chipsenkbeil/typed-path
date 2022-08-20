mod component;
mod constants;
mod parser;

pub use component::*;
pub use constants::*;

use crate::{CharSeparator, Components, Encoding, Path, PathBuf};
use std::fmt;

/// Represents a Unix-specific [`Path`]
pub type UnixPath = Path<UnixEncoding>;

/// Represents a Unix-specific [`PathBuf`]
pub type UnixPathBuf = PathBuf<UnixEncoding>;

/// Represents a Unix-specific [`Components`]
pub type UnixComponents<'a> = Components<'a, UnixEncoding>;

/// Represents a Unix-specific [`Encoding`]
pub struct UnixEncoding;

impl<'a> Encoding<'a> for UnixEncoding {
    type Component = UnixComponent<'a>;
    type Separator = CharSeparator<SEPARATOR>;

    fn components(bytes: &'a [u8]) -> Components<'a, Self> {
        parser::parse(bytes)
    }
}

impl<'a> Components<'a, UnixEncoding> {
    /// Extracts a slice corresponding to the portion of the path remaining for iteration
    pub fn as_path(&self) -> &'a UnixPath {
        UnixPath::new(self.raw)
    }
}

impl fmt::Debug for Components<'_, UnixEncoding> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a>(&'a UnixPath);

        impl fmt::Debug for DebugHelper<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.components()).finish()
            }
        }

        f.debug_tuple(stringify!([<$platform:camel Components>]))
            .field(&DebugHelper(self.as_path()))
            .finish()
    }
}
