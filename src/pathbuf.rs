use crate::{UnixPath, WindowsPath};
use paste::paste;
use std::{borrow::Borrow, collections::TryReserveError, ops::Deref};

macro_rules! impl_path_buf {
    ($platform:ident $sep:literal $prefix:literal) => {
        paste! {
            #[doc = "UTF-8 version of [`std::path::PathBuf`] for " [<$platform:camel>]]
            #[doc = ""]
            #[doc = " This type provides methods like [`push`] and [`set_extension`] that mutate"]
            #[doc = " the path in place. It also implements [`Deref`] to [`" [<$platform:camel Path>] "`], meaning that"]
            #[doc = " all methods on [`" [<$platform:camel Path>] "`] slices are available on [`" [<$platform:camel PathBuf>] "`] values as well."]
            #[doc = ""]
            #[doc = " [`push`]: " [<$platform:camel PathBuf>] "::push"]
            #[doc = " [`set_extension`]: " [<$platform:camel PathBuf>] "::set_extension"]
            #[doc = ""]
            #[doc = " More details about the overall approach can be found in"]
            #[doc = " the [module documentation](self)."]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " You can use [`push`] to build up a [`" [<$platform:camel PathBuf>] "`] from"]
            #[doc = " components:"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::" [<$platform:camel PathBuf>] ";"]
            #[doc = ""]
            #[doc = " let mut path = " [<$platform:camel PathBuf>] "::new();"]
            #[doc = ""]
            #[doc = " path.push(\"" $prefix $sep "\");"]
            #[doc = " path.push(\"some\");"]
            #[doc = " path.push(\"path\");"]
            #[doc = ""]
            #[doc = " path.set_extension(\"txt\");"]
            #[doc = " ```"]
            #[doc = ""]
            #[doc = " However, [`push`] is best used for dynamic situations. This is a better way"]
            #[doc = " to do this when you know all of the components ahead of time:"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::" [<$platform:camel PathBuf>] ";"]
            #[doc = ""]
            #[doc = " let path: " [<$platform:camel PathBuf>] " = [\"" $prefix $sep "\", \"some\", \"path.txt\"].iter().collect();"]
            #[doc = " ```"]
            #[doc = ""]
            #[doc = " We can still do better than this! Since these are all strings, we can use"]
            #[doc = " `From::from`:"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::" [<$platform:camel PathBuf>] ";"]
            #[doc = ""]
            #[doc = " let path = " [<$platform:camel PathBuf>] "::from(\"" $prefix $sep "some" $sep "path.txt\");"]
            #[doc = " ```"]
            #[doc = ""]
            #[doc = " Which method works best depends on what kind of situation you're in."]
            #[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
            pub struct [<$platform:camel PathBuf>] {
                /// Path as an unparsed `String`
                pub(crate) inner: String,
            }

            impl [<$platform:camel PathBuf>] {
                #[doc = "Allocates an empty [`" [<$platform:camel PathBuf>] "`]."]
                #[doc = ""]
                #[doc = "# Examples"]
                #[doc = ""]
                #[doc = "```"]
                #[doc = "use typed_path::" [<$platform:camel PathBuf>] ";"]
                #[doc = ""]
                #[doc = "let path = " [<$platform:camel PathBuf>] "::new();"]
                #[doc = "```"]
                pub fn new() -> [<$platform:camel PathBuf>] {
                    [<$platform:camel PathBuf>] {
                        inner: String::new(),
                    }
                }

                #[doc = " Creates a new [`" [<$platform:camel PathBuf>] "`] with a given capacity used to create the"]
                #[doc = " internal [`String`]. See [`with_capacity`] defined on [`String`]."]
                #[doc = ""]
                #[doc = " # Examples"]
                #[doc = ""]
                #[doc = " ```"]
                #[doc = " use typed_path::" [<$platform:camel PathBuf>] ";"]
                #[doc = ""]
                #[doc = " let mut path = " [<$platform:camel PathBuf>] "::with_capacity(10);"]
                #[doc = " let capacity = path.capacity();"]
                #[doc = ""]
                #[doc = " // This push is done without reallocating"]
                #[doc = " path.push(\"" $prefix $sep "\");"]
                #[doc = ""]
                #[doc = " assert_eq!(capacity, path.capacity());"]
                #[doc = " ```"]
                #[doc = ""]
                #[doc = " [`with_capacity`]: String::with_capacity"]
                #[inline]
                pub fn with_capacity(capacity: usize) -> [<$platform:camel PathBuf>] {
                    [<$platform:camel PathBuf>] { inner: String::with_capacity(capacity) }
                }

                #[doc = " Coerces to a [`" [<$platform:camel Path>] "`] slice."]
                #[doc = ""]
                #[doc = " # Examples"]
                #[doc = ""]
                #[doc = " ```"]
                #[doc = " use typed_path::{" [<$platform:camel Path>] ", " [<$platform:camel PathBuf>] "};"]
                #[doc = ""]
                #[doc = " let p = " [<$platform:camel PathBuf>] "::from(\"" $sep "test\");"]
                #[doc = " assert_eq!(" [<$platform:camel Path>] "::new(\"" $sep "test\"), p.as_path());"]
                #[doc = " ```"]
                #[inline]
                pub fn as_path(&self) -> &$crate::[<$platform:camel Path>] {
                    self
                }

                #[doc = " Truncates `self` to [`self.parent`]."]
                #[doc = ""]
                #[doc = " Returns `false` and does nothing if [`self.parent`] is [`None`]."]
                #[doc = " Otherwise, returns `true`."]
                #[doc = ""]
                #[doc = " [`self.parent`]: " [<$platform:camel Path>] "::parent"]
                #[doc = ""]
                #[doc = " # Examples"]
                #[doc = ""]
                #[doc = " ```"]
                #[doc = " use typed_path::{" [<$platform:camel Path>] ", " [<$platform:camel PathBuf>] "};"]
                #[doc = ""]
                #[doc = " let mut p = " [<$platform:camel PathBuf>] "::from(\"" $sep "spirited" $sep "away.rs\");"]
                #[doc = ""]
                #[doc = " p.pop();"]
                #[doc = " assert_eq!(" [<$platform:camel Path>] "::new(\"" $sep "spirited\"), p);"]
                #[doc = " p.pop();"]
                #[doc = " assert_eq!(" [<$platform:camel Path>] "::new(\"" $sep "\"), p);"]
                #[doc = " ```"]
                pub fn pop(&mut self) -> bool {
                    match self.parent().map(|p| p.as_str().len()) {
                        Some(len) => {
                            self.inner.truncate(len);
                            true
                        }
                        None => false,
                    }
                }

                #[doc = " Updates [`self.file_name`] to `file_name`."]
                #[doc = ""]
                #[doc = " If [`self.file_name`] was [`None`], this is equivalent to pushing"]
                #[doc = " `file_name`."]
                #[doc = ""]
                #[doc = " Otherwise it is equivalent to calling [`pop`] and then pushing"]
                #[doc = " `file_name`. The new path will be a sibling of the original path."]
                #[doc = " (That is, it will have the same parent.)"]
                #[doc = ""]
                #[doc = " [`self.file_name`]: " [<$platform:camel Path>] "::file_name"]
                #[doc = " [`pop`]: " [<$platform:camel PathBuf>] "::pop"]
                #[doc = ""]
                #[doc = " # Examples"]
                #[doc = ""]
                #[doc = " ```"]
                #[doc = " use typed_path::" [<$platform:camel PathBuf>] ";"]
                #[doc = ""]
                #[doc = " let mut buf = " [<$platform:camel PathBuf>] "::from(\"" $sep "\");"]
                #[doc = " assert!(buf.file_name() == None);"]
                #[doc = " buf.set_file_name(\"bar\");"]
                #[doc = " assert!(buf == " [<$platform:camel PathBuf>] "::from(\"" $sep "bar\"));"]
                #[doc = " assert!(buf.file_name().is_some());"]
                #[doc = " buf.set_file_name(\"baz.txt\");"]
                #[doc = " assert!(buf == " [<$platform:camel PathBuf>] "::from(\"" $sep "baz.txt\"));"]
                #[doc = " ```"]
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

                #[doc = " Updates [`self.extension`] to `extension`."]
                #[doc = ""]
                #[doc = " Returns `false` and does nothing if [`self.file_name`] is [`None`],"]
                #[doc = " returns `true` and updates the extension otherwise."]
                #[doc = ""]
                #[doc = " If [`self.extension`] is [`None`], the extension is added; otherwise"]
                #[doc = " it is replaced."]
                #[doc = ""]
                #[doc = " [`self.file_name`]: " [<$platform:camel Path>] "::file_name"]
                #[doc = " [`self.extension`]: " [<$platform:camel Path>] "::extension"]
                #[doc = ""]
                #[doc = " # Examples"]
                #[doc = ""]
                #[doc = " ```"]
                #[doc = " use typed_path::{" [<$platform:camel Path>] ", " [<$platform:camel PathBuf>] "};"]
                #[doc = ""]
                #[doc = " let mut p = " [<$platform:camel PathBuf>] "::from(\"" $sep "feel" $sep "the\");"]
                #[doc = ""]
                #[doc = " p.set_extension(\"force\");"]
                #[doc = " assert_eq!(" [<$platform:camel Path>] "::new(\"" $sep "feel" $sep "the.force\"), p.as_path());"]
                #[doc = ""]
                #[doc = " p.set_extension(\"dark_side\");"]
                #[doc = " assert_eq!(" [<$platform:camel Path>] "::new(\"" $sep "feel" $sep "the.dark_side\"), p.as_path());"]
                #[doc = " ```"]
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
                        if self.inner.chars().next() != Some('.') {
                            self.inner.push('.');
                        }

                        self.inner.push_str(extension);
                    }

                    true
                }

                #[doc = " Consumes the [`" [<$platform:camel PathBuf>] "`], yielding its internal [`String`] storage."]
                #[doc = ""]
                #[doc = " # Examples"]
                #[doc = ""]
                #[doc = " ```"]
                #[doc = " use typed_path::" [<$platform:camel PathBuf>] ";"]
                #[doc = ""]
                #[doc = " let p = " [<$platform:camel PathBuf>] "::from(\"" $sep "the" $sep "head\");"]
                #[doc = " let s = p.into_string();"]
                #[doc = " ```"]
                #[inline]
                pub fn into_string(self) -> String {
                    self.inner
                }

                #[doc = "Converts this `" [<$platform:camel PathBuf>] "` into a [boxed](Box) [`" [<$platform:camel Path>] "`]."]
                #[inline]
                pub fn into_boxed_path(self) -> Box<[<$platform:camel Path>]> {
                    let rw = Box::into_raw(self.inner.into_boxed_str()) as *mut [<$platform:camel Path>];
                    unsafe { Box::from_raw(rw) }
                }

                /// Invokes [`capacity`] on the underlying instance of [`String`].
                ///
                /// [`capacity`]: String::capacity
                #[inline]
                pub fn capacity(&self) -> usize {
                    self.inner.capacity()
                }

                /// Invokes [`clear`] on the underlying instance of [`String`].
                ///
                /// [`clear`]: String::clear
                #[inline]
                pub fn clear(&mut self) {
                    self.inner.clear()
                }

                /// Invokes [`reserve`] on the underlying instance of [`String`].
                ///
                /// [`reserve`]: String::reserve
                #[inline]
                pub fn reserve(&mut self, additional: usize) {
                    self.inner.reserve(additional)
                }

                /// Invokes [`try_reserve`] on the underlying instance of [`String`].
                ///
                /// [`try_reserve`]: String::try_reserve
                #[inline]
                pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
                    self.inner.try_reserve(additional)
                }

                /// Invokes [`reserve_exact`] on the underlying instance of [`String`].
                ///
                /// [`reserve_exact`]: String::reserve_exact
                #[inline]
                pub fn reserve_exact(&mut self, additional: usize) {
                    self.inner.reserve_exact(additional)
                }

                /// Invokes [`try_reserve_exact`] on the underlying instance of [`String`].
                ///
                /// [`try_reserve_exact`]: String::try_reserve_exact
                #[inline]
                pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
                    self.inner.try_reserve_exact(additional)
                }

                /// Invokes [`shrink_to_fit`] on the underlying instance of [`String`].
                ///
                /// [`shrink_to_fit`]: String::shrink_to_fit
                #[inline]
                pub fn shrink_to_fit(&mut self) {
                    self.inner.shrink_to_fit()
                }

                /// Invokes [`shrink_to`] on the underlying instance of [`String`].
                ///
                /// [`shrink_to`]: String::shrink_to
                #[inline]
                pub fn shrink_to(&mut self, min_capacity: usize) {
                    self.inner.shrink_to(min_capacity)
                }
            }

            impl AsRef<$crate::[<$platform:camel Path>]> for [<$platform:camel PathBuf>] {
                #[inline]
                fn as_ref(&self) -> &$crate::[<$platform:camel Path>] {
                    self
                }
            }

            impl Deref for [<$platform:camel PathBuf>] {
                type Target = $crate::[<$platform:camel Path>];

                #[inline]
                fn deref(&self) -> &$crate::[<$platform:camel Path>] {
                    $crate::[<$platform:camel Path>]::new(&self.inner)
                }
            }

            impl Borrow<$crate::[<$platform:camel Path>]> for [<$platform:camel PathBuf>] {
                #[inline]
                fn borrow(&self) -> &$crate::[<$platform:camel Path>] {
                    self.deref()
                }
            }

            impl Default for [<$platform:camel PathBuf>] {
                #[inline]
                fn default() -> Self {
                    [<$platform:camel PathBuf>]::new()
                }
            }
        }
    };
}

