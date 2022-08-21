use crate::{
    unix::{CURRENT_DIR, PARENT_DIR, SEPARATOR_STR},
    Component,
};

/// Byte slice version of [`std::path::Component`] that represents a Unix-specific component
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnixComponent<'a> {
    RootDir,
    CurDir,
    ParentDir,
    Normal(&'a [u8]),
}

impl<'a> Component<'a> for UnixComponent<'a> {
    /// Extracts the underlying [`[u8]`] slice
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::UnixPath;
    ///
    /// let path = UnixPath::new(b"/tmp/./foo/../bar.txt");
    /// let components: Vec<_> = path.components().map(|comp| comp.as_str()).collect();
    /// assert_eq!(&components, &[b"/", b"tmp", b".", b"foo", b".", b"bar.txt"]);
    /// ```
    fn as_bytes(&self) -> &'a [u8] {
        match self {
            Self::RootDir => SEPARATOR_STR.as_bytes(),
            Self::CurDir => CURRENT_DIR,
            Self::ParentDir => PARENT_DIR,
            Self::Normal(path) => path,
        }
    }

    fn is_root(&self) -> bool {
        matches!(self, Self::RootDir)
    }

    fn is_normal(&self) -> bool {
        matches!(self, Self::Normal(_))
    }

    fn len(&self) -> usize {
        self.as_bytes().len()
    }
}
