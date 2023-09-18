use crate::unix::{UnixComponents, CURRENT_DIR, PARENT_DIR, SEPARATOR_STR};
use crate::{private, Component, Encoding, ParseError, Path};

/// Byte slice version of [`std::path::Component`] that represents a Unix-specific component
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnixComponent<'a> {
    RootDir,
    CurDir,
    ParentDir,
    Normal(&'a [u8]),
}

impl private::Sealed for UnixComponent<'_> {}

impl<'a> UnixComponent<'a> {
    /// Returns path representing this specific component
    pub fn as_path<T>(&self) -> &Path<T>
    where
        T: for<'enc> Encoding<'enc>,
    {
        Path::new(self.as_bytes())
    }
}

impl<'a> Component<'a> for UnixComponent<'a> {
    /// Extracts the underlying [`[u8]`] slice
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{Component, UnixPath};
    ///
    /// let path = UnixPath::new(b"/tmp/foo/../bar.txt");
    /// let components: Vec<_> = path.components().map(|comp| comp.as_bytes()).collect();
    /// assert_eq!(&components, &[
    ///     b"/".as_slice(),
    ///     b"tmp".as_slice(),
    ///     b"foo".as_slice(),
    ///     b"..".as_slice(),
    ///     b"bar.txt".as_slice(),
    /// ]);
    /// ```
    fn as_bytes(&self) -> &'a [u8] {
        match self {
            Self::RootDir => SEPARATOR_STR.as_bytes(),
            Self::CurDir => CURRENT_DIR,
            Self::ParentDir => PARENT_DIR,
            Self::Normal(path) => path,
        }
    }

    /// Returns true if is the root dir component
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{Component, unix::UnixComponent};
    /// use std::convert::TryFrom;
    ///
    /// let root_dir = UnixComponent::try_from(b"/").unwrap();
    /// assert!(root_dir.is_root());
    ///
    /// let normal = UnixComponent::try_from(b"file.txt").unwrap();
    /// assert!(!normal.is_root());
    /// ```
    fn is_root(&self) -> bool {
        matches!(self, Self::RootDir)
    }

    /// Returns true if is a normal component
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{Component, unix::UnixComponent};
    /// use std::convert::TryFrom;
    ///
    /// let normal = UnixComponent::try_from(b"file.txt").unwrap();
    /// assert!(normal.is_normal());
    ///
    /// let root_dir = UnixComponent::try_from(b"/").unwrap();
    /// assert!(!root_dir.is_normal());
    /// ```
    fn is_normal(&self) -> bool {
        matches!(self, Self::Normal(_))
    }

    /// Returns true if is a parent directory component
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{Component, unix::UnixComponent};
    /// use std::convert::TryFrom;
    ///
    /// let parent = UnixComponent::try_from("..").unwrap();
    /// assert!(parent.is_parent());
    ///
    /// let root_dir = UnixComponent::try_from("/").unwrap();
    /// assert!(!root_dir.is_parent());
    /// ```
    fn is_parent(&self) -> bool {
        matches!(self, Self::ParentDir)
    }

    /// Returns true if is the current directory component
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{Component, unix::UnixComponent};
    /// use std::convert::TryFrom;
    ///
    /// let current = UnixComponent::try_from(".").unwrap();
    /// assert!(current.is_current());
    ///
    /// let root_dir = UnixComponent::try_from("/").unwrap();
    /// assert!(!root_dir.is_current());
    /// ```
    fn is_current(&self) -> bool {
        matches!(self, Self::CurDir)
    }

    fn len(&self) -> usize {
        self.as_bytes().len()
    }

    /// Returns the root directory component.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{Component, unix::UnixComponent};
    ///
    /// assert_eq!(UnixComponent::root(), UnixComponent::RootDir);
    /// ```
    fn root() -> Self {
        Self::RootDir
    }

    /// Returns the parent directory component.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{Component, unix::UnixComponent};
    ///
    /// assert_eq!(UnixComponent::parent(), UnixComponent::ParentDir);
    /// ```
    fn parent() -> Self {
        Self::ParentDir
    }

    /// Returns the current directory component.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{Component, unix::UnixComponent};
    ///
    /// assert_eq!(UnixComponent::current(), UnixComponent::CurDir);
    /// ```
    fn current() -> Self {
        Self::CurDir
    }
}

impl AsRef<[u8]> for UnixComponent<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<T> AsRef<Path<T>> for UnixComponent<'_>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        Path::new(self.as_bytes())
    }
}

impl<'a> TryFrom<&'a [u8]> for UnixComponent<'a> {
    type Error = ParseError;

    /// Parses the byte slice into a [`UnixComponent`]
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::unix::UnixComponent;
    /// use std::convert::TryFrom;
    ///
    /// // Supports parsing standard unix path components
    /// assert_eq!(UnixComponent::try_from(b"/"), Ok(UnixComponent::RootDir));
    /// assert_eq!(UnixComponent::try_from(b"."), Ok(UnixComponent::CurDir));
    /// assert_eq!(UnixComponent::try_from(b".."), Ok(UnixComponent::ParentDir));
    /// assert_eq!(UnixComponent::try_from(b"file.txt"), Ok(UnixComponent::Normal(b"file.txt")));
    /// assert_eq!(UnixComponent::try_from(b"dir/"), Ok(UnixComponent::Normal(b"dir")));
    ///
    /// // Parsing more than one component will fail
    /// assert!(UnixComponent::try_from(b"/file").is_err());
    /// ```
    fn try_from(path: &'a [u8]) -> Result<Self, Self::Error> {
        let mut components = UnixComponents::new(path);

        let component = components.next().ok_or("no component found")?;
        if components.next().is_some() {
            return Err("found more than one component");
        }

        Ok(component)
    }
}

impl<'a, const N: usize> TryFrom<&'a [u8; N]> for UnixComponent<'a> {
    type Error = ParseError;

    fn try_from(path: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from(path.as_slice())
    }
}

impl<'a> TryFrom<&'a str> for UnixComponent<'a> {
    type Error = ParseError;

    fn try_from(path: &'a str) -> Result<Self, Self::Error> {
        Self::try_from(path.as_bytes())
    }
}
