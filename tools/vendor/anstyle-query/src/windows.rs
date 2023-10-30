#[cfg(windows)]
mod windows_console {
    use std::os::windows::io::AsRawHandle;
    use std::os::windows::io::RawHandle;

    use windows_sys::Win32::System::Console::CONSOLE_MODE;
    use windows_sys::Win32::System::Console::ENABLE_VIRTUAL_TERMINAL_PROCESSING;

    fn enable_vt(handle: RawHandle) -> std::io::Result<()> {
        unsafe {
            let handle = std::mem::transmute(handle);
            if handle == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "console is detached",
                ));
            }

            let mut dwmode: CONSOLE_MODE = 0;
            if windows_sys::Win32::System::Console::GetConsoleMode(handle, &mut dwmode) == 0 {
                return Err(std::io::Error::last_os_error());
            }

            dwmode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
            if windows_sys::Win32::System::Console::SetConsoleMode(handle, dwmode) == 0 {
                return Err(std::io::Error::last_os_error());
            }

            Ok(())
        }
    }

    fn enable_ansi_colors_raw() -> std::io::Result<()> {
        let stdout = std::io::stdout();
        let stdout_handle = stdout.as_raw_handle();
        let stderr = std::io::stderr();
        let stderr_handle = stderr.as_raw_handle();

        enable_vt(stdout_handle)?;
        if stdout_handle != stderr_handle {
            enable_vt(stderr_handle)?;
        }

        Ok(())
    }

    #[inline]
    pub fn enable_ansi_colors() -> Option<bool> {
        Some(enable_ansi_colors_raw().map(|_| true).unwrap_or(false))
    }
}

#[cfg(not(windows))]
mod windows_console {
    #[inline]
    pub fn enable_ansi_colors() -> Option<bool> {
        None
    }
}

pub use self::windows_console::enable_ansi_colors;
