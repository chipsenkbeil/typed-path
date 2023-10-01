use std::fmt;
use std::iter::FusedIterator;

use crate::common::Iter;
use crate::typed::TypedPath;
use crate::unix::UnixEncoding;
use crate::windows::WindowsEncoding;
use crate::{Component, Components, Encoding, Path};

/// An iterator over the [`TypedComponent`]s of a [`TypedPath`], as [`[u8]`] slices.
///
/// This `struct` is created by the [`iter`] method on [`TypedPath`].
/// See its documentation for more.
///
/// [`iter`]: TypedPath::iter
#[derive(Clone)]
pub enum TypedIter<'a> {
    Unix(Iter<'a, UnixEncoding>),
    Windows(Iter<'a, WindowsEncoding>),
}

impl<'a> TypedIter<'a> {
    /// Extracts a slice corresponding to the portion of the path remaining for iteration.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::TypedPath;
    ///
    /// let mut iter = TypedPath::new("/tmp/foo/bar.txt").iter();
    /// iter.next();
    /// iter.next();
    ///
    /// assert_eq!(TypedPath::new("foo/bar.txt"), iter.as_path());
    /// ```
    pub fn as_path(&self) -> TypedPath {
        TypedPath::new(impl_typed_fn!(self, as_bytes))
    }
}

impl<'a> fmt::Debug for TypedIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a>(TypedPath<'a>);

        impl<'a> fmt::Debug for DebugHelper<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter()).finish()
            }
        }

        f.debug_tuple(stringify!(Iter))
            .field(&DebugHelper(self.as_path()))
            .finish()
    }
}

impl<'a> AsRef<TypedPath<'a>> for TypedIter<'a> {
    #[inline]
    fn as_ref(&self) -> &TypedPath<'a> {
        &self.as_path()
    }
}

impl<'a> AsRef<[u8]> for TypedIter<'a> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_path().as_bytes()
    }
}

impl<'a> Iterator for TypedIter<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(c) => Some(c.as_bytes()),
            None => None,
        }
    }
}

impl<'a> DoubleEndedIterator for TypedIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.inner.next_back() {
            Some(c) => Some(c.as_bytes()),
            None => None,
        }
    }
}

impl<'a> FusedIterator for TypedIter<'a> {}

/// An iterator over [`TypedPath`] and its ancestors.
///
/// This `struct` is created by the [`ancestors`] method on [`TypedPath`].
/// See its documentation for more.
///
/// # Examples
///
/// ```
/// use typed_path::{TypedPath};
///
/// let path = TypedPath::new("/foo/bar");
///
/// for ancestor in path.ancestors() {
///     println!("{}", ancestor.display());
/// }
/// ```
///
/// [`ancestors`]: Path::ancestors
#[derive(Copy, Clone, Debug)]
pub struct TypedAncestors<'a> {
    pub(crate) next: Option<&'a TypedPath<'a>>,
}

impl<'a> Iterator for TypedAncestors<'a> {
    type Item = &'a TypedPath<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;
        self.next = next.and_then(Path::parent);
        next
    }
}

impl<'a> FusedIterator for TypedAncestors<'a> {}
