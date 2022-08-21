use std::{error::Error, fmt};

/// An error returned from [`Path::strip_prefix`] if the prefix was not found.
///
/// This `struct` is created by the [`strip_prefix`] method on [`Path`].
/// See its documentation for more.
///
/// [`strip_prefix`]: Path::strip_prefix
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StripPrefixError(pub(crate) ());

impl fmt::Display for StripPrefixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "prefix not found")
    }
}

impl Error for StripPrefixError {}
