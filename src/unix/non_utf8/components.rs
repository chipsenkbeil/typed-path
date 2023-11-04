mod component;
mod parser;

use core::{cmp, fmt, iter};

pub use component::*;
use parser::Parser;

use crate::{private, Components, Encoding, Path};

#[derive(Clone)]
pub struct UnixComponents<'a> {
    parser: Parser<'a>,
}

impl<'a> UnixComponents<'a> {
    pub(crate) fn new(path: &'a [u8]) -> Self {
        Self {
            parser: Parser::new(path),
        }
    }

    /// Extracts a slice corresponding to the portion of the path remaining for iteration.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{Path, UnixEncoding};
    ///
    /// // NOTE: A path cannot be created on its own without a defined encoding
    /// let mut components = Path::<UnixEncoding>::new("/tmp/foo/bar.txt").components();
    /// components.next();
    /// components.next();
    ///
    /// assert_eq!(Path::<UnixEncoding>::new("foo/bar.txt"), components.as_path());
    /// ```
    pub fn as_path<T>(&self) -> &'a Path<T>
    where
        T: for<'enc> Encoding<'enc>,
    {
        Path::new(self.parser.remaining())
    }
}

impl private::Sealed for UnixComponents<'_> {}

impl<'a> Components<'a> for UnixComponents<'a> {
    type Component = UnixComponent<'a>;

    fn as_bytes(&self) -> &'a [u8] {
        self.parser.remaining()
    }

    fn is_absolute(&self) -> bool {
        self.has_root()
    }

    fn has_root(&self) -> bool {
        // Create a copy of our parser so we don't mutate state
        let mut parser = self.parser.clone();

        matches!(parser.next_front(), Ok(UnixComponent::RootDir))
    }
}

impl AsRef<[u8]> for UnixComponents<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<T> AsRef<Path<T>> for UnixComponents<'_>
where
    T: for<'enc> Encoding<'enc>,
{
    #[inline]
    fn as_ref(&self) -> &Path<T> {
        Path::new(self.as_bytes())
    }
}

impl<'a> fmt::Debug for UnixComponents<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a>(UnixComponents<'a>);

        impl<'a> fmt::Debug for DebugHelper<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.clone()).finish()
            }
        }

        f.debug_tuple("WindowsComponents")
            .field(&DebugHelper(self.clone()))
            .finish()
    }
}

impl<'a> Iterator for UnixComponents<'a> {
    type Item = <Self as Components<'a>>::Component;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next_front().ok()
    }
}

impl<'a> DoubleEndedIterator for UnixComponents<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.parser.next_back().ok()
    }
}

impl<'a> iter::FusedIterator for UnixComponents<'a> {}

impl<'a> cmp::PartialEq for UnixComponents<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let _self = Self::new(self.parser.remaining());
        let _other = Self::new(other.parser.remaining());

        _self.eq(_other)
    }
}

impl<'a> cmp::Eq for UnixComponents<'a> {}

impl<'a> cmp::PartialOrd for UnixComponents<'a> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> cmp::Ord for UnixComponents<'a> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let _self = Self::new(self.parser.remaining());
        let _other = Self::new(other.parser.remaining());

        _self.cmp(_other)
    }
}
