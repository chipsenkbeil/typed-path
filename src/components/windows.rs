mod prefix;
pub use prefix::*;

use crate::{ByteComponent, CURRENT_DIR_BYTES, PARENT_DIR_BYTES, WINDOWS_SEPARATOR_BYTES};

/// UTF-8 version of [`std::path::Component`] that represents a Windows-specific component
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum WindowsComponent<'a> {
    Prefix(PrefixComponent<'a>),
    RootDir,
    CurDir,
    ParentDir,
    Normal(&'a [u8]),
}

impl<'a> ByteComponent<'a> for WindowsComponent<'a> {
    /// Extracts the underlying [`OsStr`] slice
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::WindowsPath;
    ///
    /// let path = WindowsPath::new(br"C:\tmp\.\foo\..\bar.txt");
    /// let components: Vec<_> = path.components().map(|comp| comp.as_os_str()).collect();
    /// assert_eq!(&components, &[b"C:", b"tmp", b".", b"foo", b"..", b"bar.txt"]);
    /// ```
    fn as_bytes(&self) -> &'a [u8] {
        match self {
            Self::Prefix(p) => p.as_bytes(),
            Self::RootDir => WINDOWS_SEPARATOR_BYTES,
            Self::CurDir => CURRENT_DIR_BYTES,
            Self::ParentDir => PARENT_DIR_BYTES,
            Self::Normal(path) => path,
        }
    }

    /// Size of component in bytes
    fn len(&self) -> usize {
        self.as_bytes().len()
    }

    /// Returns true only when the component is a normal path, but the path is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
