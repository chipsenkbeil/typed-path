mod components;
mod iter;
mod path;
mod pathbuf;
mod stdlib;

#[macro_use]
pub(crate) mod parser;

pub use components::*;
pub use iter::*;
pub use parser::ParseError;
pub use path::*;
pub use pathbuf::*;

use crate::private;
use std::{fmt, hash::Hasher};

/// Interface to provide meaning to a byte slice such that paths can be derived
pub trait Encoding<'a>: Clone + fmt::Debug + fmt::Display + Sized + private::Sealed {
    /// Represents the type of component that will be derived by this encoding
    type Components: Components<'a>;

    /// Produces an iterator of [`Component`]s over the given the byte slice (`path`)
    fn components(path: &'a [u8]) -> Self::Components;

    /// Hashes a byte slice (`path`)
    fn hash<H: Hasher>(path: &[u8], h: &mut H);

    /// Pushes a byte slice (`path`) onto the an existing path (`current_path`)
    fn push(current_path: &mut Vec<u8>, path: &[u8]);
}
