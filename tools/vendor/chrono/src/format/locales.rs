use pure_rust_locales::{locale_match, Locale};

pub(crate) fn short_months(locale: Locale) -> &'static [&'static str] {
    locale_match!(locale => LC_TIME::ABMON)
}

pub(crate) fn long_months(locale: Locale) -> &'static [&'static str] {
    locale_match!(locale => LC_TIME::MON)
}

pub(crate) fn short_weekdays(locale: Locale) -> &'static [&'static str] {
    locale_match!(locale => LC_TIME::ABDAY)
}

pub(crate) fn long_weekdays(locale: Locale) -> &'static [&'static str] {
    locale_match!(locale => LC_TIME::DAY)
}

pub(crate) fn am_pm(locale: Locale) -> &'static [&'static str] {
    locale_match!(locale => LC_TIME::AM_PM)
}

pub(crate) fn d_fmt(locale: Locale) -> &'static str {
    locale_match!(locale => LC_TIME::D_FMT)
}

pub(crate) fn d_t_fmt(locale: Locale) -> &'static str {
    locale_match!(locale => LC_TIME::D_T_FMT)
}

pub(crate) fn t_fmt(locale: Locale) -> &'static str {
    locale_match!(locale => LC_TIME::T_FMT)
}
