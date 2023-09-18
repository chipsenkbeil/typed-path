use std::convert::TryFrom;
use std::ffi::OsStr;
use std::path::{Component as StdComponent, Path as StdPath, PathBuf as StdPathBuf};

use crate::native::{Utf8NativePath, Utf8NativePathBuf};
use crate::unix::UnixComponent;
use crate::windows::{WindowsComponent, WindowsPrefixComponent};
use crate::{Encoding, Path, PathBuf};

/// Interface to try to perform a cheap reference-to-reference conversion.
pub trait TryAsRef<T: ?Sized> {
    fn try_as_ref(&self) -> Option<&T>;
}

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

impl<'a> TryFrom<UnixComponent<'a>> for StdComponent<'a> {
    type Error = UnixComponent<'a>;

    /// Attempts to convert a [`UnixComponent`] into a [`std::path::Component`], returning a result
    /// containing the new path when successful or the original path when failed
    ///
    /// # Examples
    ///
    /// ```
    /// use std::convert::TryFrom;
    /// use std::ffi::OsStr;
    /// use std::path::Component;
    /// use typed_path::unix::UnixComponent;
    ///
    /// let component = Component::try_from(UnixComponent::RootDir).unwrap();
    /// assert_eq!(component, Component::RootDir);
    ///
    /// let component = Component::try_from(UnixComponent::CurDir).unwrap();
    /// assert_eq!(component, Component::CurDir);
    ///
    /// let component = Component::try_from(UnixComponent::ParentDir).unwrap();
    /// assert_eq!(component, Component::ParentDir);
    ///
    /// let component = Component::try_from(UnixComponent::Normal(b"file.txt")).unwrap();
    /// assert_eq!(component, Component::Normal(OsStr::new("file.txt")));
    /// ```
    fn try_from(component: UnixComponent<'a>) -> Result<Self, Self::Error> {
        match &component {
            UnixComponent::RootDir => Ok(Self::RootDir),
            UnixComponent::CurDir => Ok(Self::CurDir),
            UnixComponent::ParentDir => Ok(Self::ParentDir),
            UnixComponent::Normal(x) => Ok(Self::Normal(OsStr::new(
                std::str::from_utf8(x).map_err(|_| component)?,
            ))),
        }
    }
}

impl<'a> TryFrom<StdComponent<'a>> for UnixComponent<'a> {
    type Error = StdComponent<'a>;

    /// Attempts to convert a [`std::path::Component`] into a [`UnixComponent`], returning a result
    /// containing the new component when successful or the original component when failed
    ///
    /// # Examples
    ///
    /// ```
    /// use std::convert::TryFrom;
    /// use std::ffi::OsStr;
    /// use std::path::Component;
    /// use typed_path::unix::UnixComponent;
    ///
    /// let component = UnixComponent::try_from(Component::RootDir).unwrap();
    /// assert_eq!(component, UnixComponent::RootDir);
    ///
    /// let component = UnixComponent::try_from(Component::CurDir).unwrap();
    /// assert_eq!(component, UnixComponent::CurDir);
    ///
    /// let component = UnixComponent::try_from(Component::ParentDir).unwrap();
    /// assert_eq!(component, UnixComponent::ParentDir);
    ///
    /// let component = UnixComponent::try_from(Component::Normal(OsStr::new("file.txt"))).unwrap();
    /// assert_eq!(component, UnixComponent::Normal(b"file.txt"));
    /// ```
    fn try_from(component: StdComponent<'a>) -> Result<Self, Self::Error> {
        match &component {
            StdComponent::Prefix(_) => Err(component),
            StdComponent::RootDir => Ok(Self::RootDir),
            StdComponent::CurDir => Ok(Self::CurDir),
            StdComponent::ParentDir => Ok(Self::ParentDir),
            StdComponent::Normal(x) => Ok(Self::Normal(x.to_str().ok_or(component)?.as_bytes())),
        }
    }
}

