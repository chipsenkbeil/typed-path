use std::borrow::{Cow, ToOwned};
use std::path::Path;
use std::{fmt, io};

use crate::common::StripPrefixError;
use crate::convert::TryAsRef;
use crate::typed::TypedPathBuf;
use crate::unix::UnixPath;
use crate::windows::WindowsPath;

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
    pub fn new<S: AsRef<[u8]> + ?Sized>(s: &'a S) -> Self {
        let winpath = WindowsPath::new(s);
        if winpath.components().has_prefix() || s.as_ref().first() == Some(&b'\\') {
            Self::Windows(winpath)
        } else {
            Self::Unix(UnixPath::new(s))
        }
    }

    /// Yields the underlying [`[u8]`] slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// let bytes = TypedPath::new("foo.txt").as_bytes();
    /// assert_eq!(bytes, b"foo.txt");
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        impl_typed_fn!(self, as_bytes)
    }

    /// Yields a [`&str`] slice if the [`TypedPath`] is valid unicode.
    ///
    /// This conversion may entail doing a check for UTF-8 validity.
    /// Note that validation is performed because non-UTF-8 strings are
    /// perfectly valid for some OS.
    ///
    /// [`&str`]: str
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// let path = TypedPath::new("foo.txt");
    /// assert_eq!(path.to_str(), Some("foo.txt"));
    /// ```
    #[inline]
    pub fn to_str(&self) -> Option<&str> {
        impl_typed_fn!(self, to_str)
    }

    /// Converts a [`TypedPath`] to a [`Cow<str>`].
    ///
    /// Any non-Unicode sequences are replaced with
    /// [`U+FFFD REPLACEMENT CHARACTER`][U+FFFD].
    ///
    /// [U+FFFD]: std::char::REPLACEMENT_CHARACTER
    ///
    /// # Examples
    ///
    /// Calling `to_string_lossy` on a [`TypedPath`] with valid unicode:
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// // NOTE: A path cannot be created on its own without a defined encoding
    /// let path = TypedPath::new("foo.txt");
    /// assert_eq!(path.to_string_lossy(), "foo.txt");
    /// ```
    ///
    /// Had `path` contained invalid unicode, the `to_string_lossy` call might
    /// have returned `"fo�.txt"`.
    #[inline]
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        impl_typed_fn!(self, to_string_lossy)
    }

    /// Converts a [`TypedPath`] into a [`TypedPathBuf`].
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{TypedPath, TypedPathBuf};
    ///
    /// let path_buf = TypedPath::new("foo.txt").to_path_buf();
    /// assert_eq!(path_buf, TypedPathBuf::from("foo.txt"));
    /// ```
    pub fn to_path_buf(&self) -> TypedPathBuf {
        match self {
            Self::Unix(path) => TypedPathBuf::Unix(path.to_path_buf()),
            Self::Windows(path) => TypedPathBuf::Windows(path.to_path_buf()),
        }
    }

    /// Returns `true` if the [`TypedPath`] is absolute, i.e., if it is independent of
    /// the current directory.
    ///
    /// * On Unix ([`UnixPath`]]), a path is absolute if it starts with the root, so
    /// `is_absolute` and [`has_root`] are equivalent.
    ///
    /// * On Windows ([`WindowsPath`]), a path is absolute if it has a prefix and starts with the
    /// root: `c:\windows` is absolute, while `c:temp` and `\temp` are not.
    ///
    /// [`UnixPath`]: crate::UnixPath
    /// [`WindowsPath`]: crate::WindowsPath
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// assert!(!TypedPath::new("foo.txt").is_absolute());
    /// ```
    ///
    /// [`has_root`]: TypedPath::has_root
    pub fn is_absolute(&self) -> bool {
        impl_typed_fn!(self, is_absolute)
    }

    /// Returns `true` if the [`TypedPath`] is relative, i.e., not absolute.
    ///
    /// See [`is_absolute`]'s documentation for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath};
    ///
    /// assert!(TypedPath::new("foo.txt").is_relative());
    /// ```
    ///
    /// [`is_absolute`]: TypedPath::is_absolute
    #[inline]
    pub fn is_relative(&self) -> bool {
        impl_typed_fn!(self, is_relative)
    }

    /// Returns `true` if the [`TypedPath`] has a root.
    ///
    /// * On Unix ([`UnixPath`]), a path has a root if it begins with `/`.
    ///
    /// * On Windows ([`WindowsPath`]), a path has a root if it:
    ///     * has no prefix and begins with a separator, e.g., `\windows`
    ///     * has a prefix followed by a separator, e.g., `c:\windows` but not `c:windows`
    ///     * has any non-disk prefix, e.g., `\\server\share`
    ///
    /// [`UnixPath`]: crate::UnixPath
    /// [`WindowsPath`]: crate::WindowsPath
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath};
    ///
    /// assert!(TypedPath::new("/etc/passwd").has_root());
    /// ```
    #[inline]
    pub fn has_root(&self) -> bool {
        impl_typed_fn!(self, has_root)
    }

    /// Returns the [`TypedPath`] without its final component, if there is one.
    ///
    /// Returns [`None`] if the path terminates in a root or prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// let path = TypedPath::new("/foo/bar");
    /// let parent = path.parent().unwrap();
    /// assert_eq!(parent, TypedPath::new("/foo"));
    ///
    /// let grand_parent = parent.parent().unwrap();
    /// assert_eq!(grand_parent, TypedPath::new("/"));
    /// assert_eq!(grand_parent.parent(), None);
    /// ```
    pub fn parent(&self) -> Option<Self> {
        match self {
            Self::Unix(path) => path.parent().map(Self::Unix),
            Self::Windows(path) => path.parent().map(Self::Windows),
        }
    }

    /// Produces an iterator over [`TypedPath`] and its ancestors.
    ///
    /// The iterator will yield the [`TypedPath`] that is returned if the [`parent`] method is used
    /// zero or more times. That means, the iterator will yield `&self`, `&self.parent().unwrap()`,
    /// `&self.parent().unwrap().parent().unwrap()` and so on. If the [`parent`] method returns
    /// [`None`], the iterator will do likewise. The iterator will always yield at least one value,
    /// namely `&self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// let mut ancestors = TypedPath::new("/foo/bar").ancestors();
    /// assert_eq!(ancestors.next(), Some(TypedPath::new("/foo/bar")));
    /// assert_eq!(ancestors.next(), Some(TypedPath::new("/foo")));
    /// assert_eq!(ancestors.next(), Some(TypedPath::new("/")));
    /// assert_eq!(ancestors.next(), None);
    ///
    /// let mut ancestors = TypedPath::new("../foo/bar").ancestors();
    /// assert_eq!(ancestors.next(), Some(TypedPath::new("../foo/bar")));
    /// assert_eq!(ancestors.next(), Some(TypedPath::new("../foo")));
    /// assert_eq!(ancestors.next(), Some(TypedPath::new("..")));
    /// assert_eq!(ancestors.next(), Some(TypedPath::new("")));
    /// assert_eq!(ancestors.next(), None);
    /// ```
    ///
    /// [`parent`]: TypedPath::parent
    #[inline]
    pub fn ancestors(&self) -> impl Iterator<Item = TypedPath> {
        todo!()
    }

    /// Returns the final component of the [`TypedPath`], if there is one.
    ///
    /// If the path is a normal file, this is the file name. If it's the path of a directory, this
    /// is the directory name.
    ///
    /// Returns [`None`] if the path terminates in `..`.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// assert_eq!(Some(b"bin".as_slice()), TypedPath::new("/usr/bin/").file_name());
    /// assert_eq!(Some(b"foo.txt".as_slice()), TypedPath::new("tmp/foo.txt").file_name());
    /// assert_eq!(Some(b"foo.txt".as_slice()), TypedPath::new("foo.txt/.").file_name());
    /// assert_eq!(Some(b"foo.txt".as_slice()), TypedPath::new("foo.txt/.//").file_name());
    /// assert_eq!(None, TypedPath::<new("foo.txt/..").file_name());
    /// assert_eq!(None, TypedPath::new("/").file_name());
    /// ```
    pub fn file_name(&self) -> Option<&[u8]> {
        impl_typed_fn!(self, file_name)
    }

    /// Returns a path that, when joined onto `base`, yields `self`.
    ///
    /// # Errors
    ///
    /// If `base` is not a prefix of `self` (i.e., [`starts_with`]
    /// returns `false`), returns [`Err`].
    ///
    /// [`starts_with`]: TypedPath::starts_with
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{TypedPath, TypedPathBuf};
    ///
    /// let path = TypedPath::new("/test/haha/foo.txt");
    ///
    /// assert_eq!(path.strip_prefix("/"), Ok(TypedPath::new("test/haha/foo.txt")));
    /// assert_eq!(path.strip_prefix("/test"), Ok(TypedPath::new("haha/foo.txt")));
    /// assert_eq!(path.strip_prefix("/test/"), Ok(TypedPath::new("haha/foo.txt")));
    /// assert_eq!(path.strip_prefix("/test/haha/foo.txt"), Ok(TypedPath::new("")));
    /// assert_eq!(path.strip_prefix("/test/haha/foo.txt/"), Ok(TypedPath::new("")));
    ///
    /// assert!(path.strip_prefix("test").is_err());
    /// assert!(path.strip_prefix("/haha").is_err());
    ///
    /// let prefix = TypedPathBuf::from("/test/");
    /// assert_eq!(path.strip_prefix(prefix), Ok(TypedPath::new("haha/foo.txt")));
    /// ```
    pub fn strip_prefix<'b, P>(&self, base: P) -> Result<TypedPath, StripPrefixError>
    where
        P: AsRef<TypedPath<'b>>,
    {
        match (self, base.as_ref()) {
            (Self::Unix(path), Self::Unix(base)) => path.strip_prefix(base).map(Self::Unix),
            (Self::Unix(path), Self::Windows(base)) => {
                path.strip_prefix(base.with_unix_encoding()).map(Self::Unix)
            }
            (Self::Windows(path), Self::Unix(base)) => path
                .strip_prefix(base.with_windows_encoding())
                .map(Self::Windows),
            (Self::Windows(path), Self::Windows(base)) => {
                path.strip_prefix(base).map(Self::Windows)
            }
        }
    }

    /// Determines whether `base` is a prefix of `self`.
    ///
    /// Only considers whole path components to match.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// let path = TypedPath::new("/etc/passwd");
    ///
    /// assert!(path.starts_with("/etc"));
    /// assert!(path.starts_with("/etc/"));
    /// assert!(path.starts_with("/etc/passwd"));
    /// assert!(path.starts_with("/etc/passwd/")); // extra slash is okay
    /// assert!(path.starts_with("/etc/passwd///")); // multiple extra slashes are okay
    ///
    /// assert!(!path.starts_with("/e"));
    /// assert!(!path.starts_with("/etc/passwd.txt"));
    ///
    /// assert!(!TypedPath::new("/etc/foo.rs").starts_with("/etc/foo"));
    /// ```
    pub fn starts_with<'b, P>(&self, base: P) -> bool
    where
        P: AsRef<TypedPath<'b>>,
    {
        match (self, base.as_ref()) {
            (Self::Unix(path), Self::Unix(base)) => path.starts_with(base),
            (Self::Unix(path), Self::Windows(base)) => path.starts_with(base.with_unix_encoding()),
            (Self::Windows(path), Self::Unix(base)) => {
                path.starts_with(base.with_windows_encoding())
            }
            (Self::Windows(path), Self::Windows(base)) => path.starts_with(base),
        }
    }

    /// Determines whether `child` is a suffix of `self`.
    ///
    /// Only considers whole path components to match.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// let path = TypedPath::new("/etc/resolv.conf");
    ///
    /// assert!(path.ends_with("resolv.conf"));
    /// assert!(path.ends_with("etc/resolv.conf"));
    /// assert!(path.ends_with("/etc/resolv.conf"));
    ///
    /// assert!(!path.ends_with("/resolv.conf"));
    /// assert!(!path.ends_with("conf")); // use .extension() instead
    /// ```
    pub fn ends_with<'b, P>(&self, child: P) -> bool
    where
        P: AsRef<TypedPath<'b>>,
    {
        match (self, child.as_ref()) {
            (Self::Unix(path), Self::Unix(child)) => path.ends_with(child),
            (Self::Unix(path), Self::Windows(child)) => path.ends_with(child.with_unix_encoding()),
            (Self::Windows(path), Self::Unix(child)) => {
                path.ends_with(child.with_windows_encoding())
            }
            (Self::Windows(path), Self::Windows(child)) => path.ends_with(child),
        }
    }

    /// Extracts the stem (non-extension) portion of [`self.file_name`].
    ///
    /// [`self.file_name`]: TypedPath::file_name
    ///
    /// The stem is:
    ///
    /// * [`None`], if there is no file name;
    /// * The entire file name if there is no embedded `.`;
    /// * The entire file name if the file name begins with `.` and has no other `.`s within;
    /// * Otherwise, the portion of the file name before the final `.`
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// assert_eq!(b"foo", TypedPath::new("foo.rs").file_stem().unwrap());
    /// assert_eq!(b"foo.tar", TypedPath::new("foo.tar.gz").file_stem().unwrap());
    /// ```
    ///
    pub fn file_stem(&self) -> Option<&[u8]> {
        impl_typed_fn!(self, file_stem)
    }

    /// Extracts the extension of [`self.file_name`], if possible.
    ///
    /// The extension is:
    ///
    /// * [`None`], if there is no file name;
    /// * [`None`], if there is no embedded `.`;
    /// * [`None`], if the file name begins with `.` and has no other `.`s within;
    /// * Otherwise, the portion of the file name after the final `.`
    ///
    /// [`self.file_name`]: TypedPath::file_name
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// // NOTE: A path cannot be created on its own without a defined encoding
    /// assert_eq!(b"rs", TypedPath::new("foo.rs").extension().unwrap());
    /// assert_eq!(b"gz", TypedPath::new("foo.tar.gz").extension().unwrap());
    /// ```
    pub fn extension(&self) -> Option<&[u8]> {
        impl_typed_fn!(self, extension)
    }

    /// Returns an owned [`TypedPathBuf`] by resolving `..` and `.` segments.
    ///
    /// When multiple, sequential path segment separation characters are found (e.g. `/` for Unix
    /// and either `\` or `/` on Windows), they are replaced by a single instance of the
    /// platform-specific path segment separator (`/` on Unix and `\` on Windows).
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{TypedPath, TypedPathBuf};
    ///
    /// assert_eq!(
    ///     TypedPath::new("foo/bar//baz/./asdf/quux/..").normalize(),
    ///     TypedPathBuf::from("foo/bar/baz/asdf"),
    /// );
    /// ```
    ///
    /// When starting with a root directory, any `..` segment whose parent is the root directory
    /// will be filtered out:
    ///
    /// ```
    /// use typed_path::{TypedPath, TypedPathBuf};
    ///
    /// // NOTE: A path cannot be created on its own without a defined encoding
    /// assert_eq!(
    ///     TypedPath::new("/../foo").normalize(),
    ///     TypedPathBuf::from("/foo"),
    /// );
    /// ```
    ///
    /// If any `..` is left unresolved as the path is relative and no parent is found, it is
    /// discarded:
    ///
    /// ```
    /// use typed_path::{TypedPath, TypedPathBuf};
    ///
    /// assert_eq!(
    ///     TypedPath::new("../foo/..").normalize(),
    ///     TypedPathBuf::from(""),
    /// );
    ///
    /// // Windows prefixes also count this way, but the prefix remains
    /// assert_eq!(
    ///     TypedPath::new(r"C:..\foo\..").normalize(),
    ///     TypedPathBuf::from(r"C:"),
    /// );
    /// ```
    pub fn normalize(&self) -> TypedPathBuf {
        match self {
            Self::Unix(path) => TypedPathBuf::Unix(path.normalize()),
            Self::Windows(path) => TypedPathBuf::Windows(path.normalize()),
        }
    }

    /// Converts a path to an absolute form by [`normalizing`] the path, returning a
    /// [`TypedPathBuf`].
    ///
    /// In the case that the path is relative, the current working directory is prepended prior to
    /// normalizing.
    ///
    /// [`normalizing`]: TypedPath::normalize
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{utils, TypedPath};
    ///
    /// // With an absolute path, it is just normalized
    /// let path = TypedPath::new("/a/b/../c/./d");
    /// assert_eq!(path.absolutize().unwrap(), TypedPath::new("/a/c/d"));
    ///
    /// // With a relative path, it is first joined with the current working directory
    /// // and then normalized
    /// let cwd = utils::current_dir().unwrap().with_encoding::<UnixEncoding>();
    /// let path = cwd.join(TypedPath::new("a/b/../c/./d"));
    /// assert_eq!(path.absolutize().unwrap(), cwd.join(TypedPath::new("a/c/d")));
    /// ```
    pub fn absolutize(&self) -> io::Result<TypedPathBuf> {
        Ok(match self {
            Self::Unix(path) => TypedPathBuf::Unix(path.absolutize()?),
            Self::Windows(path) => TypedPathBuf::Windows(path.absolutize()?),
        })
    }

    /// Creates an owned [`TypedPathBuf`] with `path` adjoined to `self`.
    ///
    /// See [`TypedPathBuf::push`] for more details on what it means to adjoin a path.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{TypedPath, TypedPathBuf};
    ///
    /// assert_eq!(
    ///     TypedPath::new("/etc").join("passwd"),
    ///     TypedPathBuf::from("/etc/passwd"),
    /// );
    /// ```
    pub fn join<'b, P: AsRef<TypedPath<'b>>>(&self, path: P) -> TypedPathBuf {
        match (self, path.as_ref()) {
            (Self::Unix(base), Self::Unix(path)) => TypedPathBuf::Unix(base.join(path)),
            (Self::Unix(base), Self::Windows(path)) => {
                TypedPathBuf::Unix(base.join(path.with_unix_encoding()))
            }
            (Self::Windows(base), Self::Unix(path)) => {
                TypedPathBuf::Windows(base.join(path.with_windows_encoding()))
            }
            (Self::Windows(base), Self::Windows(path)) => TypedPathBuf::Windows(base.join(path)),
        }
    }

    /// Creates an owned [`TypedPathBuf`] like `self` but with the given file name.
    ///
    /// See [`TypedPathBuf::set_file_name`] for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{TypedPath, TypedPathBuf};
    ///
    /// let path = TypedPath::new("/tmp/foo.txt");
    /// assert_eq!(path.with_file_name("bar.txt"), TypedPathBuf::from("/tmp/bar.txt"));
    ///
    /// let path = TypedPath::new("/tmp");
    /// assert_eq!(path.with_file_name("var"), TypedPathBuf::from("/var"));
    /// ```
    pub fn with_file_name<S: AsRef<[u8]>>(&self, file_name: S) -> TypedPathBuf {
        match self {
            Self::Unix(path) => TypedPathBuf::Unix(path.with_file_name(file_name)),
            Self::Windows(path) => TypedPathBuf::Windows(path.with_file_name(file_name)),
        }
    }

    /// Creates an owned [`TypedPathBuf`] like `self` but with the given extension.
    ///
    /// See [`TypedPathBuf::set_extension`] for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{TypedPath, TypedPathBuf};
    ///
    /// let path = TypedPath::new("foo.rs");
    /// assert_eq!(path.with_extension("txt"), TypedPathBuf::from("foo.txt"));
    ///
    /// let path = TypedPath::new("foo.tar.gz");
    /// assert_eq!(path.with_extension(""), TypedPathBuf::from("foo.tar"));
    /// assert_eq!(path.with_extension("xz"), TypedPathBuf::from("foo.tar.xz"));
    /// assert_eq!(path.with_extension("").with_extension("txt"), TypedPathBuf::from("foo.txt"));
    /// ```
    pub fn with_extension<S: AsRef<[u8]>>(&self, extension: S) -> TypedPathBuf {
        match self {
            Self::Unix(path) => TypedPathBuf::Unix(path.with_extension(extension)),
            Self::Windows(path) => TypedPathBuf::Windows(path.with_extension(extension)),
        }
    }

    /// Produces an iterator over the [`TypedComponent`]s of the path.
    ///
    /// When parsing the path, there is a small amount of normalization:
    ///
    /// * Repeated separators are ignored, so `a/b` and `a//b` both have
    ///   `a` and `b` as components.
    ///
    /// * Occurrences of `.` are normalized away, except if they are at the
    ///   beginning of the path. For example, `a/./b`, `a/b/`, `a/b/.` and
    ///   `a/b` all have `a` and `b` as components, but `./a/b` starts with
    ///   an additional [`CurDir`] component.
    ///
    /// * A trailing slash is normalized away, `/a/b` and `/a/b/` are equivalent.
    ///
    /// Note that no other normalization takes place; in particular, `a/c`
    /// and `a/b/../c` are distinct, to account for the possibility that `b`
    /// is a symbolic link (so its parent isn't `a`).
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{TypedPath, TypedComponent};
    ///
    /// let mut components = TypedPath::new("/tmp/foo.txt").components();
    ///
    /// assert_eq!(components.next(), Some(TypedComponent::RootDir));
    /// assert_eq!(components.next(), Some(TypedComponent::Normal(b"tmp")));
    /// assert_eq!(components.next(), Some(TypedComponent::Normal(b"foo.txt")));
    /// assert_eq!(components.next(), None)
    /// ```
    ///
    /// [`CurDir`]: crate::TypedComponent::CurDir
    pub fn components(&self) -> impl Iterator<Item = TypedComponent> {
        todo!()
    }

    /// Produces an iterator over the path's components viewed as [`[u8]`] slices.
    ///
    /// For more information about the particulars of how the path is separated
    /// into components, see [`components`].
    ///
    /// [`components`]: TypedPath::components
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// let mut it = TypedPath::new("/tmp/foo.txt").iter();
    ///
    /// assert_eq!(it.next(), Some(typed_path::constants::unix::SEPARATOR_STR.as_bytes()));
    /// assert_eq!(it.next(), Some(b"tmp".as_slice()));
    /// assert_eq!(it.next(), Some(b"foo.txt".as_slice()));
    /// assert_eq!(it.next(), None)
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &[u8]> {
        self.components()
    }

    /// Returns an object that implements [`Display`] for safely printing paths
    /// that may contain non-Unicode data. This may perform lossy conversion,
    /// depending on the platform.  If you would like an implementation which
    /// escapes the path please use [`Debug`] instead.
    ///
    /// [`Debug`]: fmt::Debug
    /// [`Display`]: fmt::Display
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// let path = TypedPath::new("/tmp/foo.rs");
    ///
    /// println!("{}", path.display());
    /// ```
    #[inline]
    pub fn display(&self) -> impl fmt::Display + '_ {
        struct Display<'a> {
            path: &'a TypedPath<'a>,
        }

        impl fmt::Display for Display<'_> {
            /// Performs lossy conversion to UTF-8 str
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.path {
                    TypedPath::Unix(path) => fmt::Display::fmt(path, f),
                    TypedPath::Windows(path) => fmt::Display::fmt(path, f),
                }
            }
        }

        Display { path: self }
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
}

impl<'a> From<&'a [u8]> for TypedPath<'a> {
    #[inline]
    fn from(s: &'a [u8]) -> Self {
        TypedPath::new(s)
    }
}

impl<'a> From<&'a str> for TypedPath<'a> {
    #[inline]
    fn from(s: &'a str) -> Self {
        TypedPath::new(s.as_bytes())
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

impl<'a> TryAsRef<Path> for TypedPath<'a> {
    fn try_as_ref(&self) -> Option<&Path> {
        match self {
            Self::Unix(path) => path.try_as_ref(),
            Self::Windows(path) => path.try_as_ref(),
        }
    }
}
