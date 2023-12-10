use crate::Component;
use crate::Encoding;
use crate::Path;
use crate::PathBuf;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct SingleComponentPathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    pub(crate) path: PathBuf<T>,
}

impl<T> SingleComponentPathBuf<T>
where
    T: for<'enc> Encoding<'enc>,
{
    pub fn new<S: Into<PathBuf<T>>>(component: S) -> Option<Self> {
        let component = Self {
            path: component.into(),
        };

        todo!();
        // component.is_valid().then_some(component)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(transparent)]
pub struct SingleComponentPath<T>
where
    T: for<'enc> Encoding<'enc>,
{
    pub(crate) path: Path<T>,
}

impl<T> SingleComponentPath<T>
where
    T: for<'enc> Encoding<'enc>,
{
    pub fn new<P: AsRef<Path<T>> + ?Sized>(component: &P) -> Option<&Self> {
        todo!();
        // let component = wrap_ref_path!(component.as_ref(), Self);

        // component.is_valid().then_some(component)
    }

    pub(crate) fn is_valid(&self) -> bool {
        let mut components = self
            .path
            .components()
            .filter(|component| !component.is_current());

        components
            .next()
            .map(|component| component.is_normal())
            .unwrap_or(false)
            && components.next().is_none()
    }
}
