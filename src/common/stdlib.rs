use crate::{Encoding, Path, PathBuf};
use std::{
    convert::TryFrom,
    ffi::OsStr,
    path::{
        Component as StdComponent, Components as StdComponents, Path as StdPath,
        PathBuf as StdPathBuf, Prefix as StdPrefix, PrefixComponent as StdPrefixComponent,
    },
};

impl<'a, T> TryFrom<&'a Path<T>> for &'a StdPath
where
    T: for<'enc> Encoding<'enc>,
{
    type Error = std::str::Utf8Error;

    fn try_from(path: &'a Path<T>) -> Result<Self, Self::Error> {
        Ok(StdPath::new(AsRef::<OsStr>::as_ref(path)))
    }
}
