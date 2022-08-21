mod component;
mod constants;
mod parser;

pub use component::*;
pub use constants::*;

use crate::{CharSeparator, Components, Encoding, Path, PathBuf};

/// Represents a Windows-specific [`Path`]
pub type WindowsPath = Path<WindowsEncoding>;

/// Represents a Windows-specific [`PathBuf`]
pub type WindowsPathBuf = PathBuf<WindowsEncoding>;

/// Represents a Windows-specific [`Components`]
pub type WindowsComponents<'a> = Components<'a, WindowsEncoding>;

/// Represents a Windows-specific [`Encoding`]
#[derive(Copy, Clone)]
pub struct WindowsEncoding;

impl<'a> Encoding<'a> for WindowsEncoding {
    type Component = WindowsComponent<'a>;
    type Separator = CharSeparator<SEPARATOR>;

    fn components(bytes: &'a [u8]) -> Components<'a, Self> {
        parser::parse(bytes).expect("TODO: Fix this panic")
    }

    fn is_absolute(bytes: &'a [u8]) -> bool {
        let mut components = Self::components(bytes);

        matches!(
            (components.next(), components.next()),
            (
                Some(WindowsComponent::Prefix(_)),
                Some(WindowsComponent::RootDir)
            )
        )
    }

    fn has_root(bytes: &'a [u8]) -> bool {
        let mut components = Self::components(bytes);

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
