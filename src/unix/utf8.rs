mod components;

pub use components::*;

use crate::{private, Encoding, UnixEncoding, Utf8Encoding, Utf8Path, Utf8PathBuf};
use std::{fmt, hash::Hasher};

/// Represents a Unix-specific [`Utf8Path`]
pub type Utf8UnixPath = Utf8Path<Utf8UnixEncoding>;

/// Represents a Unix-specific [`Utf8PathBuf`]
pub type Utf8UnixPathBuf = Utf8PathBuf<Utf8UnixEncoding>;

/// Represents a Unix-specific [`Utf8Encoding`]
pub struct Utf8UnixEncoding;

impl private::Sealed for Utf8UnixEncoding {}

impl<'a> Utf8Encoding<'a> for Utf8UnixEncoding {
    type Components = Utf8UnixComponents<'a>;

    fn label() -> &'static str {
        "unix"
    }

    fn components(path: &'a str) -> Self::Components {
        Utf8UnixComponents::new(path)
    }

    fn hash<H: Hasher>(path: &str, h: &mut H) {
        UnixEncoding::hash(path.as_bytes(), h);
    }

    fn push(current_path: &mut String, path: &str) {
        unsafe {
            UnixEncoding::push(current_path.as_mut_vec(), path.as_bytes());
        }
    }
}

impl fmt::Debug for Utf8UnixEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Utf8UnixEncoding").finish()
    }
}

impl fmt::Display for Utf8UnixEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Utf8UnixEncoding")
    }
}
