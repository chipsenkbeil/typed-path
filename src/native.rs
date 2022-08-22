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
