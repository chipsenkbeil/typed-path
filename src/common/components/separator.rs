use std::marker::PhantomData;

/// Interface providing logic to separate bytes
pub trait Separator: Sized {
    /// Finds position of next separator, traversing from the front of byte slice
    fn find(bytes: &[u8]) -> Option<usize>;

    /// Finds position of next separator, traversing from the back of byte slice
    fn rfind(bytes: &[u8]) -> Option<usize>;

    /// Returns size of separator in bytes
    fn len() -> usize;

    /// Returns an iterator over subslices separated by the separator
    fn split<'a>(bytes: &'a [u8]) -> SeparatorSplit<'a, Self> {
        SeparatorSplit {
            _separator: PhantomData,
            inner: Some(bytes),
        }
    }

    /// Returns an iterator over subslices separated by the separator, starting from the end of the
    /// slice
    fn rsplit<'a>(bytes: &'a [u8]) -> SeparatorRSplit<'a, Self> {
        SeparatorRSplit {
            _separator: PhantomData,
            inner: Some(bytes),
        }
    }

    /// Splits byte slice into two on either side of the next separator position from the front
    fn split_once<'a>(bytes: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
        match Self::find(bytes) {
            Some(i) => Some((&bytes[..i], &bytes[i + Self::len()..])),
            None => None,
        }
    }

    /// Splits byte slice into two on either side of the next separator position from the back
    fn rsplit_once<'a>(bytes: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
        match Self::rfind(bytes) {
            Some(i) => Some((&bytes[..i], &bytes[i + Self::len()..])),
            None => None,
        }
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

/// Implements [`Separator`] for a specific character
pub struct CharSeparator<const C: char>;

impl<const C: char> Separator for CharSeparator<C> {
    fn find(bytes: &[u8]) -> Option<usize> {
        bytes
            .into_iter()
            .enumerate()
            .find_map(|(i, b)| if *b == C as u8 { Some(i) } else { None })
    }

    fn rfind(bytes: &[u8]) -> Option<usize> {
        bytes
            .into_iter()
            .enumerate()
            .rev()
            .find_map(|(i, b)| if *b == C as u8 { Some(i) } else { None })
    }

    fn len() -> usize {
        1
    }
}
