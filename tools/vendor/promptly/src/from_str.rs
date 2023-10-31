use super::*;
use std::str::FromStr;

#[cfg(feature = "nightly")]
/// Blanket impl for `FromStr` types. Re-prompts until `FromStr` parsing succeeds.
default impl<T> Promptable for T
where
    T: FromStr + Display,
    <T as FromStr>::Err: ::std::error::Error,
{
    /// Prompt until the input parses into the specified type
    ///
    /// ```no_run
    /// use promptly::Promptable;
    /// u32::prompt("Enter your age")?;
    /// # Result::<_,Box<std::error::Error>>::Ok(())
    /// ```
    fn prompt<S: AsRef<str>>(msg: S) -> Result<Self> {
        prompt_parse(msg)
    }

    /// Prompt for an optional, parseable value.
    ///
    /// Returns `None` if empty, otherwise prompts until input parses into specified type.
    ///
    /// ```no_run
    /// # use std::net::IpAddr;
    /// use promptly::Promptable;
    /// IpAddr::prompt_opt("Enter your IP Address (optional)")?;
    /// # Result::<_,Box<std::error::Error>>::Ok(())
    /// ```
    fn prompt_opt<S: AsRef<str>>(msg: S) -> Result<Option<Self>> {
        prompt_parse_opt(msg)
    }

    /// Prompt for a parseable value with a provided fallback value if empty.
    ///
    /// ```no_run
    /// use promptly::Promptable;
    /// u32::prompt_default("Enter the year", 2018)?;
    /// # Result::<_,Box<std::error::Error>>::Ok(())
    /// ```
    ///
    /// Default value is visible in the prompt as: `(default=USA)`
    fn prompt_default<S: AsRef<str>>(msg: S, default: Self) -> Result<Self> {
        let msg = format!("{} (default={})", msg.as_ref(), default);
        prompt_parse_opt(msg).unwrap_or(default)
    }
}

impl Promptable for String {
    /// Prompt until you get a non-empty string
    ///
    /// ```no_run
    /// use promptly::Promptable;
    /// String::prompt("Enter your name")?;
    /// # Result::<_,Box<std::error::Error>>::Ok(())
    /// ```
    fn prompt<S: AsRef<str>>(msg: S) -> Result<Self> {
        Prompter::new().prompt_nonempty(msg)
    }

    /// Prompt for an optional string
    ///
    /// ```no_run
    /// use promptly::Promptable;
    /// String::prompt_opt("Enter your phone number (optional)")?;
    /// # Result::<_,Box<std::error::Error>>::Ok(())
    /// ```
    fn prompt_opt<S: AsRef<str>>(msg: S) -> Result<Option<Self>> {
        Prompter::new().prompt_opt(msg)
    }

    /// Prompt for a string with a provided fallback value if empty.
    ///
    /// ```no_run
    /// use promptly::Promptable;
    /// String::prompt_default("Enter your country", "USA".into())?;
    /// # Result::<_,Box<std::error::Error>>::Ok(())
    /// ```
    ///
    /// Default value is visible in the prompt as: `(default=USA)`
    fn prompt_default<S: AsRef<str>>(msg: S, default: Self) -> Result<Self> {
        let msg = format!("{} (default={})", msg.as_ref(), default);
        Ok(Prompter::new().prompt_opt(msg)?.unwrap_or(default))
    }
}

// Macro to provide Promptable implementations until specialization stabilizes
macro_rules! impl_promptable_from_str {
    ($t:ty) => {
        impl Promptable for $t {
            fn prompt<S: AsRef<str>>(msg: S) -> Result<Self> {
                prompt_parse(msg)
            }

            fn prompt_opt<S: AsRef<str>>(msg: S) -> Result<Option<Self>> {
                prompt_parse_opt(msg)
            }

            fn prompt_default<S: AsRef<str>>(msg: S, default: Self) -> Result<Self> {
                let msg = format!("{} (default={})", msg.as_ref(), default);
                Ok(prompt_parse_opt(msg)?.unwrap_or(default))
            }
        }
    };
}

impl_promptable_from_str!(char);
impl_promptable_from_str!(u8);
impl_promptable_from_str!(u16);
impl_promptable_from_str!(u32);
impl_promptable_from_str!(u64);
impl_promptable_from_str!(u128);
impl_promptable_from_str!(usize);
impl_promptable_from_str!(i8);
impl_promptable_from_str!(i16);
impl_promptable_from_str!(i32);
impl_promptable_from_str!(i64);
impl_promptable_from_str!(i128);
impl_promptable_from_str!(isize);
impl_promptable_from_str!(f32);
impl_promptable_from_str!(f64);
impl_promptable_from_str!(::std::net::IpAddr);
impl_promptable_from_str!(::std::net::Ipv4Addr);
impl_promptable_from_str!(::std::net::Ipv6Addr);
impl_promptable_from_str!(::std::net::SocketAddrV4);
impl_promptable_from_str!(::std::net::SocketAddrV6);
impl_promptable_from_str!(::std::num::NonZeroI128);
impl_promptable_from_str!(::std::num::NonZeroI64);
impl_promptable_from_str!(::std::num::NonZeroI32);
impl_promptable_from_str!(::std::num::NonZeroI16);
impl_promptable_from_str!(::std::num::NonZeroI8);
impl_promptable_from_str!(::std::num::NonZeroIsize);
impl_promptable_from_str!(::std::num::NonZeroU128);
impl_promptable_from_str!(::std::num::NonZeroU64);
impl_promptable_from_str!(::std::num::NonZeroU32);
impl_promptable_from_str!(::std::num::NonZeroU16);
impl_promptable_from_str!(::std::num::NonZeroU8);
impl_promptable_from_str!(::std::num::NonZeroUsize);

#[cfg(feature = "url")]
impl_promptable_from_str!(url::Url);

fn prompt_parse<T, S>(msg: S) -> Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: ::std::error::Error,
    S: AsRef<str>,
{
    Prompter::new().prompt_then(msg, |s| T::from_str(s.as_ref()).map_err(|e| e.to_string()))
}

fn prompt_parse_opt<T, S>(msg: S) -> Result<Option<T>>
where
    T: FromStr,
    <T as FromStr>::Err: ::std::error::Error,
    S: AsRef<str>,
{
    Prompter::new().prompt_then(msg, |s| match s.trim() {
        "" => Ok(None),
        _ => match T::from_str(s.as_ref()) {
            Ok(n) => Ok(Some(n)),
            Err(e) => Err(e.to_string()),
        },
    })
}
