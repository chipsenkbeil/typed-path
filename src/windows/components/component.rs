mod prefix;
pub use prefix::{WindowsPrefix, WindowsPrefixComponent};

use crate::{
    private,
    windows::{WindowsComponents, CURRENT_DIR, PARENT_DIR, SEPARATOR_STR},
    Component, ParseError,
};
use std::convert::TryFrom;

/// Byte slice version of [`std::path::Component`] that represents a Windows-specific component
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum WindowsComponent<'a> {
    Prefix(WindowsPrefixComponent<'a>),
    RootDir,
    CurDir,
    ParentDir,
    Normal(&'a [u8]),
}

impl private::Sealed for WindowsComponent<'_> {}

impl<'a> WindowsComponent<'a> {
    /// Returns true if represents a prefix
    pub fn is_prefix(&self) -> bool {
        matches!(self, Self::Prefix(_))
    }

    /// Converts from `WindowsComponent` to [`Option<WindowsPrefixComponent>`]
    ///
    /// Converts `self` into an [`Option<WindowsPrefixComponent>`], consuming `self`, and
    /// discarding if not a [`WindowsPrefixComponent`]
    pub fn prefix(self) -> Option<WindowsPrefixComponent<'a>> {
        match self {
            Self::Prefix(p) => Some(p),
            _ => None,
        }
    }

    /// Converts from `WindowsComponent` to [`Option<WindowsPrefix>`]
    ///
    /// Converts `self` into an [`Option<WindowsPrefix>`], consuming `self`, and
    /// discarding if not a [`WindowsPrefixComponent`] whose [`kind`] method we invoke
    ///
    /// [`kind`]: WindowsPrefixComponent::kind
    pub fn prefix_kind(self) -> Option<WindowsPrefix<'a>> {
        self.prefix().map(|p| p.kind())
    }
}

impl<'a> Component<'a> for WindowsComponent<'a> {
    /// Extracts the underlying [`[u8]`] slice
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{Component, WindowsPath};
    ///
    /// let path = WindowsPath::new(br"C:\tmp\foo\..\bar.txt");
    /// let components: Vec<_> = path.components().map(|comp| comp.as_bytes()).collect();
    /// assert_eq!(&components, &[
    ///     b"C:".as_slice(),
    ///     br"\".as_slice(),
    ///     b"tmp".as_slice(),
    ///     b"foo".as_slice(),
    ///     b"..".as_slice(),
    ///     b"bar.txt".as_slice(),
    /// ]);
    /// ```
    fn as_bytes(&self) -> &'a [u8] {
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
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{Component, windows::WindowsComponent};
    /// use std::convert::TryFrom;
    ///
    /// let root_dir = WindowsComponent::try_from(br"\").unwrap();
    /// assert!(root_dir.is_root());
    ///
    /// let non_disk_prefix = WindowsComponent::try_from(br"\\?\pictures").unwrap();
    /// assert!(non_disk_prefix.is_root());
    ///
    /// let disk_prefix = WindowsComponent::try_from(b"C:").unwrap();
    /// assert!(!disk_prefix.is_root());
    ///
    /// let normal = WindowsComponent::try_from(b"file.txt").unwrap();
    /// assert!(!normal.is_root());
    /// ```
    fn is_root(&self) -> bool {
        match self {
            Self::RootDir => true,
            Self::Prefix(prefix) => !matches!(prefix.kind(), WindowsPrefix::Disk(_)),
            _ => false,
        }
    }

    /// Returns true if component is normal
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{Component, windows::WindowsComponent};
    /// use std::convert::TryFrom;
    ///
    /// let normal = WindowsComponent::try_from(b"file.txt").unwrap();
    /// assert!(normal.is_normal());
    ///
    /// let root_dir = WindowsComponent::try_from(br"\").unwrap();
    /// assert!(!root_dir.is_normal());
    /// ```
    fn is_normal(&self) -> bool {
        matches!(self, Self::Normal(_))
    }

    fn len(&self) -> usize {
        self.as_bytes().len()
    }
}

impl<'a> TryFrom<&'a [u8]> for WindowsComponent<'a> {
    type Error = ParseError;

    /// Parses the byte slice into a [`WindowsComponent`]
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::windows::{WindowsComponent, WindowsPrefix};
    /// use std::convert::TryFrom;
    ///
    /// // Supports parsing Windows prefixes
    /// let component = WindowsComponent::try_from(b"c:").unwrap();
    /// assert_eq!(component.prefix_kind(), Some(WindowsPrefix::Disk(b'c')));
    ///
    /// // Supports parsing standard windows path components
    /// assert_eq!(WindowsComponent::try_from(br"\"), Ok(WindowsComponent::RootDir));
    /// assert_eq!(WindowsComponent::try_from(b"."), Ok(WindowsComponent::CurDir));
    /// assert_eq!(WindowsComponent::try_from(b".."), Ok(WindowsComponent::ParentDir));
    /// assert_eq!(WindowsComponent::try_from(br"file.txt"), Ok(WindowsComponent::Normal(b"file.txt")));
    /// assert_eq!(WindowsComponent::try_from(br"dir\"), Ok(WindowsComponent::Normal(b"dir")));
    ///
    /// // Parsing more than one component will fail
    /// assert!(WindowsComponent::try_from(br"\file").is_err());
    /// ```
    fn try_from(path: &'a [u8]) -> Result<Self, Self::Error> {
        let mut components = WindowsComponents::new(path);

        let component = components.next().ok_or("no component found")?;
        if components.next().is_some() {
            return Err("found more than one component");
        }

        Ok(component)
    }
}

impl<'a, const N: usize> TryFrom<&'a [u8; N]> for WindowsComponent<'a> {
    type Error = ParseError;

    fn try_from(path: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from(path.as_slice())
    }
}

impl<'a> TryFrom<&'a str> for WindowsComponent<'a> {
    type Error = ParseError;

    fn try_from(path: &'a str) -> Result<Self, Self::Error> {
        Self::try_from(path.as_bytes())
    }
}
