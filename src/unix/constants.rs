/// The primary separator of path components for unix platforms
pub const SEPARATOR: char = '/';

/// The primary separator of path components for unix platforms
pub const SEPARATOR_STR: &str = "/";

/// Path component value that represents the parent directory
pub const PARENT_DIR: &[u8] = b"..";

/// Path component value that represents the current directory
pub const CURRENT_DIR: &[u8] = b".";

/// Bytes that are not allowed in file or directory names
pub const DISALLOWED_FILENAME_BYTES: [u8; 2] = [b'/', b'\0'];
