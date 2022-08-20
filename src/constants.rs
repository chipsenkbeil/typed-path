/// The primary separator of path components for windows platforms
pub const WINDOWS_SEPARATOR: char = '\\';

/// The primary separator of path components for windows platforms
pub const WINDOWS_SEPARATOR_STR: &str = "\\";

/// The primary separator of path components for windows platforms
pub const WINDOWS_SEPARATOR_BYTES: &[u8] = b"\\";

/// The primary separator of path components for unix platforms
pub const UNIX_SEPARATOR: char = '/';

/// The primary separator of path components for unix platforms
pub const UNIX_SEPARATOR_STR: &str = "/";

/// The primary separator of path components for unix platforms
pub const UNIX_SEPARATOR_BYTES: &[u8] = b"/";

/// Path component value that represents the parent directory
pub const PARENT_DIR_BYTES: &[u8] = b"..";

/// Path component value that represents the current directory
pub const CURRENT_DIR_BYTES: &[u8] = b".";
