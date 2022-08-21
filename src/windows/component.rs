mod prefix;
pub use prefix::{WindowsPrefix, WindowsPrefixComponent};

use crate::{
    windows::{CURRENT_DIR, PARENT_DIR, SEPARATOR_STR},
    Component,
};

/// Byte slice version of [`std::path::Component`] that represents a Windows-specific component
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum WindowsComponent<'a> {
    Prefix(WindowsPrefixComponent<'a>),
    RootDir,
    CurDir,
    ParentDir,
    Normal(&'a [u8]),
}

impl Component for WindowsComponent<'_> {
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
    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Prefix(p) => p.as_bytes(),
            Self::RootDir => SEPARATOR_STR.as_bytes(),
            Self::CurDir => CURRENT_DIR,
            Self::ParentDir => PARENT_DIR,
            Self::Normal(path) => path,
        }
    }

    /// Root is one of two situations
    ///
    /// * Is the root separator, e.g. `\windows`
    /// * Is a non-disk prefix, e.g. `\\server\share`
    fn is_root(&self) -> bool {
        matches!(self, Self::Prefix(_) | Self::RootDir)
    }

    fn is_normal(&self) -> bool {
        matches!(self, Self::Normal(_))
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
