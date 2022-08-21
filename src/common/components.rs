mod component;
mod separator;

pub use component::*;
pub use separator::*;

use crate::{private, Path};
use std::{
    cmp,
    collections::VecDeque,
    fmt,
    iter::{DoubleEndedIterator, FusedIterator},
};

/// Interface to provide meaning to a byte slice such that paths can be derived
pub trait Encoding<'a>: Clone + Sized + private::Sealed {
    /// Represents the type of component that will be derived by this encoding
    type Component: Component<'a>;

    /// Represents the path separator tied to this encoding
    type Separator: Separator;

    /// Pushes a byte slice (`path`) onto the an existing path (`bytes`)
    fn push(bytes: &mut Vec<u8>, path: &[u8]);

    /// Wraps a byte slice in a parser of [`Component`]s
    fn components(bytes: &'a [u8]) -> Components<'a, Self>;

    /// Reports back whether the provided byte slice represents an absolute path
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
    fn is_absolute(bytes: &[u8]) -> bool;

    /// Returns `true` if the provided byte slice represents a path that has a root.
    ///
    /// The definition of what it means for a path to have a root can vary:
    ///
    /// * On Unix, a path has a root if it begins with `/`.
    ///
    /// * On Windows, a path has a root if it:
    ///     * has no prefix and begins with a separator, e.g., `\windows`
    ///     * has a prefix followed by a separator, e.g., `c:\windows` but not `c:windows`
    ///     * has any non-disk prefix, e.g., `\\server\share`
    fn has_root(bytes: &[u8]) -> bool;
}

/// Represents an iterator over a collection of [`Component`]s
#[derive(Clone)]
pub struct Components<'a, T>
where
    T: Encoding<'a>,
{
    /// Represents raw byte slice that comprises the remaining components
    pub(crate) raw: &'a [u8],

    /// Represents the parsed components
    pub(crate) components: VecDeque<T::Component>,
}

impl<'a, T> Components<'a, T>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Extracts a slice corresponding to the portion of the path remaining for iteration
    pub fn as_path(&self) -> &'a Path<T> {
        Path::new(self.raw)
    }
}

impl<'a, T> fmt::Debug for Components<'a, T>
where
    T: for<'enc> Encoding<'enc> + 'a,
    for<'enc> <T as Encoding<'enc>>::Component: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a, T>(&'a Path<T>)
        where
            T: for<'enc> Encoding<'enc>,
            for<'enc> <T as Encoding<'enc>>::Component: fmt::Debug;

        impl<'a, T> fmt::Debug for DebugHelper<'a, T>
        where
            T: for<'enc> Encoding<'enc> + 'a,
            for<'enc> <T as Encoding<'enc>>::Component: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.components()).finish()
            }
        }

        f.debug_tuple("Components")
            .field(&DebugHelper(self.as_path()))
            .finish()
    }
}

impl<'a, T> Iterator for Components<'a, T>
where
    T: Encoding<'a>,
{
    type Item = <T as Encoding<'a>>::Component;

    fn next(&mut self) -> Option<Self::Item> {
        let component = self.components.pop_front();

        // We need to adjust our raw str to advance by the len of the component and all
        // separators leading to the next component
        if let Some(c) = component.as_ref() {
            // Advance by the len of the component
            self.raw = &self.raw[c.len()..];

            // Now advance while we still have separators in front of our next component
            self.raw = match T::Separator::split_once(self.raw) {
                Some((_, right)) => right,
                None => b"",
            };
        }

        component
    }
}

impl<T> DoubleEndedIterator for Components<'_, T>
where
    T: for<'enc> Encoding<'enc>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let component = self.components.pop_back();

        // We need to adjust our raw str to trim from the end by the len of the component and all
        // separators leading to the previous component
        if let Some(c) = component.as_ref() {
            // Trim from end by the len of the component
            self.raw = &self.raw[..=(self.raw.len() - c.len())];

            // Now trim from end while we still have separators in after of our last component
            self.raw = match T::Separator::rsplit_once(self.raw) {
                Some((left, _)) => left,
                None => b"",
            };
        }

        component
    }
}

impl<T> FusedIterator for Components<'_, T> where T: for<'enc> Encoding<'enc> {}

impl<'a, T> cmp::PartialEq for Components<'a, T>
where
    T: for<'enc> Encoding<'enc>,
    for<'enc> <T as Encoding<'enc>>::Component: cmp::PartialEq,
{
    #[inline]
    fn eq(&self, other: &Components<'a, T>) -> bool {
        self.components == other.components
    }
}

impl<T> cmp::Eq for Components<'_, T>
where
    T: for<'enc> Encoding<'enc>,
    for<'enc> <T as Encoding<'enc>>::Component: cmp::Eq,
{
}

impl<'a, T> cmp::PartialOrd for Components<'a, T>
where
    T: for<'enc> Encoding<'enc>,
    for<'enc> <T as Encoding<'enc>>::Component: cmp::PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Components<'a, T>) -> Option<cmp::Ordering> {
        self.components.partial_cmp(&other.components)
    }
}

impl<T> cmp::Ord for Components<'_, T>
where
    T: for<'enc> Encoding<'enc>,
    for<'enc> <T as Encoding<'enc>>::Component: cmp::Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.components.cmp(&other.components)
    }
}
