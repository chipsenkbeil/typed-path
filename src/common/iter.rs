use crate::{Component, Components, Encoding, Path};
use std::{fmt, iter::FusedIterator, marker::PhantomData};

pub struct Iter<'a, T>
where
    T: for<'enc> Encoding<'a>,
{
    _encoding: PhantomData<T>,
    inner: <T as Encoding<'a>>::Components,
}

impl<'a, T> Iter<'a, T>
where
    T: for<'enc> Encoding<'enc>,
{
    pub(crate) fn new(inner: impl Components<'a>) -> Self {
        Self {
            _encoding: PhantomData,
            inner,
        }
    }

    pub fn as_path(&self) -> &'a Path<T> {
        self.inner.as_path()
    }
}

impl<'a, T> fmt::Debug for Iter<'a, T>
where
    T: for<'enc> Encoding<'enc> + 'a,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a, T>(&'a Path<T>)
        where
            T: for<'enc> Encoding<'enc>;

        impl<'a, T> fmt::Debug for DebugHelper<'a, T>
        where
            T: for<'enc> Encoding<'enc>,
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
    T: for<'enc> Encoding<'enc> + 'a,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        self.as_path()
    }
}

impl<'a, T> AsRef<[u8]> for Iter<'a, T>
where
    T: for<'enc> Encoding<'enc> + 'a,
{
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_path().as_bytes()
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: for<'enc> Encoding<'enc> + 'a,
{
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(c) => Some(c.as_bytes()),
            None => None,
        }
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: for<'enc> Encoding<'enc> + 'a,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.inner.next_back() {
            Some(c) => Some(c.as_bytes()),
            None => None,
        }
    }
}

impl<'a, T> FusedIterator for Iter<'a, T> where T: for<'enc> Encoding<'enc> + 'a {}

pub struct Ancestors<'a, T>
where
    T: for<'enc> Encoding<'enc>,
{
    pub(crate) next: Option<&'a Path<T>>,
}

impl<'a, T> Iterator for Ancestors<'a, T>
where
    T: for<'enc> Encoding<'enc>,
{
    type Item = &'a Path<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;
        self.next = next.and_then(Path::parent);
        next
    }
}

impl<'a, T> FusedIterator for Ancestors<'a, T> where T: for<'enc> Encoding<'enc> {}
