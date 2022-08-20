mod component;
mod separator;

pub use component::*;
pub use separator::*;

use std::{
    cmp,
    collections::VecDeque,
    iter::{DoubleEndedIterator, FusedIterator},
};

/// Interface to provide meaning to a byte slice such that paths can be derived
pub trait Encoding<'a>: Sized {
    /// Represents the type of component that will be derived by this encoding
    type Component: Component;

    /// Represents the path separator tied to this encoding
    type Separator: Separator;

    /// Wraps a byte slice in a parser of [`ByteComponent`]s
    fn components(bytes: &'a [u8]) -> Components<'a, Self>;
}

/// Represents an iterator over a collection of [`ByteComponent`]s
pub struct Components<'a, T: Encoding<'a>> {
    /// Represents raw byte slice that comprises the remaining components
    raw: &'a [u8],

    /// Represents the parsed components
    components: VecDeque<T::Component>,
}

impl<'a, T: Encoding<'a>> Iterator for Components<'a, T> {
    type Item = T::Component;

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

impl<'a, T: Encoding<'a>> DoubleEndedIterator for Components<'a, T> {
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

impl<'a, T: Encoding<'a>> FusedIterator for Components<'a, T> {}

impl<'a, T> cmp::PartialEq for Components<'a, T>
where
    T: Encoding<'a>,
    T::Component: cmp::PartialEq,
{
    #[inline]
    fn eq(&self, other: &Components<'a, T>) -> bool {
        self.components == other.components
    }
}

impl<'a, T> cmp::Eq for Components<'a, T>
where
    T: Encoding<'a>,
    T::Component: cmp::Eq,
{
}

impl<'a, T> cmp::PartialOrd for Components<'a, T>
where
    T: Encoding<'a>,
    T::Component: cmp::PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Components<'a, T>) -> Option<cmp::Ordering> {
        self.components.partial_cmp(&other.components)
    }
}

impl<'a, T> cmp::Ord for Components<'a, T>
where
    T: Encoding<'a>,
    T::Component: cmp::Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.components.cmp(&other.components)
    }
}
