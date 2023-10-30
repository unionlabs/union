// This is a part of Chrono.
// See README.md and LICENSE.txt for details.

/*!
`strftime`/`strptime`-inspired date and time formatting syntax.

## Specifiers

The following specifiers are available both to formatting and parsing.

| Spec. | Example  | Description                                                                |
|-------|----------|----------------------------------------------------------------------------|
|       |          | **DATE SPECIFIERS:**                                                       |
| `%Y`  | `2001`   | The full proleptic Gregorian year, zero-padded to 4 digits. chrono supports years from -262144 to 262143. |
| `%C`  | `20`     | The proleptic Gregorian year divided by 100, zero-padded to 2 digits. [^1] |
| `%y`  | `01`     | The proleptic Gregorian year modulo 100, zero-padded to 2 digits. [^1]     |
|       |          |                                                                            |
| `%m`  | `07`     | Month number (01--12), zero-padded to 2 digits.                            |
| `%b`  | `Jul`    | Abbreviated month name. Always 3 letters.                                  |
| `%B`  | `July`   | Full month name. Also accepts corresponding abbreviation in parsing.       |
| `%h`  | `Jul`    | Same as `%b`.                                                              |
|       |          |                                                                            |
| `%d`  | `08`     | Day number (01--31), zero-padded to 2 digits.                              |
| `%e`  | ` 8`     | Same as `%d` but space-padded. Same as `%_d`.                              |
|       |          |                                                                            |
| `%a`  | `Sun`    | Abbreviated weekday name. Always 3 letters.                                |
| `%A`  | `Sunday` | Full weekday name. Also accepts corresponding abbreviation in parsing.     |
| `%w`  | `0`      | Sunday = 0, Monday = 1, ..., Saturday = 6.                                 |
| `%u`  | `7`      | Monday = 1, Tuesday = 2, ..., Sunday = 7. (ISO 8601)                       |
|       |          |                                                                            |
| `%U`  | `28`     | Week number starting with Sunday (00--53), zero-padded to 2 digits. [^2]   |
| `%W`  | `27`     | Same as `%U`, but week 1 starts with the first Monday in that year instead.|
|       |          |                                                                            |
| `%G`  | `2001`   | Same as `%Y` but uses the year number in ISO 8601 week date. [^3]          |
| `%g`  | `01`     | Same as `%y` but uses the year number in ISO 8601 week date. [^3]          |
| `%V`  | `27`     | Same as `%U` but uses the week number in ISO 8601 week date (01--53). [^3] |
|       |          |                                                                            |
| `%j`  | `189`    | Day of the year (001--366), zero-padded to 3 digits.                       |
|       |          |                                                                            |
| `%D`  | `07/08/01`    | Month-day-year format. Same as `%m/%d/%y`.                            |
| `%x`  | `07/08/01`    | Locale's date representation (e.g., 12/31/99).                        |
| `%F`  | `2001-07-08`  | Year-month-day format (ISO 8601). Same as `%Y-%m-%d`.                 |
| `%v`  | ` 8-Jul-2001` | Day-month-year format. Same as `%e-%b-%Y`.                            |
|       |          |                                                                            |
|       |          | **TIME SPECIFIERS:**                                                       |
| `%H`  | `00`     | Hour number (00--23), zero-padded to 2 digits.                             |
| `%k`  | ` 0`     | Same as `%H` but space-padded. Same as `%_H`.                              |
| `%I`  | `12`     | Hour number in 12-hour clocks (01--12), zero-padded to 2 digits.           |
| `%l`  | `12`     | Same as `%I` but space-padded. Same as `%_I`.                              |
|       |          |                                                                            |
| `%P`  | `am`     | `am` or `pm` in 12-hour clocks.                                            |
| `%p`  | `AM`     | `AM` or `PM` in 12-hour clocks.                                            |
|       |          |                                                                            |
| `%M`  | `34`     | Minute number (00--59), zero-padded to 2 digits.                           |
| `%S`  | `60`     | Second number (00--60), zero-padded to 2 digits. [^4]                      |
| `%f`  | `026490000`   | The fractional seconds (in nanoseconds) since last whole second. [^7] |
| `%.f` | `.026490`| Similar to `.%f` but left-aligned. These all consume the leading dot. [^7] |
| `%.3f`| `.026`        | Similar to `.%f` but left-aligned but fixed to a length of 3. [^7]    |
| `%.6f`| `.026490`     | Similar to `.%f` but left-aligned but fixed to a length of 6. [^7]    |
| `%.9f`| `.026490000`  | Similar to `.%f` but left-aligned but fixed to a length of 9. [^7]    |
| `%3f` | `026`         | Similar to `%.3f` but without the leading dot. [^7]                   |
| `%6f` | `026490`      | Similar to `%.6f` but without the leading dot. [^7]                   |
| `%9f` | `026490000`   | Similar to `%.9f` but without the leading dot. [^7]                   |
|       |               |                                                                       |
| `%R`  | `00:34`       | Hour-minute format. Same as `%H:%M`.                                  |
| `%T`  | `00:34:60`    | Hour-minute-second format. Same as `%H:%M:%S`.                        |
| `%X`  | `00:34:60`    | Locale's time representation (e.g., 23:13:48).                        |
| `%r`  | `12:34:60 AM` | Hour-minute-second format in 12-hour clocks. Same as `%I:%M:%S %p`.   |
|       |          |                                                                            |
|       |          | **TIME ZONE SPECIFIERS:**                                                  |
| `%Z`  | `ACST`   | Local time zone name. Skips all non-whitespace characters during parsing. [^8] |
| `%z`  | `+0930`  | Offset from the local time to UTC (with UTC being `+0000`).                |
| `%:z` | `+09:30` | Same as `%z` but with a colon.                                             |
|`%::z`|`+09:30:00`| Offset from the local time to UTC with seconds.                            |
|`%:::z`| `+09`    | Offset from the local time to UTC without minutes.                         |
| `%#z` | `+09`    | *Parsing only:* Same as `%z` but allows minutes to be missing or present.  |
|       |          |                                                                            |
|       |          | **DATE & TIME SPECIFIERS:**                                                |
|`%c`|`Sun Jul  8 00:34:60 2001`|Locale's date and time (e.g., Thu Mar  3 23:05:25 2005).       |
| `%+`  | `2001-07-08T00:34:60.026490+09:30` | ISO 8601 / RFC 3339 date & time format. [^5]     |
|       |               |                                                                       |
| `%s`  | `994518299`   | UNIX timestamp, the number of seconds since 1970-01-01 00:00 UTC. [^6]|
|       |          |                                                                            |
|       |          | **SPECIAL SPECIFIERS:**                                                    |
| `%t`  |          | Literal tab (`\t`).                                                        |
| `%n`  |          | Literal newline (`\n`).                                                    |
| `%%`  |          | Literal percent sign.                                                      |

It is possible to override the default padding behavior of numeric specifiers `%?`.
This is not allowed for other specifiers and will result in the `BAD_FORMAT` error.

Modifier | Description
-------- | -----------
`%-?`    | Suppresses any padding including spaces and zeroes. (e.g. `%j` = `012`, `%-j` = `12`)
`%_?`    | Uses spaces as a padding. (e.g. `%j` = `012`, `%_j` = ` 12`)
`%0?`    | Uses zeroes as a padding. (e.g. `%e` = ` 9`, `%0e` = `09`)

Notes:

[^1]: `%C`, `%y`:
   This is floor division, so 100 BCE (year number -99) will print `-1` and `99` respectively.

[^2]: `%U`:
   Week 1 starts with the first Sunday in that year.
   It is possible to have week 0 for days before the first Sunday.

[^3]: `%G`, `%g`, `%V`:
   Week 1 is the first week with at least 4 days in that year.
   Week 0 does not exist, so this should be used with `%G` or `%g`.

[^4]: `%S`:
   It accounts for leap seconds, so `60` is possible.

[^5]: `%+`: Same as `%Y-%m-%dT%H:%M:%S%.f%:z`, i.e. 0, 3, 6 or 9 fractional
   digits for seconds and colons in the time zone offset.
   <br>
   <br>
   This format also supports having a `Z` or `UTC` in place of `%:z`. They
   are equivalent to `+00:00`.
   <br>
   <br>
   Note that all `T`, `Z`, and `UTC` are parsed case-insensitively.
   <br>
   <br>
   The typical `strftime` implementations have different (and locale-dependent)
   formats for this specifier. While Chrono's format for `%+` is far more
   stable, it is best to avoid this specifier if you want to control the exact
   output.

[^6]: `%s`:
   This is not padded and can be negative.
   For the purpose of Chrono, it only accounts for non-leap seconds
   so it slightly differs from ISO C `strftime` behavior.

[^7]: `%f`, `%.f`, `%.3f`, `%.6f`, `%.9f`, `%3f`, `%6f`, `%9f`:
   <br>
   The default `%f` is right-aligned and always zero-padded to 9 digits
   for the compatibility with glibc and others,
   so it always counts the number of nanoseconds since the last whole second.
   E.g. 7ms after the last second will print `007000000`,
   and parsing `7000000` will yield the same.
   <br>
   <br>
   The variant `%.f` is left-aligned and print 0, 3, 6 or 9 fractional digits
   according to the precision.
   E.g. 70ms after the last second under `%.f` will print `.070` (note: not `.07`),
   and parsing `.07`, `.070000` etc. will yield the same.
   Note that they can print or read nothing if the fractional part is zero or
   the next character is not `.`.
   <br>
   <br>
   The variant `%.3f`, `%.6f` and `%.9f` are left-aligned and print 3, 6 or 9 fractional digits
   according to the number preceding `f`.
   E.g. 70ms after the last second under `%.3f` will print `.070` (note: not `.07`),
   and parsing `.07`, `.070000` etc. will yield the same.
   Note that they can read nothing if the fractional part is zero or
   the next character is not `.` however will print with the specified length.
   <br>
   <br>
   The variant `%3f`, `%6f` and `%9f` are left-aligned and print 3, 6 or 9 fractional digits
   according to the number preceding `f`, but without the leading dot.
   E.g. 70ms after the last second under `%3f` will print `070` (note: not `07`),
   and parsing `07`, `070000` etc. will yield the same.
   Note that they can read nothing if the fractional part is zero.

[^8]: `%Z`:
   Offset will not be populated from the parsed data, nor will it be validated.
   Timezone is completely ignored. Similar to the glibc `strptime` treatment of
   this format code.
   <br>
   <br>
   It is not possible to reliably convert from an abbreviation to an offset,
   for example CDT can mean either Central Daylight Time (North America) or
   China Daylight Time.
*/

