/// Interface providing logic to separate bytes
pub trait Separator {
    /// Finds position of next separator, traversing from the front of byte slice
    fn find(bytes: &[u8]) -> Option<usize>;

    /// Finds position of next separator, traversing from the back of byte slice
    fn rfind(bytes: &[u8]) -> Option<usize>;

    /// Returns size of separator in bytes
    fn len() -> usize;

    /// Splits byte slice into two on either side of the next separator position from the front
    fn split<'a>(bytes: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
        match Self::find(bytes) {
            Some(i) => Some(&bytes[..i], &bytes[i + Self::len()..]),
            None => None,
        }
    }

    /// Splits byte slice into two on either side of the next separator position from the back
    fn rsplit<'a>(bytes: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
        match Self::rfind(bytes) {
            Some(i) => Some(&bytes[..i], &bytes[i + Self::len()..]),
            None => None,
        }
    }
}

/// Implements [`Separator`] for a specific byte
pub struct ByteSeparator<const B: u8>;

impl<const B: u8> Separator for ByteSeparator<B> {
    fn find(bytes: &[u8]) -> Option<usize> {
        bytes.into_iter().enumerate().find_map(|(i, b)| b == B)
    }

    fn rfind(bytes: &[u8]) -> Option<usize> {
        bytes
            .into_iter()
            .enumerate()
            .rev()
            .find_map(|(i, b)| b == B)
    }

    fn len() -> usize {
        1
    }
}
