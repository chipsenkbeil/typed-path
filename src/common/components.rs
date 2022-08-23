mod component;
mod separator;

pub use component::*;
pub use separator::*;

use crate::private;
use std::{cmp, fmt, iter};

/// Interface to provide meaning to a byte slice such that paths can be derived
pub trait Encoding<'a>: Clone + Sized + private::Sealed {
    /// Represents the type of component that will be derived by this encoding
    type Components: Components<'a>;

    /// Produces an iterator of [`Component`]s over the given the byte slice (`path`)
    fn components(path: &'a [u8]) -> Self::Components;

    /// Pushes a byte slice (`path`) onto the an existing path (`current_path`)
    fn push(current_path: &mut Vec<u8>, path: &[u8]);
}

/// Interface of an iterator over a collection of [`Component`]s
pub trait Components<'a>:
    Clone
    + fmt::Debug
    + cmp::PartialEq
    + cmp::Eq
    + cmp::PartialOrd
    + cmp::Ord
    + iter::Iterator<Item = Self::Component>
    + iter::DoubleEndedIterator<Item = Self::Component>
    + iter::FusedIterator
    + Sized
    + private::Sealed
{
    /// Type of [`Component`] iterated over
    type Component: Component<'a>;

    /// Extracts a slice corresponding to the portion of the path remaining for iteration
    fn as_bytes(&self) -> &[u8];

    /// Reports back whether the iterator represents an absolute path
    ///
    /// The definition of an absolute path can vary:
    ///
    /// * On Unix, a path is absolute if it starts with the root, so `is_absolute` and [`has_root`]
    ///   are equivalent.
    ///
    /// * On Windows, a path is absolute if it has a prefix and starts with the root: `c:\windows`
    ///   is absolute, while `c:temp` and `\temp` are not.
    ///
    /// [`has_root`]: Encoding::has_root
    fn is_absolute(&self) -> bool;

    /// Returns `true` if the iterator represents a path that has a root.
    ///
    /// The definition of what it means for a path to have a root can vary:
    ///
    /// * On Unix, a path has a root if it begins with `/`.
    ///
    /// * On Windows, a path has a root if it:
    ///     * has no prefix and begins with a separator, e.g., `\windows`
    ///     * has a prefix followed by a separator, e.g., `c:\windows` but not `c:windows`
    ///     * has any non-disk prefix, e.g., `\\server\share`
    fn has_root(&self) -> bool;
}
