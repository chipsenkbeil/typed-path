#![doc = include_str!("../README.md")]

#[macro_use]
mod common;
mod convert;
pub mod native;
pub mod unix;
pub mod windows;

mod private {
    /// Used to mark traits as sealed to prevent implements from others outside of this crate
    pub trait Sealed {}
}

pub use common::{
    Ancestors, Component, Components, Display, Encoding, Iter, ParseError, Path, PathBuf,
    StripPrefixError,
};
pub use convert::TryAsRef;
pub use native::{NativePath, NativePathBuf};
pub use unix::{UnixEncoding, UnixPath, UnixPathBuf};
pub use windows::{WindowsEncoding, WindowsPath, WindowsPathBuf};
