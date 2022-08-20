use crate::{Component, Components, Encoding, Path};
use std::{fmt, iter::FusedIterator, marker::PhantomData};

pub struct Iter<'a, T>
where
    T: for<'b> Encoding<'b>,
{
    inner: Components<'a, T>,
    _encoding: PhantomData<T>,
}

impl<'a, T> Iter<'a, T>
where
    T: for<'b> Encoding<'b>,
{
    pub(crate) fn new(inner: Components<'a, T>) -> Self {
        Self {
            inner,
            _encoding: PhantomData,
        }
    }

    pub fn as_path(&self) -> &'a Path<T> {
        self.inner.as_path()
    }
}

impl<'a, T> fmt::Debug for Iter<'a, T>
where
    T: for<'b> Encoding<'b>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a, T>(&'a Path<T>)
        where
            T: for<'b> Encoding<'b>;

        impl<'a, T> fmt::Debug for DebugHelper<'a, T>
        where
            T: for<'b> Encoding<'b>,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter()).finish()
            }
        }

        f.debug_tuple(stringify!(Iter))
            .field(&DebugHelper(self.as_path()))
            .finish()
    }
}

impl<'a, T> AsRef<Path<T>> for Iter<'a, T>
where
    T: for<'b> Encoding<'b>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        self.as_path()
    }
}

impl<'a, T> AsRef<str> for Iter<'a, T>
where
    T: for<'b> Encoding<'b>,
{
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_path().as_str()
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: for<'b> Encoding<'b>,
{
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Component::as_bytes)
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: for<'b> Encoding<'b>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(Component::as_bytes)
    }
}

impl<'a, T> FusedIterator for Iter<'a, T> where T: for<'b> Encoding<'b> {}

pub struct Ancestors<'a, T>
where
    T: for<'b> Encoding<'b>,
{
    pub(crate) next: Option<&'a Path<T>>,
}

impl<'a, T> Iterator for Ancestors<'a, T>
where
    T: for<'b> Encoding<'b>,
{
    type Item = &'a Path<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;
        self.next = next.and_then(Path::parent);
        next
    }
}

impl<'a, T> FusedIterator for Ancestors<'a, T> where T: for<'b> Encoding<'b> {}
