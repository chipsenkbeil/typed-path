use crate::{Encoding, Iter, Path};
use std::{borrow::Borrow, collections::TryReserveError, marker::PhantomData, ops::Deref};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
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

    pub fn push<P: AsRef<Path<T>>>(&mut self, _path: P) {
        todo!();
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

    /// Converts this `BytePathBuf` into a [boxed](Box) [`BytePath`].
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

impl<T> AsRef<Path<T>> for PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        self
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

impl<'a, T> IntoIterator for &'a PathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    type Item = &'a [u8];
    type IntoIter = Iter<'a, T>;
    #[inline]
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}
