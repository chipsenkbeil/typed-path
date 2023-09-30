use std::convert::TryFrom;
use std::path::PathBuf;

use crate::typed::Utf8TypedPath;
use crate::unix::Utf8UnixPathBuf;
use crate::windows::Utf8WindowsPathBuf;

/// Represents a UTF-8 pathbuf with a known type that can be one of:
///
/// * [`Utf8UnixPathBuf`]
/// * [`Utf8WindowsPathBuf`]
pub enum Utf8TypedPathBuf {
    Unix(Utf8UnixPathBuf),
    Windows(Utf8WindowsPathBuf),
}

impl Utf8TypedPathBuf {
    /// Returns true if this path represents a Unix path.
    #[inline]
    pub fn is_unix(&self) -> bool {
        matches!(self, Self::Unix(_))
    }

    /// Returns true if this path represents a Windows path.
    #[inline]
    pub fn is_windows(&self) -> bool {
        matches!(self, Self::Windows(_))
    }

    /// Converts into a [`Utf8TypedPath`].
    pub fn as_path(&self) -> Utf8TypedPath<'_> {
        match self {
            Self::Unix(path) => Utf8TypedPath::Unix(path.as_path()),
            Self::Windows(path) => Utf8TypedPath::Windows(path.as_path()),
        }
    }
}

impl<'a> From<&'a str> for Utf8TypedPathBuf {
    /// Creates a new UTF-8 typed pathbuf from a str slice by determining if the path represents a
    /// Windows or Unix path. This is accomplished by first trying to parse as a Windows path. If
    /// the resulting path contains a prefix such as `C:` or begins with a `\`, it is assumed to be
    /// a [`Utf8WindowsPathBuf`]; otherwise, the slice will be represented as a
    /// [`Utf8UnixPathBuf`].
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::Utf8TypedPathBuf;
    ///
    /// assert!(Utf8TypedPathBuf::from(r#"C:\some\path\to\file.txt"#).is_windows());
    /// assert!(Utf8TypedPathBuf::from(r#"\some\path\to\file.txt"#).is_windows());
    /// assert!(Utf8TypedPathBuf::from(r#"/some/path/to/file.txt"#).is_unix());
    ///
    /// // NOTE: If we don't start with a backslash, it's too difficult to
    /// //       determine and we therefore just assume a Unix/POSIX path.
    /// assert!(Utf8TypedPathBuf::from(r#"some\path\to\file.txt"#).is_unix());
    /// assert!(Utf8TypedPathBuf::from("file.txt").is_unix());
    /// assert!(Utf8TypedPathBuf::from("").is_unix());
    /// ```
    #[inline]
    fn from(s: &'a str) -> Self {
        Utf8TypedPath::new(s).to_path_buf()
    }
}

impl From<String> for Utf8TypedPathBuf {
    #[inline]
    fn from(s: String) -> Self {
        // NOTE: We use the typed path to check the underlying format, and then
        //       create it manually to avoid a clone of the string itself
        match Utf8TypedPath::new(s.as_str()) {
            Utf8TypedPath::Unix(_) => Utf8TypedPathBuf::Unix(Utf8UnixPathBuf::from(s)),
            Utf8TypedPath::Windows(_) => Utf8TypedPathBuf::Windows(Utf8WindowsPathBuf::from(s)),
        }
    }
}

impl TryFrom<Utf8TypedPathBuf> for Utf8UnixPathBuf {
    type Error = Utf8TypedPathBuf;

    fn try_from(path: Utf8TypedPathBuf) -> Result<Self, Self::Error> {
        match path {
            Utf8TypedPathBuf::Unix(path) => Ok(path),
            path => Err(path),
        }
    }
}

impl TryFrom<Utf8TypedPathBuf> for Utf8WindowsPathBuf {
    type Error = Utf8TypedPathBuf;

    fn try_from(path: Utf8TypedPathBuf) -> Result<Self, Self::Error> {
        match path {
            Utf8TypedPathBuf::Windows(path) => Ok(path),
            path => Err(path),
        }
    }
}

impl TryFrom<Utf8TypedPathBuf> for PathBuf {
    type Error = Utf8TypedPathBuf;

    fn try_from(path: Utf8TypedPathBuf) -> Result<Self, Self::Error> {
        match path {
            #[cfg(unix)]
            Utf8TypedPathBuf::Unix(path) => Ok(PathBuf::from(path)),
            #[cfg(windows)]
            Utf8TypedPathBuf::Windows(path) => Ok(PathBuf::from(path)),
            path => Err(path),
        }
    }
}
