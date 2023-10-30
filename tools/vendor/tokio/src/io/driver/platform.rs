pub(crate) use self::sys::*;

#[cfg(unix)]
mod sys {
    use mio::unix::UnixReady;
    use mio::Ready;

    pub(crate) fn hup() -> Ready {
        UnixReady::hup().into()
    }

    pub(crate) fn is_hup(ready: Ready) -> bool {
        UnixReady::from(ready).is_hup()
    }

    pub(crate) fn error() -> Ready {
        UnixReady::error().into()
    }

    pub(crate) fn is_error(ready: Ready) -> bool {
        UnixReady::from(ready).is_error()
    }
}

#[cfg(windows)]
mod sys {
    use mio::Ready;

    pub(crate) fn hup() -> Ready {
        Ready::empty()
    }

    pub(crate) fn is_hup(_: Ready) -> bool {
        false
    }

    pub(crate) fn error() -> Ready {
        Ready::empty()
    }

    pub(crate) fn is_error(_: Ready) -> bool {
        false
    }
}
