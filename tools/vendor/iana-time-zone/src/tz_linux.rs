use std::fs::{read_link, read_to_string};

pub(crate) fn get_timezone_inner() -> Result<String, crate::GetTimezoneError> {
    etc_localtime().or_else(|_| etc_timezone())
}

fn etc_timezone() -> Result<String, crate::GetTimezoneError> {
    // see https://stackoverflow.com/a/12523283
    let mut contents = read_to_string("/etc/timezone")?;
    // Trim to the correct length without allocating.
    contents.truncate(contents.trim_end().len());
    Ok(contents)
}

fn etc_localtime() -> Result<String, crate::GetTimezoneError> {
    // Per <https://www.man7.org/linux/man-pages/man5/localtime.5.html>:
    // “ The /etc/localtime file configures the system-wide timezone of the local system that is
    //   used by applications for presentation to the user. It should be an absolute or relative
    //   symbolic link pointing to /usr/share/zoneinfo/, followed by a timezone identifier such as
    //   "Europe/Berlin" or "Etc/UTC". The resulting link should lead to the corresponding binary
    //   tzfile(5) timezone data for the configured timezone. ”

    // Systemd does not canonicalize the link, but only checks if it is prefixed by
    // "/usr/share/zoneinfo/" or "../usr/share/zoneinfo/". So we do the same.
    // <https://github.com/systemd/systemd/blob/9102c625a673a3246d7e73d8737f3494446bad4e/src/basic/time-util.c#L1493>

    const PREFIXES: &[&str] = &[
        "/usr/share/zoneinfo/",   // absolute path
        "../usr/share/zoneinfo/", // relative path
        "/etc/zoneinfo/",         // absolute path for NixOS
        "../etc/zoneinfo/",       // relative path for NixOS
    ];
    let mut s = read_link("/etc/localtime")?
        .into_os_string()
        .into_string()
        .map_err(|_| crate::GetTimezoneError::FailedParsingString)?;
    for &prefix in PREFIXES {
        if s.starts_with(prefix) {
            // Trim to the correct length without allocating.
            s.replace_range(..prefix.len(), "");
            return Ok(s);
        }
    }
    Err(crate::GetTimezoneError::FailedParsingString)
}
