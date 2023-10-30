#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ArgPredicate<'help> {
    IsPresent,
    Equals(&'help std::ffi::OsStr),
}

impl<'help> From<Option<&'help std::ffi::OsStr>> for ArgPredicate<'help> {
    fn from(other: Option<&'help std::ffi::OsStr>) -> Self {
        match other {
            Some(other) => Self::Equals(other),
            None => Self::IsPresent,
        }
    }
}
