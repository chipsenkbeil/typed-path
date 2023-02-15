mod components;
mod iter;
mod path;
mod pathbuf;

pub use components::*;
pub use iter::*;
pub use path::*;
pub use pathbuf::*;

use crate::private;
use std::hash::Hasher;

/// Interface to provide meaning to a byte slice such that paths can be derived
pub trait Utf8Encoding<'a>: private::Sealed {
    /// Represents the type of component that will be derived by this encoding
    type Components: Utf8Components<'a>;

    /// Static label representing encoding type
    fn label() -> &'static str;

    /// Produces an iterator of [`Utf8Component`]s over the given the byte slice (`path`)
    fn components(path: &'a str) -> Self::Components;

    /// Hashes a utf8 str (`path`)
    fn hash<H: Hasher>(path: &str, h: &mut H);

    /// Pushes a utf8 str (`path`) onto the an existing path (`current_path`)
    fn push(current_path: &mut String, path: &str);
}
