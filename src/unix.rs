mod components;
mod constants;

pub use components::*;
pub use constants::*;

use crate::{private, Components, Encoding, Path, PathBuf};

/// Represents a Unix-specific [`Path`]
pub type UnixPath = Path<UnixEncoding>;

/// Represents a Unix-specific [`PathBuf`]
pub type UnixPathBuf = PathBuf<UnixEncoding>;

/// Represents a Unix-specific [`Encoding`]
#[derive(Copy, Clone)]
pub struct UnixEncoding;

impl private::Sealed for UnixEncoding {}

impl<'a> Encoding<'a> for UnixEncoding {
    type Components = UnixComponents<'a>;

    fn components(path: &'a [u8]) -> Self::Components {
        UnixComponents::new(path)
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
