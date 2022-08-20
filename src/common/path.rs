use crate::{Ancestors, Component, Components, Encoding, Iter, PathBuf};
use std::{
    borrow::{Cow, ToOwned},
    cmp, fmt,
    marker::PhantomData,
};

/// Represents an error occurred while attempting to strip prefix off path
pub struct StripPrefixError(());

#[repr(transparent)]
pub struct Path<T>
where
    T: for<'a> Encoding<'a>,
{
    /// Encoding associated with path buf
    _encoding: PhantomData<T>,

    /// Path as an unparsed byte slice
    pub(crate) inner: [u8],
}

impl<T> Path<T>
where
    T: for<'a> Encoding<'a>,
{
    #[inline]
    pub fn new<S: AsRef<[u8]> + ?Sized>(s: &S) -> &Self {
        unsafe { &*(s.as_ref() as *const [u8] as *const Path) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    pub fn to_path_buf(&self) -> PathBuf<T> {
        PathBuf {
            inner: self.inner.to_owned(),
            _encoding: PhantomData,
        }
    }

    #[inline]
    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    pub fn parent(&self) -> Option<&Self> {
        let mut comps = self.components();
        let comp = comps.next_back();
        comp.and_then(|p| {
            if !p.is_root() {
                Some(comps.as_path())
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn ancestors(&self) -> Ancestors<'_, T> {
        Ancestors { next: Some(&self) }
    }

    pub fn file_name(&self) -> Option<&[u8]> {
        self.components().next_back().and_then(|p| {
            if p.is_normal() {
                Some(p.as_bytes())
            } else {
                None
            }
        })
    }

    pub fn strip_prefix<P>(&self, base: P) -> Result<&Path<T>, StripPrefixError>
    where
        P: AsRef<Path<T>>,
    {
        self._strip_prefix(base.as_ref())
    }

    fn _strip_prefix(&self, base: &Path<T>) -> Result<&Path<T>, StripPrefixError> {
        helpers::iter_after(self.components(), base.components())
            .map(|c| c.as_path())
            .ok_or(StripPrefixError)
    }

    pub fn starts_with<P>(&self, base: P) -> bool
    where
        P: AsRef<Path<T>>,
    {
        self._starts_with(base.as_ref())
    }

    fn _starts_with(&self, base: &Path<T>) -> bool {
        helpers::iter_after(self.components(), base.components()).is_some()
    }

    pub fn ends_with<P>(&self, child: P) -> bool
    where
        P: AsRef<Path<T>>,
    {
        self._ends_with(child.as_ref())
    }

    fn _ends_with(&self, child: &Path<T>) -> bool {
        helpers::iter_after(self.components().rev(), child.components().rev()).is_some()
    }

    pub fn file_stem(&self) -> Option<&[u8]> {
        self.file_name()
            .map(helpers::rsplit_file_at_dot)
            .and_then(|(before, after)| before.or(after))
    }

    pub fn extension(&self) -> Option<&[u8]> {
        self.file_name()
            .map(helpers::rsplit_file_at_dot)
            .and_then(|(before, after)| before.and(after))
    }

    pub fn join<P: AsRef<Path<T>>>(&self, path: P) -> PathBuf<T> {
        self._join(path.as_ref())
    }

    fn _join(&self, path: &Path<T>) -> PathBuf<T> {
        let mut buf = self.to_path_buf();
        buf.push(path);
        buf
    }

    pub fn with_file_name<S: AsRef<[u8]>>(&self, file_name: S) -> PathBuf<T> {
        self._with_file_name(file_name.as_ref())
    }

    fn _with_file_name(&self, file_name: &[u8]) -> PathBuf<T> {
        let mut buf = self.to_path_buf();
        buf.set_file_name(file_name);
        buf
    }

    pub fn with_extension<S: AsRef<[u8]>>(&self, extension: S) -> PathBuf<T> {
        self._with_extension(extension.as_ref())
    }

    fn _with_extension(&self, extension: &[u8]) -> PathBuf<T> {
        let mut buf = self.to_path_buf();
        buf.set_extension(extension);
        buf
    }

    pub fn components(&self) -> Components<T> {
        T::components(&self.inner)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self.components())
    }

    /// Converts a [`Box<BytePath>`](Box) into a
    /// [`BytePathBuf`] without copying or allocating.
    pub fn into_path_buf(self: Box<Path<T>>) -> PathBuf<T> {
        let rw = Box::into_raw(self) as *mut [u8];
        let inner = unsafe { Box::from_raw(rw) };
        PathBuf {
            inner: inner.into_vec(),
            _encoding: PhantomData,
        }
    }
}

impl<T> AsRef<[u8]> for Path<T>
where
    T: for<'a> Encoding<'a>,
{
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl<T> fmt::Debug for Path<T>
where
    T: for<'a> Encoding<'a>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, formatter)
    }
}

impl<T> fmt::Display for Path<T>
where
    T: for<'a> Encoding<'a>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl<T> cmp::PartialEq for Path<T>
where
    T: for<'a> Encoding<'a>,
{
    #[inline]
    fn eq(&self, other: &Path<T>) -> bool {
        self.components() == other.components()
    }
}

impl<T> cmp::Eq for Path<T> where T: for<'a> Encoding<'a> {}

impl<T> cmp::PartialOrd for Path<T>
where
    T: for<'a> Encoding<'a>,
{
    #[inline]
    fn partial_cmp(&self, other: &Path<T>) -> Option<cmp::Ordering> {
        self.components().partial_cmp(other.components())
    }
}

impl<T> cmp::Ord for Path<T>
where
    T: for<'a> Encoding<'a>,
{
    #[inline]
    fn cmp(&self, other: &Path<T>) -> cmp::Ordering {
        self.components().cmp(other.components())
    }
}

impl<T> AsRef<Path<T>> for Path<T>
where
    T: for<'a> Encoding<'a>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        self
    }
}

