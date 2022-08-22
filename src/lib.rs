#[macro_use]
mod common;
pub mod native;
pub mod unix;
pub mod windows;

mod private {
    /// Used to mark traits as sealed to prevent implements from others outside of this crate
    pub trait Sealed {}
}

pub use common::*;
pub use native::{NativePath, NativePathBuf};
pub use unix::{UnixEncoding, UnixPath, UnixPathBuf};
pub use windows::{WindowsEncoding, WindowsPath, WindowsPathBuf};
