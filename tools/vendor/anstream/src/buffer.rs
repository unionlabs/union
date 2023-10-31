/// In-memory [`RawStream`][crate::RawStream]
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Buffer(Vec<u8>);

impl Buffer {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for Buffer {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

#[cfg(feature = "auto")]
impl is_terminal::IsTerminal for Buffer {
    #[inline]
    fn is_terminal(&self) -> bool {
        false
    }
}

impl std::io::Write for Buffer {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.extend(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(all(windows, feature = "wincon"))]
impl anstyle_wincon::WinconStream for Buffer {
    fn set_colors(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
    ) -> std::io::Result<()> {
        use std::io::Write as _;

        if let Some(fg) = fg {
            write!(self, "{}", fg.render_fg())?;
        }
        if let Some(bg) = bg {
            write!(self, "{}", bg.render_bg())?;
        }
        if fg.is_none() && bg.is_none() {
            write!(self, "{}", anstyle::Reset.render())?;
        }
        Ok(())
    }

    fn get_colors(
        &self,
    ) -> std::io::Result<(Option<anstyle::AnsiColor>, Option<anstyle::AnsiColor>)> {
        Ok((None, None))
    }
}