impl<T> AsRef<Path<T>> for Cow<'_, str>
where
    T: for<'a> Encoding<'a>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        Path::new(self)
    }
}

impl<T> AsRef<Path<T>> for str
where
    T: for<'a> Encoding<'a>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        Path::new(self)
    }
}

impl<T> AsRef<Path<T>> for String
where
    T: for<'a> Encoding<'a>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        Path::new(self)
    }
}

impl<T> ToOwned for Path<T>
where
    T: for<'a> Encoding<'a>,
{
    type Owned = PathBuf<T>;

    #[inline]
    fn to_owned(&self) -> PathBuf<T> {
        self.to_path_buf()
    }
}

impl<'a, T> IntoIterator for &'a Path<T>
where
    T: for<'b> Encoding<'b>,
{
    type Item = &'a [u8];
    type IntoIter = Iter<'a, T>;
    #[inline]
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

mod helpers {
    use super::*;

    pub fn rsplit_file_at_dot(file: &[u8]) -> (Option<&[u8]>, Option<&[u8]>) {
        if file.bytes() == b".." {
            return (Some(file), None);
        }

        let mut iter = file.bytes().rsplitn(2, |b| *b == b'.');
        let after = iter.next();
        let before = iter.next();
        if before == Some(b"") {
            (Some(file), None)
        } else {
            (before, after)
        }
    }

    // Iterate through `iter` while it matches `prefix`; return `None` if `prefix`
    // is not a prefix of `iter`, otherwise return `Some(iter_after_prefix)` giving
    // `iter` after having exhausted `prefix`.
    pub fn iter_after<'a, 'b, T, I, J>(mut iter: I, mut prefix: J) -> Option<I>
    where
        T: Component,
        I: Iterator<Item = T> + Clone,
        J: Iterator<Item = T>,
    {
        loop {
            let mut iter_next = iter.clone();
            match (iter_next.next(), prefix.next()) {
                (Some(ref x), Some(ref y)) if x == y => (),
                (Some(_), Some(_)) => return None,
                (Some(_), None) => return Some(iter),
                (None, None) => return Some(iter),
                (None, Some(_)) => return None,
            }
            iter = iter_next;
        }
    }
}
