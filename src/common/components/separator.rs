use std::marker::PhantomData;

/// Interface providing logic to separate bytes
pub trait Separator: Sized {
    /// Returns the bytes representing the separator
    fn as_bytes() -> &'static [u8];

    /// Returns size of separator in bytes
    #[inline]
    fn len() -> usize {
        Self::as_bytes().len()
    }

    /// Finds position of next separator, traversing from the front of byte slice
    fn find(bytes: &[u8]) -> Option<usize> {
        let len = bytes.len();
        let sep_bytes = Self::as_bytes();
        let sep_len = sep_bytes.len();

        // Separator is bigger than the byte slice, so we'll never find it
        if sep_len == 0 || len == 0 || sep_len > len {
            return None;
        }

        // Check at each position for a match within the byte slice of the separator
        for i in 0..=(len - sep_len) {
            if &bytes[i..(i + sep_len)] == sep_bytes {
                return Some(i);
            }
        }

        None
    }

    /// Finds position of next separator, traversing from the back of byte slice
    fn rfind(bytes: &[u8]) -> Option<usize> {
        let len = bytes.len();
        let sep_bytes = Self::as_bytes();
        let sep_len = sep_bytes.len();

        // Separator is bigger than the byte slice, so we'll never find it
        if sep_len == 0 || len == 0 || sep_len > len {
            return None;
        }

        // Check at each position for a match within the byte slice of the separator
        for i in (0..=(len - sep_len)).rev() {
            if &bytes[i..(i + sep_len)] == sep_bytes {
                return Some(i);
            }
        }

        None
    }

    /// Returns true if the byte slice starts with the separator
    fn starts_with(bytes: &[u8]) -> bool {
        bytes.starts_with(Self::as_bytes())
    }

    /// Returns true if the byte slice ends with the separator
    fn ends_with(bytes: &[u8]) -> bool {
        bytes.ends_with(Self::as_bytes())
    }

    /// Returns an iterator over subslices separated by the separator
    fn split(bytes: &[u8]) -> SeparatorSplit<Self> {
        SeparatorSplit {
            _separator: PhantomData,
            inner: Some(bytes),
        }
    }

    /// Returns an iterator over subslices separated by the separator, starting from the end of the
    /// slice
    fn rsplit(bytes: &[u8]) -> SeparatorRSplit<Self> {
        SeparatorRSplit {
            _separator: PhantomData,
            inner: Some(bytes),
        }
    }

    /// Splits byte slice into two on either side of the next separator position from the front
    fn split_once(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
        if bytes.len() < Self::len() {
            return None;
        }

        Self::find(bytes).map(|i| (&bytes[..i], &bytes[i + Self::len()..]))
    }

    /// Splits byte slice into two on either side of the next separator position from the back
    fn rsplit_once(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
        if bytes.len() < Self::len() {
            return None;
        }

        Self::rfind(bytes).map(|i| (&bytes[..i], &bytes[i + Self::len()..]))
    }
}

/// An iterator over subslices separated by the separator
pub struct SeparatorSplit<'a, S>
where
    S: Separator,
{
    _separator: PhantomData<S>,
    inner: Option<&'a [u8]>,
}

impl<'a, S> Iterator for SeparatorSplit<'a, S>
where
    S: Separator,
{
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.take() {
            Some(bytes) => match S::split_once(bytes) {
                Some((item, remaining)) => {
                    self.inner = Some(remaining);
                    Some(item)
                }
                None => Some(bytes),
            },
            None => None,
        }
    }
}

/// An iterator over subslices separated by the separator, starting form the end of the slice
pub struct SeparatorRSplit<'a, S>
where
    S: Separator,
{
    _separator: PhantomData<S>,
    inner: Option<&'a [u8]>,
}

impl<'a, S> Iterator for SeparatorRSplit<'a, S>
where
    S: Separator,
{
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.take() {
            Some(bytes) => match S::rsplit_once(bytes) {
                Some((remaining, item)) => {
                    self.inner = Some(remaining);
                    Some(item)
                }
                None => Some(bytes),
            },
            None => None,
        }
    }
}

/// Implements [`Separator`] for a specific byte
pub struct ByteSeparator<const B: u8>;

impl<const B: u8> Separator for ByteSeparator<B> {
    fn as_bytes() -> &'static [u8] {
        &[B]
    }
}

/// Implements [`Separator`] for a specific character
pub struct CharSeparator<const C: char>;

impl<const C: char> Separator for CharSeparator<C> {
    fn as_bytes() -> &'static [u8] {
        &[C as u8]
    }
}
