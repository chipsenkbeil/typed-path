mod components;
mod iter;
mod path;
mod pathbuf;

#[macro_use]
pub(crate) mod parser;

pub use components::*;
pub use iter::*;
pub use parser::ParseError;
pub use path::*;
pub use pathbuf::*;

use crate::private;

/// Interface to provide meaning to a byte slice such that paths can be derived
pub trait Encoding<'a>: Clone + Sized + private::Sealed {
    /// Represents the type of component that will be derived by this encoding
    type Components: Components<'a>;

    /// Produces an iterator of [`Component`]s over the given the byte slice (`path`)
    fn components(path: &'a [u8]) -> Self::Components;

    /// Pushes a byte slice (`path`) onto the an existing path (`current_path`)
    fn push(current_path: &mut Vec<u8>, path: &[u8]);
}
