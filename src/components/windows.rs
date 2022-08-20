mod prefix;
pub use prefix::*;

/// UTF-8 version of [`std::path::Component`] that represents a Windows-specific component
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum WindowsComponent<'a> {
    Prefix(PrefixComponent<'a>),
    RootDir,
    CurDir,
    ParentDir,
    Normal(&'a str),
}

impl<'a> WindowsComponent<'a> {
    /// Extracts the underlying [`str`] slice
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::WindowsPath;
    ///
    /// let path = WindowsPath::new(r"C:\tmp\.\foo\..\bar.txt");
    /// let components: Vec<_> = path.components().map(|comp| comp.as_str()).collect();
    /// assert_eq!(&components, &["C:", "tmp", ".", "foo", "..", "bar.txt"]);
    /// ```
    pub fn as_str(self) -> &'a str {
        match self {
            Self::Prefix(p) => p.as_str(),
            Self::RootDir => "\\",
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
