mod components;

use core::fmt;
use core::hash::{Hash, Hasher};

pub use components::*;

use super::constants::*;
use crate::no_std_compat::*;
use crate::typed::{TypedPath, TypedPathBuf};
use crate::{private, Component, Components, Encoding, Path, PathBuf};

/// Represents a Windows-specific [`Path`]
pub type WindowsPath = Path<WindowsEncoding>;

/// Represents a Windows-specific [`PathBuf`]
pub type WindowsPathBuf = PathBuf<WindowsEncoding>;

/// Represents a Windows-specific [`Encoding`]
#[derive(Copy, Clone)]
pub struct WindowsEncoding;

impl private::Sealed for WindowsEncoding {}

impl<'a> Encoding<'a> for WindowsEncoding {
    type Components = WindowsComponents<'a>;

    fn label() -> &'static str {
        "windows"
    }

    fn components(path: &'a [u8]) -> Self::Components {
        WindowsComponents::new(path)
    }

    fn hash<H: Hasher>(path: &[u8], h: &mut H) {
        let (prefix_len, verbatim) = match Self::components(path).prefix() {
            Some(prefix) => {
                prefix.hash(h);
                (prefix.len(), prefix.kind().is_verbatim())
            }
            None => (0, false),
        };
        let bytes = &path[prefix_len..];

        let mut component_start = 0;
        let mut bytes_hashed = 0;

        for i in 0..bytes.len() {
            let is_sep = if verbatim {
                path[i] == SEPARATOR as u8
            } else {
                path[i] == SEPARATOR as u8 || path[i] == ALT_SEPARATOR as u8
            };
            if is_sep {
                if i > component_start {
                    let to_hash = &bytes[component_start..i];
                    h.write(to_hash);
                    bytes_hashed += to_hash.len();
                }

                // skip over separator and optionally a following CurDir item
                // since components() would normalize these away.
                component_start = i + 1;

                let tail = &bytes[component_start..];

                if !verbatim {
                    component_start += match tail {
                        [b'.'] => 1,
                        [b'.', sep, ..]
                            if *sep == SEPARATOR as u8 || *sep == ALT_SEPARATOR as u8 =>
                        {
                            1
                        }
                        _ => 0,
                    };
                }
            }
        }

        if component_start < bytes.len() {
            let to_hash = &bytes[component_start..];
            h.write(to_hash);
            bytes_hashed += to_hash.len();
        }

        h.write_usize(bytes_hashed);
    }

    // COMPLEX RULES OF WINDOWS PATH APPENDING
    //
    // 1. If the incoming path being pushed is absolute or has a prefix:
    //    * replace the current path with the incoming path
    //
    // 2. If the current path have a verbatim, verbatim disk, or verbatim UNC prefix
    //    and the incoming path being pushed is not empty:
    //    * we know that incoming path has NO prefix (checked @ #1)
    //    * build up the components representing our path (buffer)
    //        * start with all of the components from the current path (assign to buffer)
    //        * iterate through components of incoming path
    //        * if the incoming path has a root dir, remove everything after
    //          prefix in the current path (from buffer)
    //        * skip appending (to buffer) any current dir component from incoming path
    //        * if parent dir, check if the last component (in buffer) is normal, and if
    //          so pop it off (of buffer)
    //        * otherwise, push component (onto buffer)
    //    * iterate through buffer of components to rebuild Vec<u8> via loop:
    //        * assign flag (`need_sep`) to track if we need to add a separator
    //        * at beginning of loop, check if `need_sep` and component not root dir:
    //            * if so, push separator into Vec<u8>
    //        * push component into Vec<u8>
    //        * re-assign `need_sep` flag:
    //            * if component was root dir, flag is false
    //            * if component was prefix, flag is true IF not drive (Prefix::Disk)
    //            * else, flag is true
    //    * update inner pathbuf value to new Vec<u8>
    //
    // 3. If the incoming path being pushed has root dir ('\') and no prefix (checked @ #1):
    //    * we shorten current path to just the prefix, which can be 0 if there is no prefix
    //    * append incoming path to current path
    //
    // 4. Otherwise:
    //    * If we need a separator (none at the end and current is not empty) and the current
    //      bytes are not just a drive letter (e.g. C:), then append a separator to the end of
    //      current path
    //    * append incoming path to current path
    fn push(current_path: &mut Vec<u8>, path: &[u8]) {
        if path.is_empty() {
            return;
        }

        let comps = Self::components(path);
        let cur_comps = Self::components(current_path);

        if comps.is_absolute() || comps.has_prefix() {
            current_path.clear();
            current_path.extend_from_slice(path);
        } else if cur_comps.has_any_verbatim_prefix() && !path.is_empty() {
            let mut buffer: Vec<_> = Self::components(current_path).collect();
            for c in Self::components(path) {
                match c {
                    WindowsComponent::RootDir => {
                        buffer.truncate(1);
                        buffer.push(c);
                    }
                    WindowsComponent::CurDir => (),
                    WindowsComponent::ParentDir => {
                        if let Some(WindowsComponent::Normal(_)) = buffer.last() {
                            buffer.pop();
                        }
                    }
                    _ => buffer.push(c),
                }
            }

            let mut new_path = Vec::new();
            let mut need_sep = false;

            for c in buffer {
                if need_sep && c != WindowsComponent::RootDir {
                    new_path.push(SEPARATOR as u8);
                }

                new_path.extend_from_slice(c.as_bytes());

                need_sep = match c {
                    WindowsComponent::RootDir => false,
                    WindowsComponent::Prefix(prefix) => {
                        !matches!(prefix.kind(), WindowsPrefix::Disk(_))
                    }
                    _ => true,
                };
            }

            *current_path = new_path;
        } else if comps.has_root() {
            let len = Self::components(current_path).prefix_len();
            current_path.truncate(len);
            current_path.extend_from_slice(path);
        } else {
            // NOTE: From std lib, there's a check that the prefix len == path len, which
            //       would imply having no other
            let needs_sep = (!current_path.is_empty()
                && !current_path.ends_with(&[SEPARATOR as u8]))
                && !Self::components(current_path).is_only_disk();

            if needs_sep {
                current_path.push(SEPARATOR as u8);
            }

            current_path.extend_from_slice(path);
        }
    }
}

impl fmt::Debug for WindowsEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WindowsEncoding").finish()
    }
}

impl fmt::Display for WindowsEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WindowsEncoding")
    }
}

impl<T> Path<T>
where
    T: for<'enc> Encoding<'enc>,
{
    /// Returns true if the encoding for the path is for Windows.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::{UnixPath, WindowsPath};
    ///
    /// assert!(!UnixPath::new("/some/path").has_windows_encoding());
    /// assert!(WindowsPath::new(r"\some\path").has_windows_encoding());
    /// ```
    pub fn has_windows_encoding(&self) -> bool {
        T::label() == WindowsEncoding::label()
    }

    /// Creates an owned [`PathBuf`] like `self` but using [`WindowsEncoding`].
    ///
    /// See [`Path::with_encoding`] for more information.
    pub fn with_windows_encoding(&self) -> PathBuf<WindowsEncoding> {
        self.with_encoding()
    }
}

impl WindowsPath {
    pub fn to_typed_path(&self) -> TypedPath {
        TypedPath::windows(self)
    }

    pub fn to_typed_path_buf(&self) -> TypedPathBuf {
        TypedPathBuf::from_windows(self)
    }
}
