pub use self::non_utf8::*;
pub use self::utf8::*;

mod non_utf8 {
    /// [`Path`](crate::Path) that is native to the platform during compilation
    #[cfg(unix)]
    pub type NativePath = crate::unix::UnixPath;

    /// [`PathBuf`](crate::PathBuf) that is native to the platform during compilation
    #[cfg(unix)]
    pub type NativePathBuf = crate::unix::UnixPathBuf;

    /// [`Component`](crate::Component) that is native to the platform during compilation
    #[cfg(unix)]
    pub type NativeComponent<'a> = crate::unix::UnixComponent<'a>;

    /// [`Path`](crate::Path) that is native to the platform during compilation
    #[cfg(windows)]
    pub type NativePath = crate::windows::WindowsPath;

    /// [`PathBuf`](crate::PathBuf) that is native to the platform during compilation
    #[cfg(windows)]
    pub type NativePathBuf = crate::windows::WindowsPathBuf;

    /// [`Component`](crate::Component) that is native to the platform during compilation
    #[cfg(windows)]
    pub type NativeComponent<'a> = crate::windows::WindowsComponent<'a>;
}

mod utf8 {
    /// [`Utf8Path`](crate::Utf8Path) that is native to the platform during compilation
    #[cfg(unix)]
    pub type Utf8NativePath = crate::unix::Utf8UnixPath;

    /// [`Utf8PathBuf`](crate::Utf8PathBuf) that is native to the platform during compilation
    #[cfg(unix)]
    pub type Utf8NativePathBuf = crate::unix::Utf8UnixPathBuf;

    /// [`Utf8Component`](crate::Utf8Component) that is native to the platform during compilation
    #[cfg(unix)]
    pub type Utf8NativeComponent<'a> = crate::unix::Utf8UnixComponent<'a>;

    /// [`Utf8Path`](crate::Utf8Path) that is native to the platform during compilation
    #[cfg(windows)]
    pub type Utf8NativePath = crate::windows::Utf8WindowsPath;

    /// [`Utf8PathBuf`](crate::Utf8PathBuf) that is native to the platform during compilation
    #[cfg(windows)]
    pub type Utf8NativePathBuf = crate::windows::Utf8WindowsPathBuf;

    /// [`Utf8Component`](crate::Utf8Component) that is native to the platform during compilation
    #[cfg(windows)]
    pub type Utf8NativeComponent<'a> = crate::windows::Utf8WindowsComponent<'a>;
}
