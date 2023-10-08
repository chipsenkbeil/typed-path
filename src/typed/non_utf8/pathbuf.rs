use std::collections::TryReserveError;
use std::convert::TryFrom;
use std::path::PathBuf;

use crate::typed::TypedPath;
use crate::unix::UnixPathBuf;
use crate::windows::WindowsPathBuf;

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

    /// Allocates an empty [`TypedPathBuf`] for a Unix path.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPathBuf;
    /// let path = TypedPathBuf::new_unix();
    /// ```
    pub fn new_unix() -> Self {
        Self::Unix(UnixPathBuf::new())
    }

    /// Allocates an empty [`TypedPathBuf`] for a Windows path.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPathBuf;
    /// let path = TypedPathBuf::new_windows();
    /// ```
    pub fn new_windows() -> Self {
        Self::Windows(WindowsPathBuf::new())
    }

    /// Creates a new [`TypedPathBuf`] from the bytes representing a Unix path.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPathBuf;
    /// let path = TypedPathBuf::from_unix("/tmp");
    /// ```
    pub fn from_unix(s: impl AsRef<[u8]>) -> Self {
        Self::Unix(UnixPathBuf::from(s.as_ref()))
    }

    /// Creates a new [`TypedPathBuf`] from the bytes representing a Windows path.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPathBuf;
    /// let path = TypedPathBuf::from_windows(r"C:\tmp");
    /// ```
    pub fn from_windows(s: impl AsRef<[u8]>) -> Self {
        Self::Windows(WindowsPathBuf::from(s.as_ref()))
    }

    /// Converts into a [`TypedPath`].
    pub fn as_path(&self) -> TypedPath {
        match self {
            Self::Unix(path) => TypedPath::Unix(path.as_path()),
            Self::Windows(path) => TypedPath::Windows(path.as_path()),
        }
    }

    /// Extends `self` with `path`.
    ///
    /// If `path` is absolute, it replaces the current path.
    ///
    /// With [`WindowsPathBuf`]:
    ///
    /// * if `path` has a root but no prefix (e.g., `\windows`), it
    ///   replaces everything except for the prefix (if any) of `self`.
    /// * if `path` has a prefix but no root, it replaces `self`.
    /// * if `self` has a verbatim prefix (e.g. `\\?\C:\windows`)
    ///   and `path` is not empty, the new path is normalized: all references
    ///   to `.` and `..` are removed.
    ///
    /// [`WindowsPathBuf`]: crate::WindowsPathBuf
    ///
    /// # Examples
    ///
    /// Pushing a relative path extends the existing path:
    ///
    /// ```
    /// use typed_path::TypedPathBuf;
    ///
    /// let mut path = TypedPathBuf::from_unix("/tmp");
    /// path.push("file.bk");
    /// assert_eq!(path, TypedPathBuf::from_unix("/tmp/file.bk"));
    /// ```
    ///
    /// Pushing an absolute path replaces the existing path:
    ///
    /// ```
    /// use typed_path::TypedPathBuf;
    ///
    /// let mut path = TypedPathBuf::from_unix("/tmp");
    /// path.push("/etc");
    /// assert_eq!(path, TypedPathBuf::from_unix("/etc"));
    /// ```
    pub fn push<'a, P: AsRef<TypedPath<'a>>>(&mut self, path: P) {
        match (self, path.as_ref()) {
            (Self::Unix(a), TypedPath::Unix(b)) => a.push(b),
            (Self::Unix(a), TypedPath::Windows(b)) => a.push(b.with_unix_encoding()),
            (Self::Windows(a), TypedPath::Windows(b)) => a.push(b),
            (Self::Windows(a), TypedPath::Unix(b)) => a.push(b.with_windows_encoding()),
        }
    }

    /// Truncates `self` to [`self.parent`].
    ///
    /// Returns `false` and does nothing if [`self.parent`] is [`None`].
    /// Otherwise, returns `true`.
    ///
    /// [`self.parent`]: TypedPath::parent
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{TypedPath, TypedPathBuf};
    ///
    /// let mut p = TypedPathBuf::from_unix("/spirited/away.rs");
    ///
    /// p.pop();
    /// assert_eq!(TypedPath::new("/spirited"), p);
    /// p.pop();
    /// assert_eq!(TypedPath::new("/"), p);
    /// ```
    pub fn pop(&mut self) -> bool {
        impl_typed_fn!(self, pop)
    }

    /// Updates [`self.file_name`] to `file_name`.
    ///
    /// If [`self.file_name`] was [`None`], this is equivalent to pushing
    /// `file_name`.
    ///
    /// Otherwise it is equivalent to calling [`pop`] and then pushing
    /// `file_name`. The new path will be a sibling of the original path.
    /// (That is, it will have the same parent.)
    ///
    /// [`self.file_name`]: TypedPath::file_name
    /// [`pop`]: TypedPathBuf::pop
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPathBuf;
    ///
    /// let mut buf = TypedPathBuf::from_unix("/");
    /// assert!(buf.file_name() == None);
    /// buf.set_file_name("bar");
    /// assert!(buf == TypedPathBuf::from_unix("/bar"));
    /// assert!(buf.file_name().is_some());
    /// buf.set_file_name("baz.txt");
    /// assert!(buf == TypedPathBuf::from_unix("/baz.txt"));
    /// ```
    pub fn set_file_name<S: AsRef<[u8]>>(&mut self, file_name: S) {
        impl_typed_fn!(self, set_file_name, file_name)
    }

    /// Updates [`self.extension`] to `extension`.
    ///
    /// Returns `false` and does nothing if [`self.file_name`] is [`None`],
    /// returns `true` and updates the extension otherwise.
    ///
    /// If [`self.extension`] is [`None`], the extension is added; otherwise
    /// it is replaced.
    ///
    /// [`self.file_name`]: TypedPath::file_name
    /// [`self.extension`]: TypedPath::extension
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{TypedPath, TypedPathBuf};
    ///
    /// let mut p = TypedPathBuf::from_unix("/feel/the");
    ///
    /// p.set_extension("force");
    /// assert_eq!(TypedPath::new("/feel/the.force"), p.as_path());
    ///
    /// p.set_extension("dark_side");
    /// assert_eq!(TypedPath::new("/feel/the.dark_side"), p.as_path());
    /// ```
    pub fn set_extension<S: AsRef<[u8]>>(&mut self, extension: S) -> bool {
        impl_typed_fn!(self, set_extension, extension)
    }

    /// Consumes the [`TypedPathBuf`], yielding its internal [`Vec<u8>`] storage.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPathBuf;
    ///
    /// let p = TypedPathBuf::from_unix("/the/head");
    /// let vec = p.into_vec();
    /// ```
    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        impl_typed_fn!(self, into_vec)
    }

    /// Invokes [`capacity`] on the underlying instance of [`Vec`].
    ///
    /// [`capacity`]: Vec::capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        impl_typed_fn!(self, capacity)
    }

    /// Invokes [`clear`] on the underlying instance of [`Vec`].
    ///
    /// [`clear`]: Vec::clear
    #[inline]
    pub fn clear(&mut self) {
        impl_typed_fn!(self, clear)
    }

    /// Invokes [`reserve`] on the underlying instance of [`Vec`].
    ///
    /// [`reserve`]: Vec::reserve
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        impl_typed_fn!(self, reserve, additional)
    }

    /// Invokes [`try_reserve`] on the underlying instance of [`Vec`].
    ///
    /// [`try_reserve`]: Vec::try_reserve
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        impl_typed_fn!(self, try_reserve, additional)
    }

    /// Invokes [`reserve_exact`] on the underlying instance of [`Vec`].
    ///
    /// [`reserve_exact`]: Vec::reserve_exact
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        impl_typed_fn!(self, reserve_exact, additional)
    }

    /// Invokes [`try_reserve_exact`] on the underlying instance of [`Vec`].
    ///
    /// [`try_reserve_exact`]: Vec::try_reserve_exact
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        impl_typed_fn!(self, try_reserve_exact, additional)
    }

    /// Invokes [`shrink_to_fit`] on the underlying instance of [`Vec`].
    ///
    /// [`shrink_to_fit`]: Vec::shrink_to_fit
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        impl_typed_fn!(self, shrink_to_fit)
    }

    /// Invokes [`shrink_to`] on the underlying instance of [`Vec`].
    ///
    /// [`shrink_to`]: Vec::shrink_to
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        impl_typed_fn!(self, shrink_to, min_capacity)
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

impl From<Vec<u8>> for TypedPathBuf {
    #[inline]
    fn from(s: Vec<u8>) -> Self {
        // NOTE: We use the typed path to check the underlying format, and then
        //       create it manually to avoid a clone of the vec itself
        match TypedPath::new(s.as_slice()) {
            TypedPath::Unix(_) => TypedPathBuf::Unix(UnixPathBuf::from(s)),
            TypedPath::Windows(_) => TypedPathBuf::Windows(WindowsPathBuf::from(s)),
        }
    }
}

impl<'a> From<&'a str> for TypedPathBuf {
    #[inline]
    fn from(s: &'a str) -> Self {
        TypedPathBuf::from(s.as_bytes())
    }
}

impl From<String> for TypedPathBuf {
    #[inline]
    fn from(s: String) -> Self {
        // NOTE: We use the typed path to check the underlying format, and then
        //       create it manually to avoid a clone of the string itself
        match TypedPath::new(s.as_bytes()) {
            TypedPath::Unix(_) => TypedPathBuf::Unix(UnixPathBuf::from(s)),
            TypedPath::Windows(_) => TypedPathBuf::Windows(WindowsPathBuf::from(s)),
        }
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
