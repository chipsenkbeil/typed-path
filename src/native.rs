/// [`Path`] that is native to the platform during compilation
#[cfg(unix)]
pub type NativePath = crate::unix::UnixPath;

/// [`PathBuf`] that is native to the platform during compilation
#[cfg(unix)]
pub type NativePathBuf = crate::unix::UnixPathBuf;

/// [`Component`] that is native to the platform during compilation
#[cfg(unix)]
pub type NativeComponent<'a> = crate::unix::UnixComponent<'a>;

/// [`Path`] that is native to the platform during compilation
#[cfg(windows)]
pub type NativePath = crate::windows::WindowsPath;

/// [`PathBuf`] that is native to the platform during compilation
#[cfg(windows)]
pub type NativePathBuf = crate::windows::WindowsPathBuf;

/// [`Component`] that is native to the platform during compilation
#[cfg(windows)]
pub type NativeComponent<'a> = crate::windows::WindowsComponent<'a>;