impl_path_buf!(Windows "\\\\" "C:");
impl_path_buf!(Unix "/" "");

impl WindowsPathBuf {
    /// Extends `self` with `path`.
    ///
    /// If `path` is absolute, it replaces the current path.
    ///
    /// On Windows:
    ///
    /// * if `path` has a root but no prefix (e.g., `\windows`), it
    ///   replaces everything except for the prefix (if any) of `self`.
    /// * if `path` has a prefix but no root, it replaces `self`.
    /// * if `self` has a verbatim prefix (e.g. `\\?\C:\windows`)
    ///   and `path` is not empty, the new path is normalized: all references
    ///   to `.` and `..` are removed.
    ///
    /// # Examples
    ///
    /// Pushing a relative path extends the existing path:
    ///
    /// ```
    /// use typed_path::WindowsPathBuf;
    ///
    /// let mut path = WindowsPathBuf::from(r"C:\tmp");
    /// path.push("file.bk");
    /// assert_eq!(path, WindowsPathBuf::from("C:\tmp\file.bk"));
    /// ```
    ///
    /// Pushing an absolute path replaces the existing path:
    ///
    /// ```
    /// use typed_path::WindowsPathBuf;
    ///
    /// let mut path = WindowsPathBuf::from("C:\tmp");
    /// path.push("\etc");
    /// assert_eq!(path, WindowsPathBuf::from("C:\etc"));
    /// ```
    pub fn push<P: AsRef<WindowsPath>>(&mut self, path: P) {
        todo!();
    }
}

impl UnixPathBuf {
    /// Extends `self` with `path`.
    ///
    /// If `path` is absolute, it replaces the current path.
    ///
    /// # Examples
    ///
    /// Pushing a relative path extends the existing path:
    ///
    /// ```
    /// use typed_path::UnixPathBuf;
    ///
    /// let mut path = UnixPathBuf::from("/tmp");
    /// path.push("file.bk");
    /// assert_eq!(path, UnixPathBuf::from("/tmp/file.bk"));
    /// ```
    ///
    /// Pushing an absolute path replaces the existing path:
    ///
    /// ```
    /// use typed_path::UnixPathBuf;
    ///
    /// let mut path = UnixPathBuf::from("/tmp");
    /// path.push("/etc");
    /// assert_eq!(path, UnixPathBuf::from("/etc"));
    /// ```
    pub fn push<P: AsRef<UnixPath>>(&mut self, path: P) {
        todo!();
    }
}
