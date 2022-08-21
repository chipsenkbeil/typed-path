use std::{
    cmp,
    hash::{Hash, Hasher},
};

/// Byte slice version of [`std::path::PrefixComponent`]
#[derive(Copy, Clone, Debug, Eq)]
pub struct WindowsPrefixComponent<'a> {
    /// The prefix as an unparsed `[u8]` slice
    pub(crate) raw: &'a [u8],

    /// The parsed prefix data
    pub(crate) parsed: WindowsPrefix<'a>,
}

impl<'a> WindowsPrefixComponent<'a> {
    /// Returns the parsed prefix data
    pub fn kind(&self) -> WindowsPrefix<'a> {
        self.parsed
    }

    /// Returns the raw [`[u8]`] slice for this prefix
    pub fn as_bytes(&self) -> &'a [u8] {
        self.raw
    }
}

impl<'a> cmp::PartialEq for WindowsPrefixComponent<'a> {
    #[inline]
    fn eq(&self, other: &WindowsPrefixComponent<'a>) -> bool {
        cmp::PartialEq::eq(&self.parsed, &other.parsed)
    }
}

impl<'a> cmp::PartialOrd for WindowsPrefixComponent<'a> {
    #[inline]
    fn partial_cmp(&self, other: &WindowsPrefixComponent<'a>) -> Option<cmp::Ordering> {
        cmp::PartialOrd::partial_cmp(&self.parsed, &other.parsed)
    }
}

impl cmp::Ord for WindowsPrefixComponent<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        cmp::Ord::cmp(&self.parsed, &other.parsed)
    }
}

impl Hash for WindowsPrefixComponent<'_> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.parsed.hash(h);
    }
}

/// Byte slice version of [`std::path::Prefix`]
#[derive(Copy, Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum WindowsPrefix<'a> {
    Verbatim(&'a [u8]),
    VerbatimUNC(&'a [u8], &'a [u8]),
    VerbatimDisk(u8),
    DeviceNS(&'a [u8]),
    UNC(&'a [u8], &'a [u8]),
    Disk(u8),
}

impl<'a> WindowsPrefix<'a> {
    /// Calculates the full byte length of the prefix
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::WindowsPrefix::*;
    ///
    /// // \\\\?\\pictures -> 12 bytes
    /// assert_eq!(Verbatim(b"pictures").len(), 12);
    ///
    /// // \\\\?\\UNC\\server -> 14 bytes
    /// assert_eq!(VerbatimUNC(b"server", b"").len(), 14);
    ///
    /// // \\\\?\\UNC\\server\\share -> 20 bytes
    /// assert_eq!(VerbatimUNC(b"server", b"share").len(), 20);
    ///
    /// // \\\\?\\c: -> 6 bytes
    /// assert_eq!(VerbatimDisk(b'C').len(), 6);
    ///
    /// // \\\\.\\BrainInterface -> 18 bytes
    /// assert_eq!(DeviceNS(b"BrainInterface").len(), 18);
    ///
    /// // \\\\server\\share -> 14 bytes
    /// assert_eq!(UNC(b"server", b"share").len(), 14);
    ///
    /// // C\: -> 2 bytes
    /// assert_eq!(Disk(b'C').len(), 2);
    /// ```
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        use self::WindowsPrefix::*;
        match *self {
            Verbatim(x) => 4 + x.len(),
            VerbatimUNC(x, y) => 8 + x.len() + if !y.is_empty() { 1 + y.len() } else { 0 },
            VerbatimDisk(_) => 6,
            UNC(x, y) => 2 + x.len() + if !y.is_empty() { 1 + y.len() } else { 0 },
            DeviceNS(x) => 4 + x.len(),
            Disk(_) => 2,
        }
    }

    /// Determines if the prefix is verbatim, i.e., begins with `\\?\`.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::WindowsPrefix::*;
    ///
    /// assert!(Verbatim(b"pictures").is_verbatim());
    /// assert!(VerbatimUNC(b"server", b"share").is_verbatim());
    /// assert!(VerbatimDisk(b'C').is_verbatim());
    /// assert!(!DeviceNS(b"BrainInterface").is_verbatim());
    /// assert!(!UNC(b"server", b"share").is_verbatim());
    /// assert!(!Disk(b'C').is_verbatim());
    /// ```
    #[inline]
    pub fn is_verbatim(&self) -> bool {
        use self::WindowsPrefix::*;
        matches!(*self, Verbatim(_) | VerbatimDisk(_) | VerbatimUNC(..))
    }

    #[inline]
    pub(crate) fn is_drive(&self) -> bool {
        matches!(*self, WindowsPrefix::Disk(_))
    }

    #[inline]
    pub(crate) fn has_implicit_root(&self) -> bool {
        !self.is_drive()
    }
}