impl<'a> TryFrom<WindowsComponent<'a>> for StdComponent<'a> {
    type Error = WindowsComponent<'a>;

    /// Attempts to convert a [`WindowsComponent`] into a [`std::path::Component`], returning a
    /// result containing the new path when successful or the original path when failed
    ///
    /// # Examples
    ///
    /// ```
    /// use std::convert::TryFrom;
    /// use std::ffi::OsStr;
    /// use std::path::Component;
    /// use typed_path::windows::WindowsComponent;
    ///
    /// let component = Component::try_from(WindowsComponent::RootDir).unwrap();
    /// assert_eq!(component, Component::RootDir);
    ///
    /// let component = Component::try_from(WindowsComponent::CurDir).unwrap();
    /// assert_eq!(component, Component::CurDir);
    ///
    /// let component = Component::try_from(WindowsComponent::ParentDir).unwrap();
    /// assert_eq!(component, Component::ParentDir);
    ///
    /// let component = Component::try_from(WindowsComponent::Normal(b"file.txt")).unwrap();
    /// assert_eq!(component, Component::Normal(OsStr::new("file.txt")));
    /// ```
    ///
    /// Alongside the traditional path components, the [`Component::Prefix`] variant is also
    /// supported, but only when compiling on Windows. When on a non-Windows platform, the
    /// conversion will always fail.
    ///
    /// [`Component::Prefix`]: std::path::Component::Prefix
    ///
    fn try_from(component: WindowsComponent<'a>) -> Result<Self, Self::Error> {
        match &component {
            // NOTE: Standard library provides no way to construct a PrefixComponent, so, we have
            //       to build a new path with just the prefix and then get the component
            //
            //       Because the prefix is not empty when being supplied to the path, we should get
            //       back at least one component and can therefore return the unwrapped result
            WindowsComponent::Prefix(x) => {
                if cfg!(windows) {
                    Ok(
                        StdPath::new(std::str::from_utf8(x.as_bytes()).map_err(|_| component)?)
                            .components()
                            .next()
                            .expect("Impossible: non-empty std path had no components"),
                    )
                } else {
                    Err(component)
                }
            }
            WindowsComponent::RootDir => Ok(Self::RootDir),
            WindowsComponent::CurDir => Ok(Self::CurDir),
            WindowsComponent::ParentDir => Ok(Self::ParentDir),
            WindowsComponent::Normal(x) => Ok(Self::Normal(OsStr::new(
                std::str::from_utf8(x).map_err(|_| component)?,
            ))),
        }
    }
}

impl<'a> TryFrom<StdComponent<'a>> for WindowsComponent<'a> {
    type Error = StdComponent<'a>;

    /// Attempts to convert a [`std::path::Component`] into a [`WindowsComponent`], returning a
    /// result containing the new component when successful or the original component when failed
    ///
    /// # Examples
    ///
    /// ```
    /// use std::convert::TryFrom;
    /// use std::ffi::OsStr;
    /// use std::path::Component;
    /// use typed_path::windows::WindowsComponent;
    ///
    /// let component = WindowsComponent::try_from(Component::RootDir).unwrap();
    /// assert_eq!(component, WindowsComponent::RootDir);
    ///
    /// let component = WindowsComponent::try_from(Component::CurDir).unwrap();
    /// assert_eq!(component, WindowsComponent::CurDir);
    ///
    /// let component = WindowsComponent::try_from(Component::ParentDir).unwrap();
    /// assert_eq!(component, WindowsComponent::ParentDir);
    ///
    /// let component = WindowsComponent::try_from(Component::Normal(OsStr::new("file.txt"))).unwrap();
    /// assert_eq!(component, WindowsComponent::Normal(b"file.txt"));
    /// ```
    ///
    /// Alongside the traditional path components, the [`Component::Prefix`] variant is also
    /// supported, but only when compiling on Windows. When on a non-Windows platform, the
    /// conversion will always fail.
    ///
    /// [`Component::Prefix`]: std::path::Component::Prefix
    ///
    fn try_from(component: StdComponent<'a>) -> Result<Self, Self::Error> {
        match &component {
            StdComponent::Prefix(x) => Ok(WindowsComponent::Prefix(
                WindowsPrefixComponent::try_from(x.as_os_str().to_str().ok_or(component)?)
                    .map_err(|_| component)?,
            )),
            StdComponent::RootDir => Ok(Self::RootDir),
            StdComponent::CurDir => Ok(Self::CurDir),
            StdComponent::ParentDir => Ok(Self::ParentDir),
            StdComponent::Normal(x) => Ok(Self::Normal(x.to_str().ok_or(component)?.as_bytes())),
        }
    }
}

impl AsRef<StdPath> for Utf8NativePath {
    /// Converts a native utf8 path (based on compilation family) into [`std::path::Path`].
    ///
    /// ```
    /// use typed_path::Utf8NativePath;
    /// use std::path::Path;
    ///
    /// let native_path = Utf8NativePath::new("some_file.txt");
    /// let std_path: &Path = native_path.as_ref();
    ///
    /// assert_eq!(std_path, Path::new("some_file.txt"));
    /// ```
    fn as_ref(&self) -> &StdPath {
        StdPath::new(self.as_str())
    }
}

