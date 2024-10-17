pub use self::non_utf8::*;
pub use self::utf8::*;

mod non_utf8 {
    use crate::common::{CheckedPathError, Encoding, Path, PathBuf};
    use crate::native::{NativeComponent, NativeEncoding};
    use crate::private;
    use core::fmt;
    use core::hash::Hasher;

    /// [`Path`] that has the platform's encoding during compilation.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::PlatformPath;
    ///
    /// // You can create the path like normal, but it is a distinct encoding from Unix/Windows
    /// let path = PlatformPath::new("some/path");
    ///
    /// // The path will still behave like normal and even report its underlying encoding
    /// assert_eq!(path.has_unix_encoding(), cfg!(unix));
    /// assert_eq!(path.has_windows_encoding(), cfg!(windows));
    ///
    /// // It can still be converted into specific platform paths
    /// let unix_path = path.with_unix_encoding();
    /// let win_path = path.with_windows_encoding();
    /// ```
    pub type PlatformPath = Path<PlatformEncoding>;

    /// [`PathBuf`] that has the platform's encoding during compilation.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::PlatformPathBuf;
    ///
    /// // You can create the pathbuf like normal, but it is a distinct encoding from Unix/Windows
    /// let path = PlatformPathBuf::from("some/path");
    ///
    /// // The path will still behave like normal and even report its underlying encoding
    /// assert_eq!(path.has_unix_encoding(), cfg!(unix));
    /// assert_eq!(path.has_windows_encoding(), cfg!(windows));
    ///
    /// // It can still be converted into specific platform paths
    /// let unix_path = path.with_unix_encoding();
    /// let win_path = path.with_windows_encoding();
    /// ```
    pub type PlatformPathBuf = PathBuf<PlatformEncoding>;

    /// [`Component`] that is has the platform's encoding during compilation.
    pub type PlatformComponent<'a> = NativeComponent<'a>;

    /// Represents an abstraction of [`Encoding`] that represents the current platform encoding.
    ///
    /// This differs from [`NativeEncoding`] in that it is its own struct instead of a type alias
    /// to the platform-specific encoding, and can therefore be used to enforce more strict
    /// compile-time checks of encodings without needing to leverage conditional configs.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::any::TypeId;
    /// use typed_path::{PlatformEncoding, UnixEncoding, WindowsEncoding};
    ///
    /// // The platform encoding is considered a distinct type from Unix/Windows encodings.
    /// assert_ne!(TypeId::of::<PlatformEncoding>(), TypeId::of::<UnixEncoding>());
    /// assert_ne!(TypeId::of::<PlatformEncoding>(), TypeId::of::<WindowsEncoding>());
    /// ```
    #[derive(Copy, Clone)]
    pub struct PlatformEncoding;

    impl private::Sealed for PlatformEncoding {}

    impl<'a> Encoding<'a> for PlatformEncoding {
        type Components = <NativeEncoding as Encoding<'a>>::Components;

        fn label() -> &'static str {
            NativeEncoding::label()
        }

        fn components(path: &'a [u8]) -> Self::Components {
            <NativeEncoding as Encoding<'a>>::components(path)
        }

        fn hash<H: Hasher>(path: &[u8], h: &mut H) {
            <NativeEncoding as Encoding<'a>>::hash(path, h)
        }

        fn push(current_path: &mut Vec<u8>, path: &[u8]) {
            <NativeEncoding as Encoding<'a>>::push(current_path, path);
        }

        fn push_checked(current_path: &mut Vec<u8>, path: &[u8]) -> Result<(), CheckedPathError> {
            <NativeEncoding as Encoding<'a>>::push_checked(current_path, path)
        }
    }

    impl fmt::Debug for PlatformEncoding {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("PlatformEncoding").finish()
        }
    }

    impl fmt::Display for PlatformEncoding {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "PlatformEncoding")
        }
    }
}

mod utf8 {
    use crate::common::{CheckedPathError, Utf8Encoding, Utf8Path, Utf8PathBuf};
    use crate::native::{Utf8NativeComponent, Utf8NativeEncoding};
    use crate::private;
    use core::fmt;
    use core::hash::Hasher;

