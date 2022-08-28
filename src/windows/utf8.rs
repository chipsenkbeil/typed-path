mod components;

pub use components::*;

use crate::{private, Encoding, Utf8Encoding, Utf8Path, Utf8PathBuf, WindowsEncoding};
use std::{fmt, hash::Hasher};

/// Represents a Windows-specific [`Utf8Path`]
pub type Utf8WindowsPath = Utf8Path<Utf8WindowsEncoding>;

/// Represents a Windows-specific [`Utf8PathBuf`]
pub type Utf8WindowsPathBuf = Utf8PathBuf<Utf8WindowsEncoding>;

/// Represents a Windows-specific [`Utf8Encoding`]
pub struct Utf8WindowsEncoding;

impl private::Sealed for Utf8WindowsEncoding {}

impl<'a> Utf8Encoding<'a> for Utf8WindowsEncoding {
    type Components = Utf8WindowsComponents<'a>;

    fn components(path: &'a str) -> Self::Components {
        Utf8WindowsComponents::new(path)
    }

    fn hash<H: Hasher>(path: &str, h: &mut H) {
        WindowsEncoding::hash(path.as_bytes(), h);
    }

    fn push(current_path: &mut String, path: &str) {
        unsafe {
            WindowsEncoding::push(current_path.as_mut_vec(), path.as_bytes());
        }
    }
}

impl fmt::Debug for Utf8WindowsEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Utf8WindowsEncoding").finish()
    }
}

impl fmt::Display for Utf8WindowsEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Utf8WindowsEncoding")
    }
}
