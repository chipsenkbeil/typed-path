#[macro_use]
mod common;
pub mod unix;
pub mod windows;

mod private {
    /// Used to mark traits as sealed to prevent implements from others outside of this crate
    pub trait Sealed {}
}

pub use common::*;
pub use unix::{UnixEncoding, UnixPath, UnixPathBuf};
pub use windows::{WindowsEncoding, WindowsPath, WindowsPathBuf};

/// [`Path`] that is native to the platform during compilation
#[cfg(unix)]
pub type NativePath = UnixPath;

/// [`PathBuf`] that is native to the platform during compilation
#[cfg(unix)]
pub type NativePathBuf = UnixPathBuf;

/// [`Path`] that is native to the platform during compilation
#[cfg(windows)]
pub type NativePath = WindowsPath;

/// [`PathBuf`] that is native to the platform during compilation
#[cfg(windows)]
pub type NativePathBuf = WindowsPathBuf;
