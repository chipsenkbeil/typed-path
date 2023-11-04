use core::fmt;
use core::iter::FusedIterator;

use crate::common::{Utf8Ancestors, Utf8Iter};
use crate::typed::Utf8TypedPath;
use crate::unix::Utf8UnixEncoding;
use crate::windows::Utf8WindowsEncoding;

/// An iterator over the [`Utf8TypedComponent`]s of a [`Utf8TypedPath`], as [`str`] slices.
///
/// This `struct` is created by the [`iter`] method on [`Utf8TypedPath`].
/// See its documentation for more.
///
/// [`iter`]: Utf8TypedPath::iter
/// [`Utf8TypedComponent`]: crate::Utf8TypedComponent
#[derive(Clone)]
pub enum Utf8TypedIter<'a> {
    Unix(Utf8Iter<'a, Utf8UnixEncoding>),
    Windows(Utf8Iter<'a, Utf8WindowsEncoding>),
}

impl<'a> Utf8TypedIter<'a> {
    /// Extracts a slice corresponding to the portion of the path remaining for iteration.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::Utf8TypedPath;
    ///
    /// let mut iter = Utf8TypedPath::derive("/tmp/foo/bar.txt").iter();
    /// iter.next();
    /// iter.next();
    ///
    /// assert_eq!(Utf8TypedPath::derive("foo/bar.txt"), iter.to_path());
    /// ```
    pub fn to_path(&self) -> Utf8TypedPath {
        match self {
            Self::Unix(it) => Utf8TypedPath::Unix(it.as_path()),
            Self::Windows(it) => Utf8TypedPath::Windows(it.as_path()),
        }
    }

    /// Returns reference to the underlying str slice represented by this iterator.
    pub fn as_str(&self) -> &str {
        impl_typed_fn!(self, as_ref)
    }
}

impl<'a> fmt::Debug for Utf8TypedIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a>(Utf8TypedPath<'a>);

        impl<'a> fmt::Debug for DebugHelper<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter()).finish()
            }
        }

        f.debug_tuple(stringify!(Utf8TypedIter))
            .field(&DebugHelper(self.to_path()))
            .finish()
    }
}

impl<'a> AsRef<[u8]> for Utf8TypedIter<'a> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl<'a> AsRef<str> for Utf8TypedIter<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Iterator for Utf8TypedIter<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Unix(it) => it.next(),
            Self::Windows(it) => it.next(),
        }
    }
}

impl<'a> DoubleEndedIterator for Utf8TypedIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Unix(it) => it.next_back(),
            Self::Windows(it) => it.next_back(),
        }
    }
}

impl<'a> FusedIterator for Utf8TypedIter<'a> {}

/// An iterator over [`Utf8TypedPath`] and its ancestors.
///
/// This `struct` is created by the [`ancestors`] method on [`Utf8TypedPath`].
/// See its documentation for more.
///
/// # Examples
///
/// ```
/// use typed_path::Utf8TypedPath;
///
/// let path = Utf8TypedPath::derive("/foo/bar");
///
/// for ancestor in path.ancestors() {
///     println!("{}", ancestor);
/// }
/// ```
///
/// [`ancestors`]: Utf8TypedPath::ancestors
#[derive(Copy, Clone, Debug)]
pub enum Utf8TypedAncestors<'a> {
    Unix(Utf8Ancestors<'a, Utf8UnixEncoding>),
    Windows(Utf8Ancestors<'a, Utf8WindowsEncoding>),
}

impl<'a> Iterator for Utf8TypedAncestors<'a> {
    type Item = Utf8TypedPath<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Unix(it) => it.next().map(Utf8TypedPath::Unix),
            Self::Windows(it) => it.next().map(Utf8TypedPath::Windows),
        }
    }
}

impl<'a> FusedIterator for Utf8TypedAncestors<'a> {}
