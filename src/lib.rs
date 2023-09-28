#![doc = include_str!("../README.md")]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

#[macro_use]
mod common;
mod convert;
pub mod native;
mod typed;
pub mod unix;
pub mod utils;
pub mod windows;

mod private {
    /// Used to mark traits as sealed to prevent implements from others outside of this crate
    pub trait Sealed {}
}

pub use common::{
    Ancestors, Component, Components, Display, Encoding, Iter, ParseError, Path, PathBuf,
    StripPrefixError, Utf8Ancestors, Utf8Component, Utf8Components, Utf8Encoding, Utf8Iter,
    Utf8Path, Utf8PathBuf,
};
pub use convert::TryAsRef;
pub use native::{NativePath, NativePathBuf, Utf8NativePath, Utf8NativePathBuf};
pub use typed::{TypedPath, TypedPathBuf, Utf8TypedPath, Utf8TypedPathBuf};
pub use unix::{
    UnixEncoding, UnixPath, UnixPathBuf, Utf8UnixEncoding, Utf8UnixPath, Utf8UnixPathBuf,
};
pub use windows::{
    Utf8WindowsEncoding, Utf8WindowsPath, Utf8WindowsPathBuf, WindowsEncoding, WindowsPath,
    WindowsPathBuf,
};