    /// [`Utf8Path`] that has the platform's encoding during compilation.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::Utf8PlatformPath;
    ///
    /// // You can create the path like normal, but it is a distinct encoding from Unix/Windows
    /// let path = Utf8PlatformPath::new("some/path");
    ///
    /// // The path will still behave like normal and even report its underlying encoding
    /// assert_eq!(path.has_unix_encoding(), cfg!(unix));
    /// assert_eq!(path.has_windows_encoding(), cfg!(windows));
    ///
    /// // It can still be converted into specific platform paths
    /// let unix_path = path.with_unix_encoding();
    /// let win_path = path.with_windows_encoding();
    /// ```
    pub type Utf8PlatformPath = Utf8Path<Utf8PlatformEncoding>;

    /// [`Utf8PathBuf`] that has the platform's encoding during compilation.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::Utf8PlatformPathBuf;
    ///
    /// // You can create the pathbuf like normal, but it is a distinct encoding from Unix/Windows
    /// let path = Utf8PlatformPathBuf::from("some/path");
    ///
    /// // The path will still behave like normal and even report its underlying encoding
    /// assert_eq!(path.has_unix_encoding(), cfg!(unix));
    /// assert_eq!(path.has_windows_encoding(), cfg!(windows));
    ///
    /// // It can still be converted into specific platform paths
    /// let unix_path = path.with_unix_encoding();
    /// let win_path = path.with_windows_encoding();
    /// ```
    pub type Utf8PlatformPathBuf = Utf8PathBuf<Utf8PlatformEncoding>;

    /// [`Utf8Component`] that is has the platform's encoding during compilation.
    pub type Utf8PlatformComponent<'a> = Utf8NativeComponent<'a>;

    /// Represents an abstraction of [`Utf8Encoding`] that represents the current platform
    /// encoding.
    ///
    /// This differs from [`Utf8NativeEncoding`] in that it is its own struct instead of a type
    /// alias to the platform-specific encoding, and can therefore be used to enforce more strict
    /// compile-time checks of encodings without needing to leverage conditional configs.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::any::TypeId;
    /// use typed_path::{Utf8PlatformEncoding, Utf8UnixEncoding, Utf8WindowsEncoding};
    ///
    /// // The UTF8 platform encoding is considered a distinct type from UTF8 Unix/Windows encodings.
    /// assert_ne!(TypeId::of::<Utf8PlatformEncoding>(), TypeId::of::<Utf8UnixEncoding>());
    /// assert_ne!(TypeId::of::<Utf8PlatformEncoding>(), TypeId::of::<Utf8WindowsEncoding>());
    /// ```
    #[derive(Copy, Clone)]
    pub struct Utf8PlatformEncoding;

    impl private::Sealed for Utf8PlatformEncoding {}

    impl<'a> Utf8Encoding<'a> for Utf8PlatformEncoding {
        type Components = <Utf8NativeEncoding as Utf8Encoding<'a>>::Components;

        fn label() -> &'static str {
            Utf8NativeEncoding::label()
        }

        fn components(path: &'a str) -> Self::Components {
            <Utf8NativeEncoding as Utf8Encoding<'a>>::components(path)
        }

        fn hash<H: Hasher>(path: &str, h: &mut H) {
            <Utf8NativeEncoding as Utf8Encoding<'a>>::hash(path, h)
        }

        fn push(current_path: &mut String, path: &str) {
            <Utf8NativeEncoding as Utf8Encoding<'a>>::push(current_path, path);
        }

        fn push_checked(current_path: &mut String, path: &str) -> Result<(), CheckedPathError> {
            <Utf8NativeEncoding as Utf8Encoding<'a>>::push_checked(current_path, path)
        }
    }

    impl fmt::Debug for Utf8PlatformEncoding {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Utf8PlatformEncoding").finish()
        }
    }

    impl fmt::Display for Utf8PlatformEncoding {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Utf8PlatformEncoding")
        }
    }
}