impl AsRef<StdPath> for Utf8NativePathBuf {
    /// Converts a native utf8 pathbuf (based on compilation family) into [`std::path::Path`].
    ///
    /// ```
    /// use typed_path::Utf8NativePathBuf;
    /// use std::path::Path;
    ///
    /// let native_path_buf = Utf8NativePathBuf::from("some_file.txt");
    /// let std_path: &Path = native_path_buf.as_ref();
    ///
    /// assert_eq!(std_path, Path::new("some_file.txt"));
    /// ```
    fn as_ref(&self) -> &StdPath {
        StdPath::new(self.as_str())
    }
}

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

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::path::Component;

    use super::*;

    fn make_windows_prefix_component(s: &str) -> WindowsComponent {
        let component = WindowsComponent::try_from(s).unwrap();
        assert!(component.is_prefix());
        component
    }

    #[test]
    #[cfg(windows)]
    fn try_from_windows_component_to_std_component_should_keep_prefix_on_windows() {
        use std::ffi::OsStr;
        use std::path::Prefix;
        fn get_prefix(component: Component) -> Prefix {
            match component {
                Component::Prefix(prefix) => prefix.kind(),
                x => panic!("Wrong component: {x:?}"),
            }
        }

        let component = Component::try_from(make_windows_prefix_component("C:")).unwrap();
        assert_eq!(get_prefix(component), Prefix::Disk(b'C'));

        let component =
            Component::try_from(make_windows_prefix_component(r"\\server\share")).unwrap();
        assert_eq!(
            get_prefix(component),
            Prefix::UNC(OsStr::new("server"), OsStr::new("share"))
        );

        let component = Component::try_from(make_windows_prefix_component(r"\\.\COM42")).unwrap();
        assert_eq!(get_prefix(component), Prefix::DeviceNS(OsStr::new("COM42")));

        let component = Component::try_from(make_windows_prefix_component(r"\\?\C:")).unwrap();
        assert_eq!(get_prefix(component), Prefix::VerbatimDisk(b'C'));

        let component =
            Component::try_from(make_windows_prefix_component(r"\\?\UNC\server\share")).unwrap();
        assert_eq!(
            get_prefix(component),
            Prefix::VerbatimUNC(OsStr::new("server"), OsStr::new("share"))
        );

        let component =
            Component::try_from(make_windows_prefix_component(r"\\?\pictures")).unwrap();
        assert_eq!(
            get_prefix(component),
            Prefix::Verbatim(OsStr::new("pictures"))
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn try_from_windows_component_to_std_component_should_fail_for_prefix_on_non_windows() {
        Component::try_from(make_windows_prefix_component("C:")).unwrap_err();
        Component::try_from(make_windows_prefix_component(r"\\server\share")).unwrap_err();
        Component::try_from(make_windows_prefix_component(r"\\.\COM42")).unwrap_err();
        Component::try_from(make_windows_prefix_component(r"\\?\C:")).unwrap_err();
        Component::try_from(make_windows_prefix_component(r"\\?\UNC\server\share")).unwrap_err();
        Component::try_from(make_windows_prefix_component(r"\\?\pictures")).unwrap_err();
    }

    #[test]
    #[cfg(windows)]
    fn try_from_std_component_to_windows_component_should_keep_prefix_on_windows() {
        use std::path::Path;

        use crate::windows::WindowsPrefix;

        fn make_component(s: &str) -> Component {
            let component = Path::new(s).components().next();
            assert!(
                matches!(component, Some(Component::Prefix(_))),
                "std component not a prefix"
            );
            component.unwrap()
        }

        fn get_prefix(component: WindowsComponent) -> WindowsPrefix {
            match component {
                WindowsComponent::Prefix(prefix) => prefix.kind(),
                x => panic!("Wrong component: {x:?}"),
            }
        }

        let component = WindowsComponent::try_from(make_component("C:")).unwrap();
        assert_eq!(get_prefix(component), WindowsPrefix::Disk(b'C'));

        let component = WindowsComponent::try_from(make_component(r"\\server\share")).unwrap();
        assert_eq!(
            get_prefix(component),
            WindowsPrefix::UNC(b"server", b"share")
        );

        let component = WindowsComponent::try_from(make_component(r"\\.\COM42")).unwrap();
        assert_eq!(get_prefix(component), WindowsPrefix::DeviceNS(b"COM42"));

        let component = WindowsComponent::try_from(make_component(r"\\?\C:")).unwrap();
        assert_eq!(get_prefix(component), WindowsPrefix::VerbatimDisk(b'C'));

        let component =
            WindowsComponent::try_from(make_component(r"\\?\UNC\server\share")).unwrap();
        assert_eq!(
            get_prefix(component),
            WindowsPrefix::VerbatimUNC(b"server", b"share")
        );

        let component = WindowsComponent::try_from(make_component(r"\\?\pictures")).unwrap();
        assert_eq!(get_prefix(component), WindowsPrefix::Verbatim(b"pictures"));
    }
}
