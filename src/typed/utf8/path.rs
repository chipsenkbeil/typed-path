use std::path::Path;

use crate::convert::TryAsRef;
use crate::typed::Utf8TypedPathBuf;
use crate::unix::Utf8UnixPath;
use crate::windows::Utf8WindowsPath;

/// Represents a UTF-8 path with a known type that can be one of:
///
/// * [`Utf8UnixPath`]
/// * [`Utf8WindowsPath`]
pub enum Utf8TypedPath<'a> {
    Unix(&'a Utf8UnixPath),
    Windows(&'a Utf8WindowsPath),
}

impl<'a> Utf8TypedPath<'a> {
    /// Creates a new UTF* typed path from a byte slice by determining if the path represents a
    /// Windows or Unix path. This is accomplished by first trying to parse as a Windows path. If
    /// the resulting path contains a prefix such as `C:` or begins with a `\`, it is assumed to be
    /// a [`Utf8WindowsPath`]; otherwise, the slice will be represented as a [`Utf8UnixPath`].
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::Utf8TypedPath;
    ///
    /// assert!(Utf8TypedPath::new(r#"C:\some\path\to\file.txt"#).is_windows());
    /// assert!(Utf8TypedPath::new(r#"\some\path\to\file.txt"#).is_windows());
    /// assert!(Utf8TypedPath::new(r#"/some/path/to/file.txt"#).is_unix());
    ///
    /// // NOTE: If we don't start with a backslash, it's too difficult to
    /// //       determine and we therefore just assume a Unix/POSIX path.
    /// assert!(Utf8TypedPath::new(r#"some\path\to\file.txt"#).is_unix());
    /// assert!(Utf8TypedPath::new("file.txt").is_unix());
    /// assert!(Utf8TypedPath::new("").is_unix());
    /// ```
    pub fn new<S: AsRef<str> + ?Sized>(s: &'a S) -> Self {
        let winpath = Utf8WindowsPath::new(s.as_ref());
        if winpath.components().has_prefix() || s.as_ref().starts_with('\\') {
            Self::Windows(winpath)
        } else {
            Self::Unix(Utf8UnixPath::new(s))
        }
    }

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

    /// Converts into a [`Utf8TypedPathBuf`].
    pub fn to_path_buf(&self) -> Utf8TypedPathBuf {
        match self {
            Self::Unix(path) => Utf8TypedPathBuf::Unix(path.to_path_buf()),
            Self::Windows(path) => Utf8TypedPathBuf::Windows(path.to_path_buf()),
        }
    }
}

impl<'a> From<&'a str> for Utf8TypedPath<'a> {
    #[inline]
    fn from(s: &'a str) -> Self {
        Utf8TypedPath::new(s)
    }
}

impl TryAsRef<Utf8UnixPath> for Utf8TypedPath<'_> {
    fn try_as_ref(&self) -> Option<&Utf8UnixPath> {
        match self {
            Self::Unix(path) => Some(path),
            _ => None,
        }
    }
}

impl TryAsRef<Utf8WindowsPath> for Utf8TypedPath<'_> {
    fn try_as_ref(&self) -> Option<&Utf8WindowsPath> {
        match self {
            Self::Windows(path) => Some(path),
            _ => None,
        }
    }
}

impl<'a> TryAsRef<Path> for Utf8TypedPath<'a> {
    fn try_as_ref(&self) -> Option<&Path> {
        match self {
            #[cfg(unix)]
            Self::Unix(path) => Some(AsRef::<Path>::as_ref(path)),
            #[cfg(windows)]
            Self::Windows(path) => Some(AsRef::<Path>::as_ref(path)),
            _ => None,
        }
    }
}
