use crate::{Encoding, Path};
use std::{borrow::Borrow, collections::TryReserveError, marker::PhantomData, ops::Deref};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PathBuf<T: Encoding> {
    /// Path as an unparsed collection of bytes
    pub(crate) inner: Vec<u8>,

    /// Encoding associated with path buf
    _encoding: PhantomData<T>,
}

impl<T: Encoding> PathBuf<T> {
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

    pub fn pop(&mut self) -> bool {
        match self.parent().map(|p| p.as_str().len()) {
            Some(len) => {
                self.inner.truncate(len);
                true
            }
            None => false,
        }
    }

    pub fn set_file_name<S: AsRef<str>>(&mut self, file_name: S) {
        self._set_file_name(file_name.as_ref())
    }

    fn _set_file_name(&mut self, file_name: &str) {
        if self.file_name().is_some() {
            let popped = self.pop();
            debug_assert!(popped);
        }
        self.push(file_name);
    }

    pub fn set_extension<S: AsRef<str>>(&mut self, extension: S) -> bool {
        self._set_extension(extension.as_ref())
    }

    fn _set_extension(&mut self, extension: &str) -> bool {
        if self.file_stem().is_none() {
            return false;
        }

        let old_ext_len = self.extension().map(str::len).unwrap_or(0);

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

            self.inner.push_str(extension);
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
        let rw = Box::into_raw(self.inner.into_boxed_slice()) as *mut Path;
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

impl<T: Encoding> AsRef<Path<T>> for PathBuf<T> {
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        self
    }
}

impl<T: Encoding> Deref for PathBuf<T> {
    type Target = Path<T>;

    #[inline]
    fn deref(&self) -> &Path<T> {
        Path::new(&self.inner)
    }
}

impl<T: Encoding> Borrow<Path<T>> for PathBuf<T> {
    #[inline]
    fn borrow(&self) -> &Path<T> {
        self.deref()
    }
}

impl<T: Encoding> Default for PathBuf<T> {
    #[inline]
    fn default() -> PathBuf<T> {
        PathBuf::new()
    }
}
