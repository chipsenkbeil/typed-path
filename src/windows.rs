mod component;
mod constants;
mod parser;

pub use component::*;
pub use constants::*;

use crate::{private, CharSeparator, Component, Components, Encoding, Path, PathBuf, Separator};

/// Represents a Windows-specific [`Path`]
pub type WindowsPath = Path<WindowsEncoding>;

/// Represents a Windows-specific [`PathBuf`]
pub type WindowsPathBuf = PathBuf<WindowsEncoding>;

/// Represents a Windows-specific [`Components`]
pub type WindowsComponents<'a> = Components<'a, WindowsEncoding>;

impl<'a> WindowsComponents<'a> {
    /// Returns true if the represented path has a prefix
    #[inline]
    pub fn has_prefix(&self) -> bool {
        self.prefix().is_some()
    }

    /// Returns the prefix of the represented path's components if it has one
    pub fn prefix(&self) -> Option<WindowsPrefixComponent> {
        match self.components.front() {
            Some(WindowsComponent::Prefix(p)) => Some(*p),
            _ => None,
        }
    }

    #[inline]
    fn prefix_len(&self) -> usize {
        self.prefix().map(|p| p.as_bytes().len()).unwrap_or(0)
    }

    /// Returns the kind of prefix associated with the represented path if it has one
    #[inline]
    pub fn prefix_kind(&self) -> Option<WindowsPrefix> {
        self.prefix().map(|p| p.kind())
    }

    /// Returns true if represented path has a verbatim, verbatim UNC, or verbatim disk prefix
    pub fn has_any_verbatim_prefix(&self) -> bool {
        matches!(
            self.prefix_kind(),
            Some(WindowsPrefix::Verbatim(_) | WindowsPrefix::UNC(..) | WindowsPrefix::Disk(_))
        )
    }

    /// Returns true if represented path has a verbatim prefix (e.g. `\\?\pictures)
    pub fn has_verbatim_prefix(&self) -> bool {
        matches!(self.prefix_kind(), Some(WindowsPrefix::Verbatim(_)))
    }

    /// Returns true if represented path has a verbatim UNC prefix (e.g. `\\?\UNC\server\share`)
    pub fn has_verbatim_unc_prefix(&self) -> bool {
        matches!(self.prefix_kind(), Some(WindowsPrefix::VerbatimUNC(..)))
    }

    /// Returns true if represented path has a verbatim disk prefix (e.g. `\\?\C:`)
    pub fn has_verbatim_disk_prefix(&self) -> bool {
        matches!(self.prefix_kind(), Some(WindowsPrefix::VerbatimDisk(_)))
    }

    /// Returns true if represented path has a device NS prefix (e.g. `\\.\BrainInterface`)
    pub fn has_device_ns_prefix(&self) -> bool {
        matches!(self.prefix_kind(), Some(WindowsPrefix::DeviceNS(_)))
    }

    /// Returns true if represented path has a UNC prefix (e.g. `\\server\share`)
    pub fn has_unc_prefix(&self) -> bool {
        matches!(self.prefix_kind(), Some(WindowsPrefix::UNC(..)))
    }

    /// Returns true if represented path has a disk prefix (e.g. `C:`)
    pub fn has_disk_prefix(&self) -> bool {
        matches!(self.prefix_kind(), Some(WindowsPrefix::Disk(_)))
    }

    /// Returns true if there is a separator immediately after the prefix, or separator
    /// starts the components if there is no prefix
    ///
    /// e.g. `C:\` and `\path` would return true whereas `\\?\path` would return false
    pub fn has_physical_root(&self) -> bool {
        match self.components.front() {
            Some(WindowsComponent::RootDir) => true,
            Some(WindowsComponent::Prefix(_)) if self.components.len() > 1 => {
                matches!(self.components[1], WindowsComponent::RootDir)
            }
            _ => false,
        }
    }

    /// Returns true if there is a root separator without a [`WindowsComponent::RootDir`]
    /// needing to be present. This is tied to prefixes like verbatim `\\?\` and UNC `\\`.
    ///
    /// Really, it's everything but a disk prefix of `C:` that provide an implicit root
    pub fn has_implicit_root(&self) -> bool {
        match self.prefix().map(|p| p.kind()) {
            Some(WindowsPrefix::Disk(_)) | None => false,
            Some(_) => true,
        }
    }

    /// Returns true if just a disk, e.g. `C:`
    fn is_only_disk(&self) -> bool {
        self.components.len() == 1 && self.has_disk_prefix()
    }
}

/// Represents a Windows-specific [`Encoding`]
#[derive(Copy, Clone)]
pub struct WindowsEncoding;

impl private::Sealed for WindowsEncoding {}

impl<'a> Encoding<'a> for WindowsEncoding {
    type Component = WindowsComponent<'a>;
    type Separator = CharSeparator<SEPARATOR>;

    fn push(current_path: &mut Vec<u8>, path: &[u8]) {
        if path.is_empty() {
            return;
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
        if Self::is_absolute(path) || Self::components(path).has_prefix() {
            current_path.clear();
            current_path.extend_from_slice(path);
        } else if Self::components(current_path).has_any_verbatim_prefix() && !path.is_empty() {
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
                    new_path.extend_from_slice(Self::Separator::as_primary_bytes());
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
        } else if Self::has_root(path) {
            let len = Self::components(current_path).prefix_len();
            current_path.truncate(len);
            current_path.extend_from_slice(path);
        } else {
            // NOTE: From std lib, there's a check that the prefix len == path len, which
            //       would imply having no other
            let needs_sep = (!current_path.is_empty()
                && !Self::Separator::is_at_end_of(current_path))
                && !Self::components(current_path).is_only_disk();

            if needs_sep {
                current_path.extend_from_slice(Self::Separator::as_primary_bytes());
            }

            current_path.extend_from_slice(path);
        }
    }

    fn components(path: &'a [u8]) -> Components<'a, Self> {
        parser::parse(path).expect("TODO: Fix this panic")
    }

    /// Returns true only if
    fn is_absolute(path: &[u8]) -> bool {
        let mut components = Self::components(path);

        matches!(
            (components.next(), components.next()),
            (
                Some(WindowsComponent::Prefix(_)),
                Some(WindowsComponent::RootDir)
            )
        )
    }

    /// Returns true if the `path` has either:
    ///
    /// * physical root, meaning it begins with the separator (e.g. `\my\path`)
    /// * implicit root, meaning it begins with a prefix that is not a drive (e.g. `\\?\pictures`)
    fn has_root(path: &[u8]) -> bool {
        let mut components = Self::components(path);

        match components.next() {
            Some(WindowsComponent::RootDir) => true,
            Some(WindowsComponent::Prefix(p)) => match p.kind() {
                WindowsPrefix::Disk(_) | WindowsPrefix::VerbatimDisk(_) => {
                    matches!(components.next(), Some(WindowsComponent::RootDir))
                }
                _ => true,
            },
            _ => false,
        }
    }
}
