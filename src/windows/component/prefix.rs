use crate::{windows::parser, ParseError};
use std::{
    cmp,
    convert::TryFrom,
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
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::windows::WindowsPrefixComponent;
    /// use std::convert::TryFrom;
    ///
    /// // Disk will include the drive letter and :
    /// let component = WindowsPrefixComponent::try_from(b"C:").unwrap();
    /// assert_eq!(component.as_bytes(), b"C:");
    ///
    /// // UNC will include server & share
    /// let component = WindowsPrefixComponent::try_from(br"\\server\share").unwrap();
    /// assert_eq!(component.as_bytes(), br"\\server\share");
    ///
    /// // Device NS will include device
    /// let component = WindowsPrefixComponent::try_from(br"\\.\BrainInterface").unwrap();
    /// assert_eq!(component.as_bytes(), br"\\.\BrainInterface");
    ///
    /// // Verbatim will include component
    /// let component = WindowsPrefixComponent::try_from(br"\\?\pictures").unwrap();
    /// assert_eq!(component.as_bytes(), br"\\?\pictures");
    ///
    /// // Verbatim UNC will include server & share
    /// let component = WindowsPrefixComponent::try_from(br"\\?\UNC\server\share").unwrap();
    /// assert_eq!(component.as_bytes(), br"\\?\UNC\server\share");
    ///
    /// // Verbatim disk will include drive letter and :
    /// let component = WindowsPrefixComponent::try_from(br"\\?\C:").unwrap();
    /// assert_eq!(component.as_bytes(), br"\\?\C:");
    /// ```
    pub fn as_bytes(&self) -> &'a [u8] {
        self.raw
    }
}

impl<'a> TryFrom<&'a [u8]> for WindowsPrefixComponent<'a> {
    type Error = ParseError;

    /// Parses the byte slice into a [`WindowsPrefixComponent`]
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::windows::{WindowsPrefix, WindowsPrefixComponent};
    /// use std::convert::TryFrom;
    ///
    /// let component = WindowsPrefixComponent::try_from(b"C:").unwrap();
    /// assert_eq!(component.kind(), WindowsPrefix::Disk(b'C'));
    ///
    /// let component = WindowsPrefixComponent::try_from(br"\\.\BrainInterface").unwrap();
    /// assert_eq!(component.kind(), WindowsPrefix::DeviceNS(b"BrainInterface"));
    ///
    /// let component = WindowsPrefixComponent::try_from(br"\\server\share").unwrap();
    /// assert_eq!(component.kind(), WindowsPrefix::UNC(b"server", b"share"));
    ///
    /// let component = WindowsPrefixComponent::try_from(br"\\?\UNC\server\share").unwrap();
    /// assert_eq!(component.kind(), WindowsPrefix::VerbatimUNC(b"server", b"share"));
    ///
    /// let component = WindowsPrefixComponent::try_from(br"\\?\C:").unwrap();
    /// assert_eq!(component.kind(), WindowsPrefix::VerbatimDisk(b'C'));
    ///
    /// let component = WindowsPrefixComponent::try_from(br"\\?\pictures").unwrap();
    /// assert_eq!(component.kind(), WindowsPrefix::Verbatim(b"pictures"));
    ///
    /// // Parsing something that is not a prefix will fail
    /// assert!(WindowsPrefixComponent::try_from(b"hello").is_err());
    ///
    /// // Parsing more than a prefix will fail
    /// assert!(WindowsPrefixComponent::try_from(br"C:\path").is_err());
    /// ```
    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        let (remaining, prefix) = parser::prefix_component(bytes)?;

        if !remaining.is_empty() {
            return Err("contains more than prefix");
        }

        Ok(prefix)
    }
}

impl<'a, const N: usize> TryFrom<&'a [u8; N]> for WindowsPrefixComponent<'a> {
    type Error = ParseError;

    fn try_from(bytes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from(bytes.as_slice())
    }
}

impl<'a> TryFrom<&'a str> for WindowsPrefixComponent<'a> {
    type Error = ParseError;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::try_from(s.as_bytes())
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

impl<'a> TryFrom<&'a [u8]> for WindowsPrefix<'a> {
    type Error = ParseError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(WindowsPrefixComponent::try_from(bytes)?.kind())
    }
}

impl<'a, const N: usize> TryFrom<&'a [u8; N]> for WindowsPrefix<'a> {
    type Error = ParseError;

    fn try_from(bytes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from(bytes.as_slice())
    }
}

impl<'a> TryFrom<&'a str> for WindowsPrefix<'a> {
    type Error = ParseError;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::try_from(s.as_bytes())
    }
}

impl<'a> WindowsPrefix<'a> {
    /// Calculates the full byte length of the prefix
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::windows::WindowsPrefix::*;
    ///
    /// // \\?\pictures -> 12 bytes
    /// assert_eq!(Verbatim(b"pictures").len(), 12);
    ///
    /// // \\?\UNC\server -> 14 bytes
    /// assert_eq!(VerbatimUNC(b"server", b"").len(), 14);
    ///
    /// // \\?\UNC\server\share -> 20 bytes
    /// assert_eq!(VerbatimUNC(b"server", b"share").len(), 20);
    ///
    /// // \\?\c: -> 6 bytes
    /// assert_eq!(VerbatimDisk(b'C').len(), 6);
    ///
    /// // \\.\BrainInterface -> 18 bytes
    /// assert_eq!(DeviceNS(b"BrainInterface").len(), 18);
    ///
    /// // \\server\share -> 14 bytes
    /// assert_eq!(UNC(b"server", b"share").len(), 14);
    ///
    /// // C: -> 2 bytes
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
    /// use typed_path::windows::WindowsPrefix::*;
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
}
