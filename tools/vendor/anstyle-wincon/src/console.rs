/// Write colored text to the screen
#[derive(Debug)]
pub struct Console<S>
where
    S: crate::WinconStream + std::io::Write,
{
    stream: Option<S>,
    initial_fg: Option<anstyle::AnsiColor>,
    initial_bg: Option<anstyle::AnsiColor>,
    last_fg: Option<anstyle::AnsiColor>,
    last_bg: Option<anstyle::AnsiColor>,
}

impl<S> Console<S>
where
    S: crate::WinconStream + std::io::Write,
{
    pub fn new(stream: S) -> Result<Self, S> {
        // HACK: Assuming the error from `get_colors()` will be present on `write` and doing basic
        // ops on the stream will cause the same result
        let (initial_fg, initial_bg) = match stream.get_colors() {
            Ok(ok) => ok,
            Err(_) => {
                return Err(stream);
            }
        };
        Ok(Self {
            stream: Some(stream),
            initial_fg,
            initial_bg,
            last_fg: initial_fg,
            last_bg: initial_bg,
        })
    }

    /// Write colored text to the screen
    pub fn write(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
        data: &[u8],
    ) -> std::io::Result<usize> {
        self.apply(fg, bg)?;
        let written = self.as_stream_mut().write(data)?;
        Ok(written)
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.as_stream_mut().flush()
    }

    /// Change the terminal back to the initial colors
    pub fn reset(&mut self) -> std::io::Result<()> {
        self.apply(self.initial_fg, self.initial_bg)
    }

    /// Close the stream, reporting any errors
    pub fn close(mut self) -> std::io::Result<()> {
        self.reset()
    }

    /// Allow changing the stream
    pub fn map<S1: crate::WinconStream + std::io::Write>(
        mut self,
        op: impl FnOnce(S) -> S1,
    ) -> Console<S1> {
        Console {
            stream: Some(op(self.stream.take().unwrap())),
            initial_fg: self.initial_fg,
            initial_bg: self.initial_bg,
            last_fg: self.last_fg,
            last_bg: self.last_bg,
        }
    }

    /// Get the inner writer
    #[inline]
    pub fn into_inner(mut self) -> S {
        let _ = self.reset();
        self.stream.take().unwrap()
    }

    fn apply(
        &mut self,
        fg: Option<anstyle::AnsiColor>,
        bg: Option<anstyle::AnsiColor>,
    ) -> std::io::Result<()> {
        let fg = fg.or(self.initial_fg);
        let bg = bg.or(self.initial_bg);
        if fg == self.last_fg && bg == self.last_bg {
            return Ok(());
        }

        // Ensure everything is written with the last set of colors before applying the next set
        self.as_stream_mut().flush()?;

        self.as_stream_mut().set_colors(fg, bg)?;
        self.last_fg = fg;
        self.last_bg = bg;

        Ok(())
    }

    fn as_stream_mut(&mut self) -> &mut S {
        self.stream.as_mut().unwrap()
    }
}

impl<S> Console<S>
where
    S: crate::WinconStream + std::io::Write,
    S: crate::Lockable,
    <S as crate::Lockable>::Locked: crate::WinconStream + std::io::Write,
{
    /// Get exclusive access to the `Console`
    ///
    /// Why?
    /// - Faster performance when writing in a loop
    /// - Avoid other threads interleaving output with the current thread
    #[inline]
    pub fn lock(mut self) -> <Self as crate::Lockable>::Locked {
        Console {
            stream: Some(self.stream.take().unwrap().lock()),
            initial_fg: self.initial_fg,
            initial_bg: self.initial_bg,
            last_fg: self.last_fg,
            last_bg: self.last_bg,
        }
    }
}

impl<S> crate::Lockable for Console<S>
where
    S: crate::WinconStream + std::io::Write,
    S: crate::Lockable,
    <S as crate::Lockable>::Locked: crate::WinconStream + std::io::Write,
{
    type Locked = Console<<S as crate::Lockable>::Locked>;

    #[inline]
    fn lock(self) -> Self::Locked {
        self.lock()
    }
}

impl<S> Drop for Console<S>
where
    S: crate::WinconStream + std::io::Write,
{
    fn drop(&mut self) {
        // Otherwise `Console::lock` took it
        if self.stream.is_some() {
            let _ = self.reset();
        }
    }
}
