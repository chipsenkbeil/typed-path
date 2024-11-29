#[cfg(feature = "std")]
use std::{
    convert::TryFrom,
    path::{Path as StdPath, PathBuf as StdPathBuf},
};

#[cfg(feature = "std")]
use crate::{Encoding, Path, PathBuf};

#[cfg(all(feature = "std", not(target_family = "wasm")))]
use crate::native::{Utf8NativePath, Utf8NativePathBuf};

/// Interface to try to perform a cheap reference-to-reference conversion.
pub trait TryAsRef<T: ?Sized> {
    fn try_as_ref(&self) -> Option<&T>;
}

#[cfg(feature = "std")]
impl<T> TryAsRef<StdPath> for Path<T>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Attempts to convert a [`Path`] into a [`std::path::Path`], succeeding if the path is
    /// comprised only of valid UTF-8 bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use typed_path::{TryAsRef, UnixPath};
    ///
    /// let unix_path = UnixPath::new("/path/to/file.txt");
    /// let std_path: &Path = unix_path.try_as_ref().unwrap();
    /// ```
    fn try_as_ref(&self) -> Option<&StdPath> {
        std::str::from_utf8(self.as_bytes()).map(StdPath::new).ok()
    }
}

#[cfg(feature = "std")]
impl<T> TryAsRef<Path<T>> for StdPath
where
    T: for<'enc> Encoding<'enc>,
{
    /// Attempts to convert a [`std::path::Path`] into a [`Path`], returning a result containing
    /// the new path when successful
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use typed_path::{TryAsRef, UnixPath};
    ///
    /// let std_path = Path::new("/path/to/file.txt");
    /// let unix_path: &UnixPath = std_path.try_as_ref().unwrap();
    /// ```
    fn try_as_ref(&self) -> Option<&Path<T>> {
        self.to_str().map(Path::new)
    }
}

#[cfg(feature = "std")]
impl<T> TryFrom<PathBuf<T>> for StdPathBuf
where
    T: for<'enc> Encoding<'enc>,
{
    type Error = PathBuf<T>;

    /// Attempts to convert a [`PathBuf`] into a [`std::path::PathBuf`], returning a result
    /// containing the new path when successful or the original path when failed
    ///
    /// # Examples
    ///
    /// ```
    /// use std::convert::TryFrom;
    /// use std::path::PathBuf;
    /// use typed_path::UnixPathBuf;
    ///
    /// let unix_path_buf = UnixPathBuf::from("/path/to/file.txt");
    /// let std_path_buf: PathBuf = TryFrom::try_from(unix_path_buf).unwrap();
    /// ```
    fn try_from(path: PathBuf<T>) -> Result<Self, Self::Error> {
        match std::str::from_utf8(path.as_bytes()) {
            Ok(s) => Ok(StdPathBuf::from(s)),
            Err(_) => Err(path),
        }
    }
}

#[cfg(feature = "std")]
impl<T> TryFrom<StdPathBuf> for PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    type Error = StdPathBuf;

    /// Attempts to convert a [`std::path::PathBuf`] into a [`PathBuf`], returning a result
    /// containing the new path when successful or the original path when failed
    ///
    /// # Examples
    ///
    /// ```
    /// use std::convert::TryFrom;
    /// use std::path::PathBuf;
    /// use typed_path::UnixPathBuf;
    ///
    /// let std_path_buf = PathBuf::from("/path/to/file.txt");
    /// let unix_path_buf: UnixPathBuf = TryFrom::try_from(std_path_buf).unwrap();
    /// ```
    fn try_from(path: StdPathBuf) -> Result<Self, Self::Error> {
        match path.to_str() {
            Some(s) => Ok(PathBuf::from(s)),
            None => Err(path),
        }
    }
}

#[cfg(all(feature = "std", not(target_family = "wasm")))]
impl<'a> From<&'a Utf8NativePath> for StdPathBuf {
    /// Converts a native utf8 path (based on compilation family) into [`std::path::PathBuf`].
    ///
    /// ```
    /// use typed_path::Utf8NativePath;
    /// use std::path::PathBuf;
    ///
    /// let native_path = Utf8NativePath::new("some_file.txt");
    /// let std_path_buf = PathBuf::from(native_path);
    ///
    /// assert_eq!(std_path_buf, PathBuf::from("some_file.txt"));
    /// ```
    fn from(utf8_native_path: &'a Utf8NativePath) -> StdPathBuf {
        StdPathBuf::from(utf8_native_path.to_string())
    }
}

#[cfg(all(feature = "std", not(target_family = "wasm")))]
impl From<Utf8NativePathBuf> for StdPathBuf {
    /// Converts a native utf8 pathbuf (based on compilation family) into [`std::path::PathBuf`].
    ///
    /// ```
    /// use typed_path::Utf8NativePathBuf;
    /// use std::path::PathBuf;
    ///
    /// let native_path_buf = Utf8NativePathBuf::from("some_file.txt");
    /// let std_path_buf = PathBuf::from(native_path_buf);
    ///
    /// assert_eq!(std_path_buf, PathBuf::from("some_file.txt"));
    /// ```
    fn from(utf8_native_path_buf: Utf8NativePathBuf) -> StdPathBuf {
        StdPathBuf::from(utf8_native_path_buf.into_string())
    }
}

#[cfg(any(
    unix,
    all(target_vendor = "fortanix", target_env = "sgx"),
    target_os = "solid_asp3",
    target_os = "hermit",
    target_os = "wasi"
))]
#[cfg(feature = "std")]
mod common {
    use std::ffi::{OsStr, OsString};
    #[cfg(all(target_vendor = "fortanix", target_env = "sgx"))]
    use std::os::fortanix_sgx as os;
    #[cfg(target_os = "solid_asp3")]
    use std::os::solid as os;
    #[cfg(any(target_os = "hermit", unix))]
    use std::os::unix as os;
    #[cfg(target_os = "wasi")]
    use std::os::wasi as os;

    use os::ffi::{OsStrExt, OsStringExt};

    use super::*;

    impl<T> From<PathBuf<T>> for OsString
    where
        T: for<'enc> Encoding<'enc>,
    {
        #[inline]
        fn from(path_buf: PathBuf<T>) -> Self {
            OsStringExt::from_vec(path_buf.into_vec())
        }
    }

    impl<T> AsRef<Path<T>> for OsStr
    where
        T: for<'enc> Encoding<'enc>,
    {
        #[inline]
        fn as_ref(&self) -> &Path<T> {
            Path::new(self.as_bytes())
        }
    }

    impl<T> AsRef<Path<T>> for OsString
    where
        T: for<'enc> Encoding<'enc>,
    {
        #[inline]
        fn as_ref(&self) -> &Path<T> {
            Path::new(self.as_bytes())
        }
    }

    impl<T> AsRef<OsStr> for Path<T>
    where
        T: for<'enc> Encoding<'enc>,
    {
        #[inline]
        fn as_ref(&self) -> &OsStr {
            OsStrExt::from_bytes(self.as_bytes())
        }
    }

    impl<T> AsRef<OsStr> for PathBuf<T>
    where
        T: for<'enc> Encoding<'enc>,
    {
        #[inline]
        fn as_ref(&self) -> &OsStr {
            OsStrExt::from_bytes(self.as_bytes())
        }
    }
}
