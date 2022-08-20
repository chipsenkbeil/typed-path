use crate::{CURRENT_DIR_BYTES, PARENT_DIR_BYTES, UNIX_SEPARATOR_BYTES};

/// Version of [`std::path::Component`] that represents a Unix-specific component
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnixComponent<'a> {
    RootDir,
    CurDir,
    ParentDir,
    Normal(&'a [u8]),
}

impl<'a> UnixComponent<'a> {
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
    pub fn as_bytes(self) -> &'a [u8] {
        match self {
            Self::RootDir => UNIX_SEPARATOR_BYTES,
            Self::CurDir => CURRENT_DIR_BYTES,
            Self::ParentDir => PARENT_DIR_BYTES,
            Self::Normal(path) => path,
        }
    }

    /// Size of component in bytes
    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

    /// Returns true only when the component is a normal path, but the path is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