#[cfg(feature = "unstable-locales")]
extern crate alloc;

#[cfg(feature = "unstable-locales")]
use alloc::vec::Vec;

#[cfg(feature = "unstable-locales")]
use super::{locales, Locale};
use super::{Fixed, InternalFixed, InternalInternal, Item, Numeric, Pad};

#[cfg(feature = "unstable-locales")]
type Fmt<'a> = Vec<Item<'a>>;
#[cfg(not(feature = "unstable-locales"))]
type Fmt<'a> = &'static [Item<'static>];

static D_FMT: &[Item<'static>] =
    &[num0!(Month), lit!("/"), num0!(Day), lit!("/"), num0!(YearMod100)];
static D_T_FMT: &[Item<'static>] = &[
    fix!(ShortWeekdayName),
    sp!(" "),
    fix!(ShortMonthName),
    sp!(" "),
    nums!(Day),
    sp!(" "),
    num0!(Hour),
    lit!(":"),
    num0!(Minute),
    lit!(":"),
    num0!(Second),
    sp!(" "),
    num0!(Year),
];
static T_FMT: &[Item<'static>] = &[num0!(Hour), lit!(":"), num0!(Minute), lit!(":"), num0!(Second)];

/// Parsing iterator for `strftime`-like format strings.
#[derive(Clone, Debug)]
pub struct StrftimeItems<'a> {
    /// Remaining portion of the string.
    remainder: &'a str,
    /// If the current specifier is composed of multiple formatting items (e.g. `%+`),
    /// parser refers to the statically reconstructed slice of them.
    /// If `recons` is not empty they have to be returned earlier than the `remainder`.
    recons: Fmt<'a>,
    /// Date format
    d_fmt: Fmt<'a>,
    /// Date and time format
    d_t_fmt: Fmt<'a>,
    /// Time format
    t_fmt: Fmt<'a>,
}

impl<'a> StrftimeItems<'a> {
    /// Creates a new parsing iterator from the `strftime`-like format string.
    pub fn new(s: &'a str) -> StrftimeItems<'a> {
        Self::with_remainer(s)
    }

    /// Creates a new parsing iterator from the `strftime`-like format string.
    #[cfg(feature = "unstable-locales")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-locales")))]
    pub fn new_with_locale(s: &'a str, locale: Locale) -> StrftimeItems<'a> {
        let d_fmt = StrftimeItems::new(locales::d_fmt(locale)).collect();
        let d_t_fmt = StrftimeItems::new(locales::d_t_fmt(locale)).collect();
        let t_fmt = StrftimeItems::new(locales::t_fmt(locale)).collect();

        StrftimeItems { remainder: s, recons: Vec::new(), d_fmt, d_t_fmt, t_fmt }
    }

    #[cfg(not(feature = "unstable-locales"))]
    fn with_remainer(s: &'a str) -> StrftimeItems<'a> {
        static FMT_NONE: &[Item<'static>; 0] = &[];

        StrftimeItems {
            remainder: s,
            recons: FMT_NONE,
            d_fmt: D_FMT,
            d_t_fmt: D_T_FMT,
            t_fmt: T_FMT,
        }
    }

    #[cfg(feature = "unstable-locales")]
    fn with_remainer(s: &'a str) -> StrftimeItems<'a> {
        StrftimeItems {
            remainder: s,
            recons: Vec::new(),
            d_fmt: D_FMT.to_vec(),
            d_t_fmt: D_T_FMT.to_vec(),
            t_fmt: T_FMT.to_vec(),
        }
    }
}

const HAVE_ALTERNATES: &str = "z";

impl<'a> Iterator for StrftimeItems<'a> {
    type Item = Item<'a>;

    fn next(&mut self) -> Option<Item<'a>> {
        // we have some reconstructed items to return
        if !self.recons.is_empty() {
            let item;
            #[cfg(feature = "unstable-locales")]
            {
                item = self.recons.remove(0);
            }
            #[cfg(not(feature = "unstable-locales"))]
            {
                item = self.recons[0].clone();
                self.recons = &self.recons[1..];
            }
            return Some(item);
        }

        match self.remainder.chars().next() {
            // we are done
            None => None,

            // the next item is a specifier
            Some('%') => {
                self.remainder = &self.remainder[1..];

                macro_rules! next {
                    () => {
                        match self.remainder.chars().next() {
                            Some(x) => {
                                self.remainder = &self.remainder[x.len_utf8()..];
                                x
                            }
                            None => return Some(Item::Error), // premature end of string
                        }
                    };
                }

                let spec = next!();
                let pad_override = match spec {
                    '-' => Some(Pad::None),
                    '0' => Some(Pad::Zero),
                    '_' => Some(Pad::Space),
                    _ => None,
                };
                let is_alternate = spec == '#';
                let spec = if pad_override.is_some() || is_alternate { next!() } else { spec };
                if is_alternate && !HAVE_ALTERNATES.contains(spec) {
                    return Some(Item::Error);
                }

                macro_rules! recons {
                    [$head:expr, $($tail:expr),+ $(,)*] => ({
                        #[cfg(feature = "unstable-locales")]
                        {
                            self.recons.clear();
                            $(self.recons.push($tail);)+
                        }
                        #[cfg(not(feature = "unstable-locales"))]
                        {
                            const RECONS: &'static [Item<'static>] = &[$($tail),+];
                            self.recons = RECONS;
                        }
                        $head
                    })
                }

                macro_rules! recons_from_slice {
                    ($slice:expr) => {{
                        #[cfg(feature = "unstable-locales")]
                        {
                            self.recons.clear();
                            self.recons.extend_from_slice(&$slice[1..]);
                        }
                        #[cfg(not(feature = "unstable-locales"))]
                        {
                            self.recons = &$slice[1..];
                        }
                        $slice[0].clone()
                    }};
                }

                let item = match spec {
                    'A' => fix!(LongWeekdayName),
                    'B' => fix!(LongMonthName),
                    'C' => num0!(YearDiv100),
                    'D' => {
                        recons![num0!(Month), lit!("/"), num0!(Day), lit!("/"), num0!(YearMod100)]
                    }
                    'F' => recons![num0!(Year), lit!("-"), num0!(Month), lit!("-"), num0!(Day)],
                    'G' => num0!(IsoYear),
                    'H' => num0!(Hour),
                    'I' => num0!(Hour12),
                    'M' => num0!(Minute),
                    'P' => fix!(LowerAmPm),
                    'R' => recons![num0!(Hour), lit!(":"), num0!(Minute)],
                    'S' => num0!(Second),
                    'T' => recons![num0!(Hour), lit!(":"), num0!(Minute), lit!(":"), num0!(Second)],
                    'U' => num0!(WeekFromSun),
                    'V' => num0!(IsoWeek),
                    'W' => num0!(WeekFromMon),
                    'X' => recons_from_slice!(self.t_fmt),
                    'Y' => num0!(Year),
                    'Z' => fix!(TimezoneName),
                    'a' => fix!(ShortWeekdayName),
                    'b' | 'h' => fix!(ShortMonthName),
                    'c' => recons_from_slice!(self.d_t_fmt),
                    'd' => num0!(Day),
                    'e' => nums!(Day),
                    'f' => num0!(Nanosecond),
                    'g' => num0!(IsoYearMod100),
                    'j' => num0!(Ordinal),
                    'k' => nums!(Hour),
                    'l' => nums!(Hour12),
                    'm' => num0!(Month),
                    'n' => sp!("\n"),
                    'p' => fix!(UpperAmPm),
                    'r' => recons![
                        num0!(Hour12),
                        lit!(":"),
                        num0!(Minute),
                        lit!(":"),
                        num0!(Second),
                        sp!(" "),
                        fix!(UpperAmPm)
                    ],
                    's' => num!(Timestamp),
                    't' => sp!("\t"),
                    'u' => num!(WeekdayFromMon),
                    'v' => {
                        recons![nums!(Day), lit!("-"), fix!(ShortMonthName), lit!("-"), num0!(Year)]
                    }
                    'w' => num!(NumDaysFromSun),
                    'x' => recons_from_slice!(self.d_fmt),
                    'y' => num0!(YearMod100),
                    'z' => {
                        if is_alternate {
                            internal_fix!(TimezoneOffsetPermissive)
                        } else {
                            fix!(TimezoneOffset)
                        }
                    }
                    '+' => fix!(RFC3339),
                    ':' => {
                        if self.remainder.starts_with("::z") {
                            self.remainder = &self.remainder[3..];
                            fix!(TimezoneOffsetTripleColon)
                        } else if self.remainder.starts_with(":z") {
                            self.remainder = &self.remainder[2..];
                            fix!(TimezoneOffsetDoubleColon)
                        } else if self.remainder.starts_with('z') {
                            self.remainder = &self.remainder[1..];
                            fix!(TimezoneOffsetColon)
                        } else {
                            Item::Error
                        }
                    }
                    '.' => match next!() {
                        '3' => match next!() {
                            'f' => fix!(Nanosecond3),
                            _ => Item::Error,
                        },
                        '6' => match next!() {
                            'f' => fix!(Nanosecond6),
                            _ => Item::Error,
                        },
                        '9' => match next!() {
                            'f' => fix!(Nanosecond9),
                            _ => Item::Error,
                        },
                        'f' => fix!(Nanosecond),
                        _ => Item::Error,
                    },
                    '3' => match next!() {
                        'f' => internal_fix!(Nanosecond3NoDot),
                        _ => Item::Error,
                    },
                    '6' => match next!() {
                        'f' => internal_fix!(Nanosecond6NoDot),
                        _ => Item::Error,
                    },
                    '9' => match next!() {
                        'f' => internal_fix!(Nanosecond9NoDot),
                        _ => Item::Error,
                    },
                    '%' => lit!("%"),
                    _ => Item::Error, // no such specifier
                };

                // adjust `item` if we have any padding modifier
                if let Some(new_pad) = pad_override {
                    match item {
                        Item::Numeric(ref kind, _pad) if self.recons.is_empty() => {
                            Some(Item::Numeric(kind.clone(), new_pad))
                        }
                        _ => Some(Item::Error), // no reconstructed or non-numeric item allowed
                    }
                } else {
                    Some(item)
                }
            }

            // the next item is space
            Some(c) if c.is_whitespace() => {
                // `%` is not a whitespace, so `c != '%'` is redundant
                let nextspec = self
                    .remainder
                    .find(|c: char| !c.is_whitespace())
                    .unwrap_or(self.remainder.len());
                assert!(nextspec > 0);
                let item = sp!(&self.remainder[..nextspec]);
                self.remainder = &self.remainder[nextspec..];
                Some(item)
            }

            // the next item is literal
            _ => {
                let nextspec = self
                    .remainder
                    .find(|c: char| c.is_whitespace() || c == '%')
                    .unwrap_or(self.remainder.len());
                assert!(nextspec > 0);
                let item = lit!(&self.remainder[..nextspec]);
                self.remainder = &self.remainder[nextspec..];
                Some(item)
            }
        }
    }
}

#[cfg(test)]
#[test]
fn test_strftime_items() {
    fn parse_and_collect(s: &str) -> Vec<Item<'_>> {
        // map any error into `[Item::Error]`. useful for easy testing.
        let items = StrftimeItems::new(s);
        let items = items.map(|spec| if spec == Item::Error { None } else { Some(spec) });
        items.collect::<Option<Vec<_>>>().unwrap_or_else(|| vec![Item::Error])
    }

    assert_eq!(parse_and_collect(""), []);
    assert_eq!(parse_and_collect(" \t\n\r "), [sp!(" \t\n\r ")]);
    assert_eq!(parse_and_collect("hello?"), [lit!("hello?")]);
    assert_eq!(
        parse_and_collect("a  b\t\nc"),
        [lit!("a"), sp!("  "), lit!("b"), sp!("\t\n"), lit!("c")]
    );
    assert_eq!(parse_and_collect("100%%"), [lit!("100"), lit!("%")]);
    assert_eq!(parse_and_collect("100%% ok"), [lit!("100"), lit!("%"), sp!(" "), lit!("ok")]);
    assert_eq!(parse_and_collect("%%PDF-1.0"), [lit!("%"), lit!("PDF-1.0")]);
    assert_eq!(
        parse_and_collect("%Y-%m-%d"),
        [num0!(Year), lit!("-"), num0!(Month), lit!("-"), num0!(Day)]
    );
    assert_eq!(parse_and_collect("[%F]"), parse_and_collect("[%Y-%m-%d]"));
    assert_eq!(parse_and_collect("%m %d"), [num0!(Month), sp!(" "), num0!(Day)]);
    assert_eq!(parse_and_collect("%"), [Item::Error]);
    assert_eq!(parse_and_collect("%%"), [lit!("%")]);
    assert_eq!(parse_and_collect("%%%"), [Item::Error]);
    assert_eq!(parse_and_collect("%%%%"), [lit!("%"), lit!("%")]);
    assert_eq!(parse_and_collect("foo%?"), [Item::Error]);
    assert_eq!(parse_and_collect("bar%42"), [Item::Error]);
    assert_eq!(parse_and_collect("quux% +"), [Item::Error]);
    assert_eq!(parse_and_collect("%.Z"), [Item::Error]);
    assert_eq!(parse_and_collect("%:Z"), [Item::Error]);
    assert_eq!(parse_and_collect("%-Z"), [Item::Error]);
    assert_eq!(parse_and_collect("%0Z"), [Item::Error]);
    assert_eq!(parse_and_collect("%_Z"), [Item::Error]);
    assert_eq!(parse_and_collect("%.j"), [Item::Error]);
    assert_eq!(parse_and_collect("%:j"), [Item::Error]);
    assert_eq!(parse_and_collect("%-j"), [num!(Ordinal)]);
    assert_eq!(parse_and_collect("%0j"), [num0!(Ordinal)]);
    assert_eq!(parse_and_collect("%_j"), [nums!(Ordinal)]);
    assert_eq!(parse_and_collect("%.e"), [Item::Error]);
    assert_eq!(parse_and_collect("%:e"), [Item::Error]);
    assert_eq!(parse_and_collect("%-e"), [num!(Day)]);
    assert_eq!(parse_and_collect("%0e"), [num0!(Day)]);
    assert_eq!(parse_and_collect("%_e"), [nums!(Day)]);
    assert_eq!(parse_and_collect("%z"), [fix!(TimezoneOffset)]);
    assert_eq!(parse_and_collect("%#z"), [internal_fix!(TimezoneOffsetPermissive)]);
    assert_eq!(parse_and_collect("%#m"), [Item::Error]);
}

#[cfg(test)]
#[test]
fn test_strftime_docs() {
    use crate::NaiveDate;
    use crate::{DateTime, FixedOffset, TimeZone, Timelike, Utc};

    let dt = FixedOffset::east_opt(34200)
        .unwrap()
        .from_local_datetime(
            &NaiveDate::from_ymd_opt(2001, 7, 8)
                .unwrap()
                .and_hms_nano_opt(0, 34, 59, 1_026_490_708)
                .unwrap(),
        )
        .unwrap();

    // date specifiers
    assert_eq!(dt.format("%Y").to_string(), "2001");
    assert_eq!(dt.format("%C").to_string(), "20");
    assert_eq!(dt.format("%y").to_string(), "01");
    assert_eq!(dt.format("%m").to_string(), "07");
    assert_eq!(dt.format("%b").to_string(), "Jul");
    assert_eq!(dt.format("%B").to_string(), "July");
    assert_eq!(dt.format("%h").to_string(), "Jul");
    assert_eq!(dt.format("%d").to_string(), "08");
    assert_eq!(dt.format("%e").to_string(), " 8");
    assert_eq!(dt.format("%e").to_string(), dt.format("%_d").to_string());
    assert_eq!(dt.format("%a").to_string(), "Sun");
    assert_eq!(dt.format("%A").to_string(), "Sunday");
    assert_eq!(dt.format("%w").to_string(), "0");
    assert_eq!(dt.format("%u").to_string(), "7");
    assert_eq!(dt.format("%U").to_string(), "28");
    assert_eq!(dt.format("%W").to_string(), "27");
    assert_eq!(dt.format("%G").to_string(), "2001");
    assert_eq!(dt.format("%g").to_string(), "01");
    assert_eq!(dt.format("%V").to_string(), "27");
    assert_eq!(dt.format("%j").to_string(), "189");
    assert_eq!(dt.format("%D").to_string(), "07/08/01");
    assert_eq!(dt.format("%x").to_string(), "07/08/01");
    assert_eq!(dt.format("%F").to_string(), "2001-07-08");
    assert_eq!(dt.format("%v").to_string(), " 8-Jul-2001");

    // time specifiers
    assert_eq!(dt.format("%H").to_string(), "00");
    assert_eq!(dt.format("%k").to_string(), " 0");
    assert_eq!(dt.format("%k").to_string(), dt.format("%_H").to_string());
    assert_eq!(dt.format("%I").to_string(), "12");
    assert_eq!(dt.format("%l").to_string(), "12");
    assert_eq!(dt.format("%l").to_string(), dt.format("%_I").to_string());
    assert_eq!(dt.format("%P").to_string(), "am");
    assert_eq!(dt.format("%p").to_string(), "AM");
    assert_eq!(dt.format("%M").to_string(), "34");
    assert_eq!(dt.format("%S").to_string(), "60");
    assert_eq!(dt.format("%f").to_string(), "026490708");
    assert_eq!(dt.format("%.f").to_string(), ".026490708");
    assert_eq!(dt.with_nanosecond(1_026_490_000).unwrap().format("%.f").to_string(), ".026490");
    assert_eq!(dt.format("%.3f").to_string(), ".026");
    assert_eq!(dt.format("%.6f").to_string(), ".026490");
    assert_eq!(dt.format("%.9f").to_string(), ".026490708");
    assert_eq!(dt.format("%3f").to_string(), "026");
    assert_eq!(dt.format("%6f").to_string(), "026490");
    assert_eq!(dt.format("%9f").to_string(), "026490708");
    assert_eq!(dt.format("%R").to_string(), "00:34");
    assert_eq!(dt.format("%T").to_string(), "00:34:60");
    assert_eq!(dt.format("%X").to_string(), "00:34:60");
    assert_eq!(dt.format("%r").to_string(), "12:34:60 AM");

    // time zone specifiers
    //assert_eq!(dt.format("%Z").to_string(), "ACST");
    assert_eq!(dt.format("%z").to_string(), "+0930");
    assert_eq!(dt.format("%:z").to_string(), "+09:30");
    assert_eq!(dt.format("%::z").to_string(), "+09:30:00");
    assert_eq!(dt.format("%:::z").to_string(), "+09");

    // date & time specifiers
    assert_eq!(dt.format("%c").to_string(), "Sun Jul  8 00:34:60 2001");
    assert_eq!(dt.format("%+").to_string(), "2001-07-08T00:34:60.026490708+09:30");

    assert_eq!(
        dt.with_timezone(&Utc).format("%+").to_string(),
        "2001-07-07T15:04:60.026490708+00:00"
    );
    assert_eq!(
        dt.with_timezone(&Utc),
        DateTime::parse_from_str("2001-07-07T15:04:60.026490708Z", "%+").unwrap()
    );
    assert_eq!(
        dt.with_timezone(&Utc),
        DateTime::parse_from_str("2001-07-07T15:04:60.026490708UTC", "%+").unwrap()
    );
    assert_eq!(
        dt.with_timezone(&Utc),
        DateTime::parse_from_str("2001-07-07t15:04:60.026490708utc", "%+").unwrap()
    );

    assert_eq!(
        dt.with_nanosecond(1_026_490_000).unwrap().format("%+").to_string(),
        "2001-07-08T00:34:60.026490+09:30"
    );
    assert_eq!(dt.format("%s").to_string(), "994518299");

    // special specifiers
    assert_eq!(dt.format("%t").to_string(), "\t");
    assert_eq!(dt.format("%n").to_string(), "\n");
    assert_eq!(dt.format("%%").to_string(), "%");
}

#[cfg(feature = "unstable-locales")]
#[test]
fn test_strftime_docs_localized() {
    use crate::{FixedOffset, TimeZone};

    let dt = FixedOffset::east_opt(34200).unwrap().ymd_opt(2001, 7, 8).unwrap().and_hms_nano(
        0,
        34,
        59,
        1_026_490_708,
    );

    // date specifiers
    assert_eq!(dt.format_localized("%b", Locale::fr_BE).to_string(), "jui");
    assert_eq!(dt.format_localized("%B", Locale::fr_BE).to_string(), "juillet");
    assert_eq!(dt.format_localized("%h", Locale::fr_BE).to_string(), "jui");
    assert_eq!(dt.format_localized("%a", Locale::fr_BE).to_string(), "dim");
    assert_eq!(dt.format_localized("%A", Locale::fr_BE).to_string(), "dimanche");
    assert_eq!(dt.format_localized("%D", Locale::fr_BE).to_string(), "07/08/01");
    assert_eq!(dt.format_localized("%x", Locale::fr_BE).to_string(), "08/07/01");
    assert_eq!(dt.format_localized("%F", Locale::fr_BE).to_string(), "2001-07-08");
    assert_eq!(dt.format_localized("%v", Locale::fr_BE).to_string(), " 8-jui-2001");

    // time specifiers
    assert_eq!(dt.format_localized("%P", Locale::fr_BE).to_string(), "");
    assert_eq!(dt.format_localized("%p", Locale::fr_BE).to_string(), "");
    assert_eq!(dt.format_localized("%R", Locale::fr_BE).to_string(), "00:34");
    assert_eq!(dt.format_localized("%T", Locale::fr_BE).to_string(), "00:34:60");
    assert_eq!(dt.format_localized("%X", Locale::fr_BE).to_string(), "00:34:60");
    assert_eq!(dt.format_localized("%r", Locale::fr_BE).to_string(), "12:34:60 ");

    // date & time specifiers
    assert_eq!(
        dt.format_localized("%c", Locale::fr_BE).to_string(),
        "dim 08 jui 2001 00:34:60 +09:30"
    );
}
