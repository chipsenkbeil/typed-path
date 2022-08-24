mod display;
mod error;

pub use display::Display;
pub use error::StripPrefixError;

use crate::{Ancestors, Component, Components, Encoding, Iter, PathBuf};
use std::{
    borrow::{Cow, ToOwned},
    cmp, fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    rc::Rc,
    sync::Arc,
};

#[repr(transparent)]
pub struct Path<T>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Encoding associated with path buf
    _encoding: PhantomData<T>,

    /// Path as an unparsed byte slice
    pub(crate) inner: [u8],
}

impl<T> Path<T>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    pub fn new<S: AsRef<[u8]> + ?Sized>(s: &S) -> &Self {
        unsafe { &*(s.as_ref() as *const [u8] as *const Self) }
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

    pub fn is_absolute(&self) -> bool {
        self.components().is_absolute()
    }

    #[inline]
    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    #[inline]
    pub fn has_root(&self) -> bool {
        self.components().has_root()
    }

    pub fn parent(&self) -> Option<&Self> {
        let mut comps = self.components();
        let comp = comps.next_back();
        comp.and_then(|p| {
            if !p.is_root() {
                Some(Self::new(comps.as_bytes()))
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn ancestors(&self) -> Ancestors<T> {
        Ancestors { next: Some(self) }
    }

    pub fn file_name(&self) -> Option<&[u8]> {
        match self.components().next_back() {
            Some(p) => {
                if p.is_normal() {
                    Some(p.as_bytes())
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn strip_prefix<'a>(&'a self, base: &'a Path<T>) -> Result<&'a Path<T>, StripPrefixError> {
        self._strip_prefix(base.as_ref())
    }

    // TODO: Revisit trying to make strip_prefix above work with AsRef<Path<T>>
    //
    //       Struggled with lifetime challenges and couldn't get it to work as base was getting
    //       dropped, yet the returned path from _strip_prefix had the base reference being
    //       included in the return
    fn _strip_prefix<'a>(&'a self, base: &'a Path<T>) -> Result<&'a Path<T>, StripPrefixError> {
        match helpers::iter_after(self.components(), base.components()) {
            Some(c) => Ok(Self::new(c.as_bytes())),
            None => Err(StripPrefixError(())),
        }
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

    pub fn components(&self) -> <T as Encoding<'_>>::Components {
        T::components(&self.inner)
    }

    #[inline]
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self.components())
    }

    /// Returns an object that implements [`Display`] for safely printing paths
    /// that may contain non-Unicode data. This may perform lossy conversion,
    /// depending on the platform.  If you would like an implementation which
    /// escapes the path please use [`Debug`] instead.
    ///
    /// [`Debug`]: fmt::Debug
    /// [`Display`]: fmt::Display
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{Path, UnixEncoding};
    ///
    /// // NOTE: A path cannot be created on its own without a defined encoding
    /// let path = Path::<UnixEncoding>::new("/tmp/foo.rs");
    ///
    /// println!("{}", path.display());
    /// ```
    #[inline]
    pub fn display(&self) -> Display<T> {
        Display { path: self }
    }

    /// Converts a [`Box<Path>`](Box) into a
    /// [`PathBuf`] without copying or allocating.
    pub fn into_path_buf(self: Box<Path<T>>) -> PathBuf<T> {
        let rw = Box::into_raw(self) as *mut [u8];
        let inner = unsafe { Box::from_raw(rw) };
        PathBuf {
            _encoding: PhantomData,
            inner: inner.into_vec(),
        }
    }
}

impl<T> AsRef<[u8]> for Path<T>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl<T> AsRef<Path<T>> for Path<T>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        self
    }
}

impl<T> AsRef<Path<T>> for [u8]
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        Path::new(self)
    }
}

impl<T> AsRef<Path<T>> for Cow<'_, [u8]>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        Path::new(self)
    }
}

impl<T> AsRef<Path<T>> for str
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        Path::new(self)
    }
}

impl<T> AsRef<Path<T>> for String
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        Path::new(self)
    }
}

#[cfg(unix)]
impl<T> AsRef<Path<T>> for std::ffi::OsStr
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        use std::os::unix::ffi::OsStrExt;
        Path::new(self.as_bytes())
    }
}

#[cfg(target_os = "wasi")]
impl<T> AsRef<Path<T>> for std::ffi::OsStr
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        use std::os::wasi::ffi::OsStrExt;
        Path::new(self.as_bytes())
    }
}

#[cfg(windows)]
impl<T> AsRef<Path<T>> for std::ffi::OsStr
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        use std::os::windows::ffi::OsStrExt;

        todo!("Below produces an iterator of u16. What do we do?");
        let wide = self.encode_wide();
        Path::new(wide)
    }
}

impl<T> fmt::Debug for Path<T>
where
    T: for<'enc> Encoding<'enc>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, formatter)
    }
}

impl<T> fmt::Display for Path<T>
where
    T: for<'enc> Encoding<'enc>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl<T> cmp::PartialEq for Path<T>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn eq(&self, other: &Path<T>) -> bool {
        self.components() == other.components()
    }
}

impl<T> cmp::Eq for Path<T> where T: for<'enc> Encoding<'enc> {}

