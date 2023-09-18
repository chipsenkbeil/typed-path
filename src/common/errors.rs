use std::error::Error;
use std::fmt;

/// An error returned if the prefix was not found.
///
/// This `struct` is created by the [`strip_prefix`] method on [`Path`].
/// See its documentation for more.
///
/// [`Path`]: crate::Path
/// [`strip_prefix`]: crate::Path::strip_prefix
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StripPrefixError(pub(crate) ());

impl fmt::Display for StripPrefixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "prefix not found")
    }
}

impl Error for StripPrefixError {}
