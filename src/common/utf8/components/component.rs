use crate::private;
use std::fmt;

/// Interface representing a component in a [`Utf8Path`]
///
/// [`Utf8Path`]: crate::Utf8Path
pub trait Utf8Component<'a>:
    AsRef<str> + Clone + fmt::Debug + PartialEq + Eq + PartialOrd + Ord + private::Sealed
{
    /// Extracts the underlying [`str`] slice
    fn as_str(&self) -> &'a str;

    /// Returns true if this component is the root component, meaning
    /// there are no more components before this one
    ///
    /// Use cases are for the root dir separator on Windows and Unix as
    /// well as Windows [`std::path::PrefixComponent`]
    ///
    /// # Examples
    ///
    /// `/my/../path/./here.txt` has the components on Unix of
    ///
    /// * `UnixComponent::RootDir` - `is_root() == true`
    /// * `UnixComponent::ParentDir` - `is_root() == false`
    /// * `UnixComponent::CurDir` - `is_root() == false`
    /// * `UnixComponent::Normal("here.txt")` - `is_root() == false`
    fn is_root(&self) -> bool;

    /// Returns true if this component represents a normal part of the path
    ///
    /// # Examples
    ///
    /// `/my/../path/./here.txt` has the components on Unix of
    ///
    /// * `UnixComponent::RootDir` - `is_normal() == false`
    /// * `UnixComponent::ParentDir` - `is_normal() == false`
    /// * `UnixComponent::CurDir` - `is_normal() == false`
    /// * `UnixComponent::Normal("here.txt")` - `is_normal() == true`
    fn is_normal(&self) -> bool;

    /// Returns size of component in bytes
    fn len(&self) -> usize;

    /// Returns true if component represents an empty str
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
