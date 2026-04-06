pub(crate) mod constants;
mod non_utf8;
mod utf8;

pub use non_utf8::*;
pub use utf8::*;

#[cfg(feature = "std")]
#[test]
fn absolutize_should_fail_for_windows_path_with_drive_letter_only_prefix() {
    let p = WindowsPath::new(r"C:something");
    assert!(p.absolutize().is_err())
}

#[cfg(feature = "std")]
#[test]
fn absolutize_should_fail_for_windows_path_with_verbatim_drive_letter_only_prefix() {
    let p = WindowsPath::new(r"\\?\C:something");
    assert!(p.absolutize().is_err())
}

#[cfg(feature = "std")]
#[test]
fn absolutize_should_fail_for_windows_path_with_unc_server_only_prefix() {
    let p = WindowsPath::new(r"\\server");
    assert!(p.absolutize().is_err())
}

#[cfg(feature = "std")]
#[test]
fn absolutize_should_fail_for_windows_path_with_verbatim_unc_server_only_prefix() {
    let p = WindowsPath::new(r"\\?\UNC\server");
    assert!(p.absolutize().is_err())
}
