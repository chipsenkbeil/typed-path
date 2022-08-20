/// UTF-8 version of [`std::path::Component`] that represents a Unix-specific component
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum UnixComponent<'a> {
    RootDir,
    CurDir,
    ParentDir,
    Normal(&'a str),
}

impl<'a> UnixComponent<'a> {
    /// Extracts the underlying [`str`] slice
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::UnixPath;
    ///
    /// let path = UnixPath::new("/tmp/./foo/../bar.txt");
    /// let components: Vec<_> = path.components().map(|comp| comp.as_str()).collect();
    /// assert_eq!(&components, &["/", "tmp", ".", "foo", ".", "bar.txt"]);
    /// ```
    pub fn as_str(self) -> &'a str {
        match self {
            Self::RootDir => "/",
            Self::CurDir => ".",
            Self::ParentDir => "..",
            Self::Normal(path) => path,
        }
    }

    /// Size of component in bytes
    pub fn len(&self) -> usize {
        self.as_str().len()
    }
}
