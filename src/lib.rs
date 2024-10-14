#![cfg_attr(feature = "std", doc = include_str!("../README.md"))]
#![cfg_attr(not(feature = "std"), no_std)]

#[doc = include_str!("../README.md")]
#[cfg(all(doctest, feature = "std"))]
pub struct ReadmeDoctests;

extern crate alloc;

mod no_std_compat {
    #[allow(unused_imports)]
    pub use alloc::{
        boxed::Box,
        string::{String, ToString},
        vec,
        vec::Vec,
    };
}

#[macro_use]
mod common;
mod convert;
#[cfg(not(target_family = "wasm"))]
mod native;
mod typed;
mod unix;
#[cfg(all(feature = "std", not(target_family = "wasm")))]
pub mod utils;
mod windows;

mod private {
    /// Used to mark traits as sealed to prevent implements from others outside of this crate
    pub trait Sealed {}
}

pub use common::*;
pub use convert::*;
#[cfg(not(target_family = "wasm"))]
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
