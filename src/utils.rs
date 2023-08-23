use crate::{NativePathBuf, Utf8NativePathBuf};

use std::convert::TryFrom;
use std::{env, io};

/// Returns the current working directory as [`NativePathBuf`].
///
/// # Errors
///
/// Returns an [`Err`] if the current working directory value is invalid
/// or unable to parse the directory with the native encoding.
///
/// Possible cases:
///
/// * Current directory does not exist.
/// * There are insufficient permissions to access the current directory.
/// * The encoding used to parse the current directory failed to parse.
///
/// # Examples
///
/// ```
/// fn main() -> std::io::Result<()> {
///     let path = typed_path::utils::current_dir()?;
///     println!("The current directory is {}", path.display());
///     Ok(())
/// }
/// ```
pub fn current_dir() -> io::Result<NativePathBuf> {
    let std_path = env::current_dir()?;
    match NativePathBuf::try_from(std_path) {
        Ok(path) => Ok(path),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "wrong encoding")),
    }
}

/// Returns the current working directory as [`Utf8NativePathBuf`].
///
/// # Errors
///
/// Returns an [`Err`] if the current working directory value is invalid
/// or unable to parse the directory with the native encoding.
///
/// Possible cases:
///
/// * Current directory does not exist.
/// * There are insufficient permissions to access the current directory.
/// * The encoding used to parse the current directory failed to parse.
/// * The current directory was not valid UTF8.
///
/// # Examples
///
/// ```
/// fn main() -> std::io::Result<()> {
///     let path = typed_path::utils::utf8_current_dir()?;
///     println!("The current directory is {}", path);
///     Ok(())
/// }
/// ```
pub fn utf8_current_dir() -> io::Result<Utf8NativePathBuf> {
    match Utf8NativePathBuf::from_bytes_path_buf(current_dir()?) {
        Ok(path) => Ok(path),
        Err(x) => Err(io::Error::new(io::ErrorKind::InvalidData, x)),
    }
}
