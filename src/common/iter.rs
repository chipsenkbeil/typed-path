use crate::{Component, Components, Encoding, Path};
use std::{fmt, iter::FusedIterator, marker::PhantomData};

pub struct Iter<'a, T: Encoding> {
    inner: Components<'a, T>,
    _encoding: PhantomData<T>,
}

impl<'a, T: Encoding> Iter<'a, T> {
    fn new(inner: Components<'a, T>) -> Self {
        Self {
            inner,
            _encoding: PhantomData,
        }
    }

    pub fn as_path(&self) -> &'a Path<T> {
        self.inner.as_path()
    }
}

impl<T: Encoding> fmt::Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a, T: Encoding>(&'a Path<T>);

        impl<T: Encoding> fmt::Debug for DebugHelper<'_, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter()).finish()
            }
        }

        f.debug_tuple(stringify!(Iter))
            .field(&DebugHelper(self.as_path()))
            .finish()
    }
}

impl<T: Encoding> AsRef<Path<T>> for Iter<'_, T> {
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        self.as_path()
    }
}

impl<T: Encoding> AsRef<str> for Iter<'_, T> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_path().as_str()
    }
}

impl<'a, T: Encoding> Iterator for Iter<'a, T> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Component::as_bytes)
    }
}

impl<'a, T: Encoding> DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(Component::as_bytes)
    }
}

impl<T: Encoding> FusedIterator for Iter<'_, T> {}

pub struct Ancestors<'a, T: Encoding> {
    pub(crate) next: Option<&'a Path<T>>,
}

impl<'a, T: Encoding> Iterator for Ancestors<'a, T> {
    type Item = &'a Path<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;
        self.next = next.and_then(Path::parent);
        next
    }
}

impl<T: Encoding> FusedIterator for Ancestors<'_, T> {}
