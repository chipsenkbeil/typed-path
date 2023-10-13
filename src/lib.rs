#![doc = include_str!("../README.md")]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

#[macro_use]
mod common;
mod convert;
mod native;
mod typed;
mod unix;
pub mod utils;
mod windows;

mod private {
    /// Used to mark traits as sealed to prevent implements from others outside of this crate
    pub trait Sealed {}
}

pub use common::*;
pub use convert::*;
pub use native::*;
pub use typed::*;
pub use unix::*;
pub use windows::*;

/// Contains constants associated with different path formats.
pub mod constants {
    use super::unix::constants as unix_constants;
    use super::windows::constants as windows_constants;

    /// Contains constants associated with Unix paths.
    pub mod unix {
        pub use super::unix_constants::*;
    }

    /// Contains constants associated with Windows paths.
    pub mod windows {
        pub use super::windows_constants::*;
    }
}
