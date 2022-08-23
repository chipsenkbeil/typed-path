mod components;
mod constants;

pub use components::*;
pub use constants::*;

use crate::{private, Components, Encoding, Path, PathBuf, Separator};

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
        if Self::is_absolute(path) {
            current_path.clear();
        } else if !Self::Separator::is_at_end_of(current_path) {
            current_path.extend_from_slice(Self::Separator::as_primary_bytes());
        }

        current_path.extend_from_slice(path);
    }
}
