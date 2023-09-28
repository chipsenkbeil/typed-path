use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use crate::convert::TryAsRef;
use crate::unix::{UnixPath, UnixPathBuf, Utf8UnixPath, Utf8UnixPathBuf};
use crate::windows::{Utf8WindowsPath, Utf8WindowsPathBuf, WindowsPath, WindowsPathBuf};

/// Represents a path with a known type that can be one of:
///
/// * [`UnixPath`]
/// * [`WindowsPath`]
pub enum TypedPath<'a> {
    Unix(&'a UnixPath),
    Windows(&'a WindowsPath),
}

impl<'a> TypedPath<'a> {
    /// Creates a new typed path from a byte slice by determining if the path represents a Windows
    /// or Unix path. This is accomplished by first trying to parse as a Windows path. If the
    /// resulting path contains a prefix such as `C:` or begins with a `\`, it is assumed to be a
    /// [`WindowsPath`]; otherwise, the slice will be represented as a [`UnixPath`].
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// assert!(TypedPath::new(br#"C:\some\path\to\file.txt"#).is_windows());
    /// assert!(TypedPath::new(br#"\some\path\to\file.txt"#).is_windows());
    /// assert!(TypedPath::new(br#"/some/path/to/file.txt"#).is_unix());
    ///
    /// // NOTE: If we don't start with a backslash, it's too difficult to
    /// //       determine and we therefore just assume a Unix/POSIX path.
    /// assert!(TypedPath::new(br#"some\path\to\file.txt"#).is_unix());
    /// assert!(TypedPath::new(b"file.txt").is_unix());
    /// assert!(TypedPath::new(b"").is_unix());
    /// ```
    pub fn new(s: &'a [u8]) -> Self {
        let winpath = WindowsPath::new(s);
        if winpath.components().has_prefix() || s.first() == Some(&b'\\') {
            Self::Windows(winpath)
        } else {
            Self::Unix(UnixPath::new(s))
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

    /// Converts into a [`TypedPathBuf`].
    pub fn to_path_buf(&self) -> TypedPathBuf {
        match self {
            Self::Unix(path) => TypedPathBuf::Unix(path.to_path_buf()),
            Self::Windows(path) => TypedPathBuf::Windows(path.to_path_buf()),
        }
    }
}

impl TryAsRef<UnixPath> for TypedPath<'_> {
    fn try_as_ref(&self) -> Option<&UnixPath> {
        match self {
            Self::Unix(path) => Some(path),
            _ => None,
        }
    }
}

impl TryAsRef<WindowsPath> for TypedPath<'_> {
    fn try_as_ref(&self) -> Option<&WindowsPath> {
        match self {
            Self::Windows(path) => Some(path),
            _ => None,
        }
    }
}

/// Represents a pathbuf with a known type that can be one of:
///
/// * [`UnixPathBuf`]
/// * [`WindowsPathBuf`]
pub enum TypedPathBuf {
    Unix(UnixPathBuf),
    Windows(WindowsPathBuf),
}

impl TypedPathBuf {
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

    /// Converts into a [`TypedPath`].
    pub fn as_path(&self) -> TypedPath<'_> {
        match self {
            Self::Unix(path) => TypedPath::Unix(path.as_path()),
            Self::Windows(path) => TypedPath::Windows(path.as_path()),
        }
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for TypedPathBuf {
    #[inline]
    fn from(s: &'a [u8; N]) -> Self {
        TypedPathBuf::from(s.as_slice())
    }
}

impl<'a> From<&'a [u8]> for TypedPathBuf {
    /// Creates a new typed pathbuf from a byte slice by determining if the path represents a
    /// Windows or Unix path. This is accomplished by first trying to parse as a Windows path. If
    /// the resulting path contains a prefix such as `C:` or begins with a `\`, it is assumed to be
    /// a [`WindowsPathBuf`]; otherwise, the slice will be represented as a [`UnixPathBuf`].
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPathBuf;
    ///
    /// assert!(TypedPathBuf::from(br#"C:\some\path\to\file.txt"#).is_windows());
    /// assert!(TypedPathBuf::from(br#"\some\path\to\file.txt"#).is_windows());
    /// assert!(TypedPathBuf::from(br#"/some/path/to/file.txt"#).is_unix());
    ///
    /// // NOTE: If we don't start with a backslash, it's too difficult to
    /// //       determine and we therefore just assume a Unix/POSIX path.
    /// assert!(TypedPathBuf::from(br#"some\path\to\file.txt"#).is_unix());
    /// assert!(TypedPathBuf::from(b"file.txt").is_unix());
    /// assert!(TypedPathBuf::from(b"").is_unix());
    /// ```
    #[inline]
    fn from(s: &'a [u8]) -> Self {
        TypedPath::new(s).to_path_buf()
    }
}

impl TryFrom<TypedPathBuf> for UnixPathBuf {
    type Error = TypedPathBuf;

    fn try_from(path: TypedPathBuf) -> Result<Self, Self::Error> {
        match path {
            TypedPathBuf::Unix(path) => Ok(path),
            path => Err(path),
        }
    }
}

impl TryFrom<TypedPathBuf> for WindowsPathBuf {
    type Error = TypedPathBuf;

    fn try_from(path: TypedPathBuf) -> Result<Self, Self::Error> {
        match path {
            TypedPathBuf::Windows(path) => Ok(path),
            path => Err(path),
        }
    }
}

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
    pub fn new(s: &'a str) -> Self {
        let winpath = Utf8WindowsPath::new(s);
        if winpath.components().has_prefix() || s.starts_with('\\') {
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
    /// Creates a new UTF-8 typed pathbuf from a byte slice by determining if the path represents a
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

impl<'a> TryAsRef<Path> for TypedPath<'a> {
    fn try_as_ref(&self) -> Option<&Path> {
        match self {
            Self::Unix(path) => path.try_as_ref(),
            Self::Windows(path) => path.try_as_ref(),
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

impl TryFrom<TypedPathBuf> for PathBuf {
    type Error = TypedPathBuf;

    fn try_from(path: TypedPathBuf) -> Result<Self, Self::Error> {
        match path {
            #[cfg(unix)]
            TypedPathBuf::Unix(path) => PathBuf::try_from(path).map_err(TypedPathBuf::Unix),
            #[cfg(windows)]
            TypedPathBuf::Windows(path) => PathBuf::try_from(path).map_err(TypedPathBuf::Windows),
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