impl<T> Hash for Path<T>
where
    T: for<'enc> Encoding<'enc>,
{
    fn hash<H: Hasher>(&self, h: &mut H) {
        T::hash(self.as_bytes(), h)
    }
}

impl<T> cmp::PartialOrd for Path<T>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn partial_cmp(&self, other: &Path<T>) -> Option<cmp::Ordering> {
        self.components().partial_cmp(other.components())
    }
}

impl<T> cmp::Ord for Path<T>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn cmp(&self, other: &Path<T>) -> cmp::Ordering {
        self.components().cmp(other.components())
    }
}

impl<T> From<&Path<T>> for Box<Path<T>>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Creates a boxed [`Path`] from a reference.
    ///
    /// This will allocate and clone `path` to it.
    fn from(path: &Path<T>) -> Self {
        let boxed: Box<[u8]> = path.inner.into();
        let rw = Box::into_raw(boxed) as *mut Path<T>;
        unsafe { Box::from_raw(rw) }
    }
}

impl<T> From<Cow<'_, Path<T>>> for Box<Path<T>>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Creates a boxed [`Path`] from a clone-on-write pointer.
    ///
    /// Converting from a `Cow::Owned` does not clone or allocate.
    #[inline]
    fn from(cow: Cow<'_, Path<T>>) -> Box<Path<T>> {
        match cow {
            Cow::Borrowed(path) => Box::from(path),
            Cow::Owned(path) => Box::from(path),
        }
    }
}

impl<T> From<PathBuf<T>> for Box<Path<T>>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Converts a [`PathBuf`] into a <code>[Box]&lt;[Path]&gt;</code>.
    ///
    /// This conversion currently should not allocate memory,
    /// but this behavior is not guaranteed on all platforms or in all future versions.
    #[inline]
    fn from(p: PathBuf<T>) -> Box<Path<T>> {
        p.into_boxed_path()
    }
}

impl<'a, T> From<&'a Path<T>> for Cow<'a, Path<T>>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Creates a clone-on-write pointer from a reference to
    /// [`Path`].
    ///
    /// This conversion does not clone or allocate.
    #[inline]
    fn from(s: &'a Path<T>) -> Self {
        Cow::Borrowed(s)
    }
}

impl<'a, T> From<PathBuf<T>> for Cow<'a, Path<T>>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Creates a clone-on-write pointer from an owned
    /// instance of [`PathBuf`].
    ///
    /// This conversion does not clone or allocate.
    #[inline]
    fn from(s: PathBuf<T>) -> Self {
        Cow::Owned(s)
    }
}

impl<'a, T> From<&'a PathBuf<T>> for Cow<'a, Path<T>>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Creates a clone-on-write pointer from a reference to
    /// [`PathBuf`].
    ///
    /// This conversion does not clone or allocate.
    #[inline]
    fn from(p: &'a PathBuf<T>) -> Self {
        Cow::Borrowed(p.as_path())
    }
}

impl<T> From<PathBuf<T>> for Arc<Path<T>>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Converts a [`PathBuf`] into an <code>[Arc]<[Path]></code> by moving the [`PathBuf`] data
    /// into a new [`Arc`] buffer.
    #[inline]
    fn from(path_buf: PathBuf<T>) -> Self {
        let arc: Arc<[u8]> = Arc::from(path_buf.into_vec());
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const Path<T>) }
    }
}

impl<T> From<&Path<T>> for Arc<Path<T>>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Converts a [`Path`] into an [`Arc`] by copying the [`Path`] data into a new [`Arc`] buffer.
    #[inline]
    fn from(path: &Path<T>) -> Self {
        let arc: Arc<[u8]> = Arc::from(path.as_bytes().to_vec());
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const Path<T>) }
    }
}

impl<T> From<PathBuf<T>> for Rc<Path<T>>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Converts a [`PathBuf`] into an <code>[Rc]<[Path]></code> by moving the [`PathBuf`] data into
    /// a new [`Rc`] buffer.
    #[inline]
    fn from(path_buf: PathBuf<T>) -> Self {
        let rc: Rc<[u8]> = Rc::from(path_buf.into_vec());
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const Path<T>) }
    }
}

impl<T> From<&Path<T>> for Rc<Path<T>>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Converts a [`Path`] into an [`Rc`] by copying the [`Path`] data into a new [`Rc`] buffer.
    #[inline]
    fn from(path: &Path<T>) -> Self {
        let rc: Rc<[u8]> = Rc::from(path.as_bytes());
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const Path<T>) }
    }
}

impl<'a, T> IntoIterator for &'a Path<T>
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

impl<T> ToOwned for Path<T>
where
    T: for<'enc> Encoding<'enc>,
{
    type Owned = PathBuf<T>;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        self.to_path_buf()
    }
}

mod helpers {
    use super::*;

    pub fn rsplit_file_at_dot(file: &[u8]) -> (Option<&[u8]>, Option<&[u8]>) {
        if file == b".." {
            return (Some(file), None);
        }

        let mut iter = file.rsplitn(2, |b| *b == b'.');
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
    pub fn iter_after<'a, T, I, J>(mut iter: I, mut prefix: J) -> Option<I>
    where
        T: Component<'a>,
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
