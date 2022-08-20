mod component;
mod constants;
mod parser;

pub use component::*;
pub use constants::*;

use crate::{CharSeparator, Components, Encoding, Path, PathBuf};
use std::fmt;

/// Represents a Windows-specific [`Path`]
pub type WindowsPath = Path<WindowsEncoding>;

/// Represents a Windows-specific [`PathBuf`]
pub type WindowsPathBuf = PathBuf<WindowsEncoding>;

/// Represents a Windows-specific [`Components`]
pub type WindowsComponents<'a> = Components<'a, WindowsEncoding>;

/// Represents a Windows-specific [`Encoding`]
pub struct WindowsEncoding;

impl<'a> Encoding<'a> for WindowsEncoding {
    type Component = WindowsComponent<'a>;
    type Separator = CharSeparator<SEPARATOR>;

    fn components(bytes: &'a [u8]) -> Components<'a, Self> {
        parser::parse(bytes)
    }
}

impl<'a> Components<'a, WindowsEncoding> {
    /// Extracts a slice corresponding to the portion of the path remaining for iteration
    pub fn as_path(&self) -> &'a WindowsPath {
        WindowsPath::new(self.raw)
    }
}

impl fmt::Debug for Components<'_, WindowsEncoding> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a>(&'a WindowsPath);

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
