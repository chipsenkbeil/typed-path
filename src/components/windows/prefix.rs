use std::{
    cmp,
    hash::{Hash, Hasher},
};

/// Byte slice version of [`std::path::PrefixComponent`]
#[derive(Copy, Clone, Debug, Eq)]
pub struct PrefixComponent<'a> {
    /// The prefix as an unparsed `[u8]` slice
    raw: &'a [u8],

    /// The parsed prefix data
    parsed: Prefix<'a>,
}

impl<'a> PrefixComponent<'a> {
    /// Returns the parsed prefix data
    pub fn kind(&self) -> Prefix<'a> {
        self.parsed
    }

    /// Returns the raw [`[u8]`] slice for this prefix
    pub fn as_bytes(&self) -> &'a [u8] {
        self.raw
    }
}

impl<'a> cmp::PartialEq for PrefixComponent<'a> {
    #[inline]
    fn eq(&self, other: &PrefixComponent<'a>) -> bool {
        cmp::PartialEq::eq(&self.parsed, &other.parsed)
    }
}

impl<'a> cmp::PartialOrd for PrefixComponent<'a> {
    #[inline]
    fn partial_cmp(&self, other: &PrefixComponent<'a>) -> Option<cmp::Ordering> {
        cmp::PartialOrd::partial_cmp(&self.parsed, &other.parsed)
    }
}

impl cmp::Ord for PrefixComponent<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        cmp::Ord::cmp(&self.parsed, &other.parsed)
    }
}

impl Hash for PrefixComponent<'_> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.parsed.hash(h);
    }
}

/// Byte slice version of [`std::path::Prefix`]
#[derive(Copy, Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Prefix<'a> {
    Verbatim(&'a [u8]),
    VerbatimUNC(&'a [u8], &'a [u8]),
    VerbatimDisk(u8),
    DeviceNS(&'a [u8]),
    UNC(&'a [u8], &'a [u8]),
    Disk(u8),
}

impl<'a> Prefix<'a> {
    /// Calculates the full byte length of the prefix
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::Prefix::*;
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
    pub fn len(&self) -> usize {
        use self::Prefix::*;
        match *self {
            Verbatim(x) => 4 + x.len(),
            VerbatimUNC(x, y) => 8 + x.len() + if y.len() > 0 { 1 + y.len() } else { 0 },
            VerbatimDisk(_) => 6,
            UNC(x, y) => 2 + x.len() + if y.len() > 0 { 1 + y.len() } else { 0 },
            DeviceNS(x) => 4 + x.len(),
            Disk(_) => 2,
        }
    }

    /// Determines if the prefix is verbatim, i.e., begins with `\\?\`.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::Prefix::*;
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
        use self::Prefix::*;
        matches!(*self, Verbatim(_) | VerbatimDisk(_) | VerbatimUNC(..))
    }

    #[inline]
    fn is_drive(&self) -> bool {
        matches!(*self, Prefix::Disk(_))
    }

    #[inline]
    fn has_implicit_root(&self) -> bool {
        !self.is_drive()
    }
}
