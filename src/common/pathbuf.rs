use crate::{Encoding, Iter, Path};
use std::{
    borrow::Borrow,
    cmp,
    collections::TryReserveError,
    hash::{Hash, Hasher},
    iter::{Extend, FromIterator},
    marker::PhantomData,
    ops::Deref,
};

/// An owned, mutable path that mirrors [`std::path::PathBuf`], but operatings using an
/// [`Encoding`] to determine how to parse the underlying bytes.
///
/// This type provides methods like [`push`] and [`set_extension`] that mutate
/// the path in place. It also implements [`Deref`] to [`Path`], meaning that
/// all methods on [`Path`] slices are available on `PathBuf` values as well.
///
/// [`push`]: PathBuf::push
/// [`set_extension`]: PathBuf::set_extension
///
/// # Examples
///
/// You can use [`push`] to build up a `PathBuf` from
/// components:
///
/// ```
/// use typed_path::{PathBuf, WindowsEncoding};
///
/// // NOTE: A pathbuf cannot be created on its own without a defined encoding
/// let mut path = PathBuf::<WindowsEncoding>::new();
///
/// path.push(r"C:\");
/// path.push("windows");
/// path.push("system32");
///
/// path.set_extension("dll");
/// ```
///
/// However, [`push`] is best used for dynamic situations. This is a better way
/// to do this when you know all of the components ahead of time:
///
/// ```
/// use typed_path::{PathBuf, WindowsEncoding};
///
/// let path: PathBuf<WindowsEncoding> = [r"C:\", "windows", "system32.dll"].iter().collect();
/// ```
///
/// We can still do better than this! Since these are all strings, we can use
/// `From::from`:
///
/// ```
/// use typed_path::{PathBuf, WindowsEncoding};
///
/// let path = PathBuf::<WindowsEncoding>::from(br"C:\windows\system32.dll");
/// ```
///
/// Which method works best depends on what kind of situation you're in.
#[derive(Clone, Debug)]
pub struct PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Encoding associated with path buf
    pub(crate) _encoding: PhantomData<T>,

    /// Path as an unparsed collection of bytes
    pub(crate) inner: Vec<u8>,
}

impl<T> PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    pub fn new() -> Self {
        PathBuf {
            inner: Vec::new(),
            _encoding: PhantomData,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        PathBuf {
            inner: Vec::with_capacity(capacity),
            _encoding: PhantomData,
        }
    }

    #[inline]
    pub fn as_path(&self) -> &Path<T> {
        self
    }

    pub fn push<P: AsRef<Path<T>>>(&mut self, path: P) {
        T::push(&mut self.inner, path.as_ref().as_bytes());
    }

    pub fn pop(&mut self) -> bool {
        match self.parent().map(|p| p.as_bytes().len()) {
            Some(len) => {
                self.inner.truncate(len);
                true
            }
            None => false,
        }
    }

    pub fn set_file_name<S: AsRef<[u8]>>(&mut self, file_name: S) {
        self._set_file_name(file_name.as_ref())
    }

    fn _set_file_name(&mut self, file_name: &[u8]) {
        if self.file_name().is_some() {
            let popped = self.pop();
            debug_assert!(popped);
        }
        self.push(file_name);
    }

    pub fn set_extension<S: AsRef<[u8]>>(&mut self, extension: S) -> bool {
        self._set_extension(extension.as_ref())
    }

    fn _set_extension(&mut self, extension: &[u8]) -> bool {
        if self.file_stem().is_none() {
            return false;
        }

        let old_ext_len = self.extension().map(|ext| ext.len()).unwrap_or(0);

        // Truncate to remove the extension
        if old_ext_len > 0 {
            self.inner.truncate(self.inner.len() - old_ext_len);
        }

        // Add the new extension if it exists
        if !extension.is_empty() {
            // Add a '.' at the end prior to adding the extension
            if self.inner.first() != Some(&b'.') {
                self.inner.push(b'.');
            }

            self.inner.extend_from_slice(extension);
        }

        true
    }

    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        self.inner
    }

    /// Converts this [`PathBuf`] into a [boxed](Box) [`Path`].
    #[inline]
    pub fn into_boxed_path(self) -> Box<Path<T>> {
        let rw = Box::into_raw(self.inner.into_boxed_slice()) as *mut Path<T>;
        unsafe { Box::from_raw(rw) }
    }

    /// Invokes [`capacity`] on the underlying instance of [`Vec`].
    ///
    /// [`capacity`]: Vec::capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Invokes [`clear`] on the underlying instance of [`Vec`].
    ///
    /// [`clear`]: Vec::clear
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Invokes [`reserve`] on the underlying instance of [`Vec`].
    ///
    /// [`reserve`]: Vec::reserve
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }

    /// Invokes [`try_reserve`] on the underlying instance of [`Vec`].
    ///
    /// [`try_reserve`]: Vec::try_reserve
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve(additional)
    }

    /// Invokes [`reserve_exact`] on the underlying instance of [`Vec`].
    ///
    /// [`reserve_exact`]: Vec::reserve_exact
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional)
    }

    /// Invokes [`try_reserve_exact`] on the underlying instance of [`Vec`].
    ///
    /// [`try_reserve_exact`]: Vec::try_reserve_exact
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve_exact(additional)
    }

    /// Invokes [`shrink_to_fit`] on the underlying instance of [`Vec`].
    ///
    /// [`shrink_to_fit`]: Vec::shrink_to_fit
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }

    /// Invokes [`shrink_to`] on the underlying instance of [`Vec`].
    ///
    /// [`shrink_to`]: Vec::shrink_to
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity)
    }
}

impl<T> AsRef<[u8]> for PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<T> AsRef<Path<T>> for PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        self
    }
}

impl<T> Borrow<Path<T>> for PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn borrow(&self) -> &Path<T> {
        self.deref()
    }
}

impl<T> Default for PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn default() -> PathBuf<T> {
        PathBuf::new()
    }
}

impl<T> Deref for PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    type Target = Path<T>;

    #[inline]
    fn deref(&self) -> &Path<T> {
        Path::new(&self.inner)
    }
}

impl<T> Eq for PathBuf<T> where T: for<'enc> Encoding<'enc> {}

impl<T> PartialEq for PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    fn eq(&self, other: &Self) -> bool {
        self.components() == other.components()
    }
}

impl<T, P> Extend<P> for PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
    P: AsRef<Path<T>>,
{
    fn extend<I: IntoIterator<Item = P>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |p| self.push(p.as_ref()));
    }
}

impl<T, P> FromIterator<P> for PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
    P: AsRef<Path<T>>,
{
    fn from_iter<I: IntoIterator<Item = P>>(iter: I) -> Self {
        let mut buf = PathBuf::new();
        buf.extend(iter);
        buf
    }
}

impl<T> Hash for PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.as_path().hash(h)
    }
}

impl<'a, T> IntoIterator for &'a PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    type Item = &'a [u8];
    type IntoIter = Iter<'a, T>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> cmp::PartialOrd for PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.components().partial_cmp(other.components())
    }
}

impl<T> cmp::Ord for PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.components().cmp(other.components())
    }
}
