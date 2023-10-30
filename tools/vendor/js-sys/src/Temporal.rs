use super::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Temporal, extends = Object)]
    #[derive(Clone, Debug)]
    pub type Instant;

    #[wasm_bindgen(static_method_of = Instant, js_namespace = Temporal)]
    pub fn from(val: &JsValue) -> Instant;

    #[wasm_bindgen(static_method_of = Instant, js_namespace = Temporal)]
    pub fn compare(one: &Instant, two: &Instant) -> i32;

    #[wasm_bindgen(method, getter, structural, js_name = epochSeconds)]
    pub fn epoch_seconds(this: &Instant) -> u64;
    #[wasm_bindgen(method, getter, structural, js_name = epochMilliseconds)]
    pub fn epoch_milliseconds(this: &Instant) -> u64;
    #[wasm_bindgen(method, getter, structural, js_name = epochMicroseconds)]
    pub fn epoch_microseconds(this: &Instant) -> BigInt;
    #[wasm_bindgen(method, getter, structural, js_name = epochNanoseconds)]
    pub fn epoch_nanoseconds(this: &Instant) -> BigInt;

    /// This method adds duration to Instant.
    ///
    /// The years, months, weeks, and days fields of duration must be zero.
    #[wasm_bindgen(method)]
    pub fn add(this: &Instant, duration: &Duration, options: &JsValue) -> Instant;

    /// This method subtracts duration to Instant.
    ///
    /// The years, months, weeks, and days fields of duration must be zero.
    #[wasm_bindgen(method)]
    pub fn subtract(this: &Instant, duration: &Duration, options: &JsValue) -> Instant;

    /// This method computes the difference between the two times represented by zonedDateTime and
    /// other, optionally rounds it, and returns it as a Temporal.Duration object. If other is earlier
    /// than zonedDateTime then the resulting duration will be negative. The returned Temporal.Duration, when
    /// added to zonedDateTime with the same options, will yield other.
    #[wasm_bindgen(method)]
    pub fn until(this: &Instant, other: &Instant, options: &JsValue) -> Duration;

    /// This method computes the difference between the two times represented by zonedDateTime and
    /// other, optionally rounds it, and returns it as a Temporal.Duration object. If other is later
    /// than zonedDateTime then the resulting duration will be negative. The returned Temporal.Duration, when
    /// added to zonedDateTime with the same options, will yield other.    
    #[wasm_bindgen(method)]
    pub fn since(this: &Instant, other: &Instant, options: &JsValue) -> Duration;

    /// Returns a new Temporal.Instant object which is zonedDateTime rounded to `roundTo` (if
    /// a string parameter is used) or `roundingIncrement` of `smallestUnit` (if an object parameter
    /// is used).
    #[wasm_bindgen(method)]
    pub fn round(this: &Instant, round_to: &JsValue) -> Instant;

    #[wasm_bindgen(method)]
    pub fn equals(this: &Instant, other: &Instant) -> bool;

    /// This method overrides the Object.prototype.toString() method and provides a convenient, unambiguous
    /// string representation of date. The string can be passed to Temporal.PlainDate.from() to
    /// create a new Temporal.PlainDate object.
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &Instant) -> JsString;

    /// This method overrides `Object.prototype.toLocaleString()` to provide a human-readable,
    /// language-sensitive representation of duration.
    ///
    /// The locales and options arguments are the same as in the constructor to Intl.DateTimeFormat.
    #[wasm_bindgen(method, js_name = toLocaleString)]
    pub fn to_locale_string(this: &Instant, locales: &JsValue, options: &JsValue) -> JsString;

}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Temporal, extends = Object)]
    #[derive(Clone, Debug)]
    pub type PlainDateTime;

    #[wasm_bindgen(static_method_of = PlainDateTime, js_namespace = Temporal)]
    pub fn from(val: &JsValue) -> PlainDateTime;

    #[wasm_bindgen(static_method_of = PlainDateTime, js_namespace = Temporal)]
    pub fn compare(one: &PlainDateTime, two: &PlainDateTime) -> i32;

    #[wasm_bindgen(method, getter, structural)]
    pub fn year(this: &PlainDateTime) -> i32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn month(this: &PlainDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn week(this: &PlainDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn day(this: &PlainDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn hour(this: &PlainDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn minute(this: &PlainDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn second(this: &PlainDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn millisecond(this: &PlainDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn microsecond(this: &PlainDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn nanosecond(this: &PlainDateTime) -> u32;

    /// In calendars that use eras, the era and eraYear read-only properties can be used
    /// together to resolve an era-relative year. Both properties are undefined when using
    /// the ISO 8601 calendar. As inputs to from or with, era and eraYear can be used instead
    /// of year. Unlike year, eraYear may decrease as time proceeds because some
    /// eras (like the BCE era in the Gregorian calendar) count years backwards.
    #[wasm_bindgen(method, getter, structural, js_name = eraYear)]
    pub fn era_year(this: &PlainDateTime) -> JsValue;

    /// In calendars that use eras, the era and eraYear read-only properties can be used
    /// together to resolve an era-relative year. Both properties are undefined when using
    /// the ISO 8601 calendar. As inputs to from or with, era and eraYear can be used instead
    /// of year. Unlike year, eraYear may decrease as time proceeds because some
    /// eras (like the BCE era in the Gregorian calendar) count years backwards.
    #[wasm_bindgen(method, getter, structural)]
    pub fn era(this: &PlainDateTime) -> JsValue;

    /// The dayOfWeek read-only property gives the weekday number that the date falls
    /// on. For the ISO 8601 calendar, the weekday number is defined as in the ISO 8601
    /// standard: a value between 1 and 7, inclusive, with Monday being 1, and Sunday 7.
    #[wasm_bindgen(method, getter, structural, js_name = dayOfWeek)]
    pub fn day_of_week(this: &PlainDateTime) -> u32;

    /// The dayOfYear read-only property gives the ordinal day of the year that the date
    /// falls on. For the ISO 8601 calendar, this is a value between 1 and 365, or 366 in
    /// a leap year.
    #[wasm_bindgen(method, getter, structural, js_name = dayOfYear)]
    pub fn day_of_year(this: &PlainDateTime) -> u32;

    /// The weekOfYear read-only property gives the ISO week number of the date. For the
    /// ISO 8601 calendar, this is normally a value between 1 and 52, but in a few cases it
    /// can be 53 as well. ISO week 1 is the week containing the first Thursday of the year.
    #[wasm_bindgen(method, getter, structural, js_name = weekOfYear)]
    pub fn week_of_year(this: &PlainDateTime) -> u32;

    /// The daysInWeek read-only property gives the number of days in the week that the
    /// date falls in. For the ISO 8601 calendar, this is always 7, but in other calendar
    /// systems it may differ from week to week.
    #[wasm_bindgen(method, getter, structural, js_name = daysInWeek)]
    pub fn days_in_week(this: &PlainDateTime) -> u32;

    /// The daysInMonth read-only property gives the number of days in the month that the
    /// date falls in. For the ISO 8601 calendar, this is 28, 29, 30, or 31, depending
    /// on the month and whether the year is a leap year.
    #[wasm_bindgen(method, getter, structural, js_name = daysInMonth)]
    pub fn days_in_month(this: &PlainDateTime) -> u32;

    /// The daysInYear read-only property gives the number of days in the year that the date
    /// falls in. For the ISO 8601 calendar, this is 365 or 366, depending on whether the year
    /// is a leap year.
    #[wasm_bindgen(method, getter, structural, js_name = daysInYear)]
    pub fn days_in_year(this: &PlainDateTime) -> u32;

    /// The monthsInYear read-only property gives the number of months in the year that the date
    /// falls in. For the ISO 8601 calendar, this is always 12, but in other calendar systems
    /// it may differ from year to year.
    #[wasm_bindgen(method, getter, structural, js_name = monthsInWeek)]
    pub fn months_in_year(this: &PlainDateTime) -> u32;

    /// The inLeapYear read-only property tells whether the year that the date falls in is a
    /// leap year or not. Its value is true if the year is a leap year, and false if not.
    #[wasm_bindgen(method, getter, structural, js_name = inLeapYear)]
    pub fn in_leap_year(this: &PlainDateTime) -> bool;

    /// This method creates a new Temporal.PlainDateTime which is a copy of zonedDateTime, but any
    /// properties present on parameter override the ones already present on zonedDateTime.
    #[wasm_bindgen(method, js_name = withPlainDate)]
    pub fn with(this: &PlainDateTime, parameter: &JsValue, options: &JsValue) -> PlainDateTime;

    /// Returns a new `Temporal.PlainDateTime` object which replaces the calendar date of zonedDateTime
    /// with the calendar date represented by plainDate.
    #[wasm_bindgen(method, js_name = withPlainDate)]
    pub fn with_plain_date(this: &PlainDateTime, date: &PlainDate) -> PlainDateTime;

    /// Returns a new `Temporal.PlainDateTime` object which is the date indicated by date, projected
    /// into calendar.
    #[wasm_bindgen(method, js_name = withCalendar)]
    pub fn with_calendar(this: &PlainDateTime, calendar: &str) -> PlainDateTime;

    /// This method adds duration to zonedDateTime.
    #[wasm_bindgen(method)]
    pub fn add(this: &PlainDateTime, duration: &Duration, options: &JsValue) -> PlainDateTime;

    /// This method subtracts duration to zonedDateTime.
    #[wasm_bindgen(method)]
    pub fn subtract(this: &PlainDateTime, duration: &Duration, options: &JsValue) -> PlainDateTime;

    /// This method computes the difference between the two times represented by zonedDateTime and
    /// other, optionally rounds it, and returns it as a Temporal.Duration object. If other is earlier
    /// than zonedDateTime then the resulting duration will be negative. The returned Temporal.Duration, when
    /// added to zonedDateTime with the same options, will yield other.
    #[wasm_bindgen(method)]
    pub fn until(this: &PlainDateTime, other: &PlainDateTime, options: &JsValue) -> Duration;

    /// This method computes the difference between the two times represented by zonedDateTime and
    /// other, optionally rounds it, and returns it as a Temporal.Duration object. If other is later
    /// than zonedDateTime then the resulting duration will be negative. The returned Temporal.Duration, when
    /// added to zonedDateTime with the same options, will yield other.    
    #[wasm_bindgen(method)]
    pub fn since(this: &PlainDateTime, other: &PlainDateTime, options: &JsValue) -> Duration;

    /// Returns a new Temporal.PlainDateTime object which is zonedDateTime rounded to `roundTo` (if
    /// a string parameter is used) or `roundingIncrement` of `smallestUnit` (if an object parameter
    /// is used).
    #[wasm_bindgen(method)]
    pub fn round(this: &PlainDateTime, round_to: &JsValue) -> PlainDateTime;

    #[wasm_bindgen(method)]
    pub fn equals(this: &PlainDateTime, other: &PlainDateTime) -> bool;

    /// Returns: a plain object with properties expressing zonedDateTime in the ISO 8601 calendar, including
    /// all date/time fields as well as the calendar, timeZone, and offset properties.
    ///
    /// This is an advanced method that's mainly useful if you are implementing a custom calendar. Most
    /// developers will not need to use it.
    #[wasm_bindgen(method, js_name = getISOFields)]
    pub fn get_iso_fields(this: &PlainDateTime) -> JsValue;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Temporal, extends = Object)]
    #[derive(Clone, Debug)]
    pub type ZonedDateTime;

    #[wasm_bindgen(static_method_of = ZonedDateTime, js_namespace = Temporal)]
    pub fn from(val: &JsValue) -> ZonedDateTime;

    #[wasm_bindgen(static_method_of = ZonedDateTime, js_namespace = Temporal)]
    pub fn compare(one: &ZonedDateTime, two: &ZonedDateTime) -> i32;

    #[wasm_bindgen(method, getter, structural)]
    pub fn year(this: &ZonedDateTime) -> i32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn month(this: &ZonedDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn week(this: &ZonedDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn day(this: &ZonedDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn hour(this: &ZonedDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn minute(this: &ZonedDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn second(this: &ZonedDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn millisecond(this: &ZonedDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn microsecond(this: &ZonedDateTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn nanosecond(this: &ZonedDateTime) -> u32;

    #[wasm_bindgen(method, getter, structural, js_name = epochSeconds)]
    pub fn epoch_seconds(this: &ZonedDateTime) -> u64;
    #[wasm_bindgen(method, getter, structural, js_name = epochMilliseconds)]
    pub fn epoch_milliseconds(this: &ZonedDateTime) -> u64;
    #[wasm_bindgen(method, getter, structural, js_name = epochMicroseconds)]
    pub fn epoch_microseconds(this: &ZonedDateTime) -> BigInt;
    #[wasm_bindgen(method, getter, structural, js_name = epochNanoseconds)]
    pub fn epoch_nanoseconds(this: &ZonedDateTime) -> BigInt;

    /// In calendars that use eras, the era and eraYear read-only properties can be used
    /// together to resolve an era-relative year. Both properties are undefined when using
    /// the ISO 8601 calendar. As inputs to from or with, era and eraYear can be used instead
    /// of year. Unlike year, eraYear may decrease as time proceeds because some
    /// eras (like the BCE era in the Gregorian calendar) count years backwards.
    #[wasm_bindgen(method, getter, structural, js_name = eraYear)]
    pub fn era_year(this: &ZonedDateTime) -> JsValue;

    /// In calendars that use eras, the era and eraYear read-only properties can be used
    /// together to resolve an era-relative year. Both properties are undefined when using
    /// the ISO 8601 calendar. As inputs to from or with, era and eraYear can be used instead
    /// of year. Unlike year, eraYear may decrease as time proceeds because some
    /// eras (like the BCE era in the Gregorian calendar) count years backwards.
    #[wasm_bindgen(method, getter, structural)]
    pub fn era(this: &ZonedDateTime) -> JsValue;

    /// The dayOfWeek read-only property gives the weekday number that the date falls
    /// on. For the ISO 8601 calendar, the weekday number is defined as in the ISO 8601
    /// standard: a value between 1 and 7, inclusive, with Monday being 1, and Sunday 7.
    #[wasm_bindgen(method, getter, structural, js_name = dayOfWeek)]
    pub fn day_of_week(this: &ZonedDateTime) -> u32;

    /// The dayOfYear read-only property gives the ordinal day of the year that the date
    /// falls on. For the ISO 8601 calendar, this is a value between 1 and 365, or 366 in
    /// a leap year.
    #[wasm_bindgen(method, getter, structural, js_name = dayOfYear)]
    pub fn day_of_year(this: &ZonedDateTime) -> u32;

    /// The weekOfYear read-only property gives the ISO week number of the date. For the
    /// ISO 8601 calendar, this is normally a value between 1 and 52, but in a few cases it
    /// can be 53 as well. ISO week 1 is the week containing the first Thursday of the year.
    #[wasm_bindgen(method, getter, structural, js_name = weekOfYear)]
    pub fn week_of_year(this: &ZonedDateTime) -> u32;

    /// The daysInWeek read-only property gives the number of days in the week that the
    /// date falls in. For the ISO 8601 calendar, this is always 7, but in other calendar
    /// systems it may differ from week to week.
    #[wasm_bindgen(method, getter, structural, js_name = daysInWeek)]
    pub fn days_in_week(this: &ZonedDateTime) -> u32;

    /// The daysInMonth read-only property gives the number of days in the month that the
    /// date falls in. For the ISO 8601 calendar, this is 28, 29, 30, or 31, depending
    /// on the month and whether the year is a leap year.
    #[wasm_bindgen(method, getter, structural, js_name = daysInMonth)]
    pub fn days_in_month(this: &ZonedDateTime) -> u32;

    /// The daysInYear read-only property gives the number of days in the year that the date
    /// falls in. For the ISO 8601 calendar, this is 365 or 366, depending on whether the year
    /// is a leap year.
    #[wasm_bindgen(method, getter, structural, js_name = daysInYear)]
    pub fn days_in_year(this: &ZonedDateTime) -> u32;

    /// The monthsInYear read-only property gives the number of months in the year that the date
    /// falls in. For the ISO 8601 calendar, this is always 12, but in other calendar systems
    /// it may differ from year to year.
    #[wasm_bindgen(method, getter, structural, js_name = monthsInWeek)]
    pub fn months_in_year(this: &ZonedDateTime) -> u32;

    /// The inLeapYear read-only property tells whether the year that the date falls in is a
    /// leap year or not. Its value is true if the year is a leap year, and false if not.
    #[wasm_bindgen(method, getter, structural, js_name = inLeapYear)]
    pub fn in_leap_year(this: &ZonedDateTime) -> bool;

    /// The `hoursInDay` read-only property returns the number of real-world hours
    /// between the start of the current day (usually midnight) in zonedDateTime.timeZone to
    /// the start of the next calendar day in the same time zone. Normally days will be 24 hours
    /// long, but on days where there are DST changes or other time zone transitions, this
    /// property may return 23 or 25. In rare cases, other integers or even non-integer
    /// values may be returned, e.g. when time zone definitions change by less than one hour.
    #[wasm_bindgen(method, getter, structural, js_name = inLeapYear)]
    pub fn hours_in_day(this: &ZonedDateTime) -> u32;

    #[wasm_bindgen(method, getter, structural, js_name = startOfDay)]
    pub fn start_of_day(this: &ZonedDateTime) -> ZonedDateTime;

    /// The offset read-only property is the offset (formatted as a string) relative
    /// to UTC of the current time zone and exact instant. Examples: '-08:00' or '+05:30'
    #[wasm_bindgen(method, getter, structural)]
    pub fn offset(this: &ZonedDateTime) -> JsString;

    /// This method creates a new Temporal.ZonedDateTime which is a copy of zonedDateTime, but any
    /// properties present on parameter override the ones already present on zonedDateTime.
    #[wasm_bindgen(method, js_name = withPlainDate)]
    pub fn with(this: &ZonedDateTime, parameter: &JsValue, options: &JsValue) -> ZonedDateTime;

    /// Returns a new `Temporal.ZonedDateTime` object which replaces the calendar date of zonedDateTime
    /// with the calendar date represented by plainDate.
    #[wasm_bindgen(method, js_name = withPlainDate)]
    pub fn with_plain_date(this: &ZonedDateTime, date: &PlainDate) -> ZonedDateTime;

    /// Returns a new `Temporal.ZonedDateTime` object which is the date indicated by date, projected
    /// into calendar.
    #[wasm_bindgen(method, js_name = withCalendar)]
    pub fn with_calendar(this: &ZonedDateTime, calendar: &str) -> ZonedDateTime;

    /// This method adds duration to zonedDateTime.
    #[wasm_bindgen(method)]
    pub fn add(this: &ZonedDateTime, duration: &Duration, options: &JsValue) -> ZonedDateTime;

    /// This method subtracts duration to zonedDateTime.
    #[wasm_bindgen(method)]
    pub fn subtract(this: &ZonedDateTime, duration: &Duration, options: &JsValue) -> ZonedDateTime;

    /// This method computes the difference between the two times represented by zonedDateTime and
    /// other, optionally rounds it, and returns it as a Temporal.Duration object. If other is earlier
    /// than zonedDateTime then the resulting duration will be negative. The returned Temporal.Duration, when
    /// added to zonedDateTime with the same options, will yield other.
    #[wasm_bindgen(method)]
    pub fn until(this: &ZonedDateTime, other: &ZonedDateTime, options: &JsValue) -> Duration;

    /// This method computes the difference between the two times represented by zonedDateTime and
    /// other, optionally rounds it, and returns it as a Temporal.Duration object. If other is later
    /// than zonedDateTime then the resulting duration will be negative. The returned Temporal.Duration, when
    /// added to zonedDateTime with the same options, will yield other.    
    #[wasm_bindgen(method)]
    pub fn since(this: &ZonedDateTime, other: &ZonedDateTime, options: &JsValue) -> Duration;

    /// Returns a new Temporal.ZonedDateTime object which is zonedDateTime rounded to `roundTo` (if
    /// a string parameter is used) or `roundingIncrement` of `smallestUnit` (if an object parameter
    /// is used).
    #[wasm_bindgen(method)]
    pub fn round(this: &ZonedDateTime, round_to: &JsValue) -> ZonedDateTime;

    #[wasm_bindgen(method)]
    pub fn equals(this: &ZonedDateTime, other: &ZonedDateTime) -> bool;

    /// Returns: a plain object with properties expressing zonedDateTime in the ISO 8601 calendar, including
    /// all date/time fields as well as the calendar, timeZone, and offset properties.
    ///
    /// This is an advanced method that's mainly useful if you are implementing a custom calendar. Most
    /// developers will not need to use it.
    #[wasm_bindgen(method, js_name = getISOFields)]
    pub fn get_iso_fields(this: &ZonedDateTime) -> JsValue;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Temporal, extends = Object)]
    #[derive(Clone, Debug)]
    pub type Duration;

    #[wasm_bindgen(static_method_of = Duration, js_namespace = Temporal)]
    pub fn from(val: &JsValue) -> Duration;

    #[wasm_bindgen(static_method_of = Duration, js_namespace = Temporal)]
    pub fn compare(one: &Duration, two: &Duration) -> i32;

    #[wasm_bindgen(method, getter, structural)]
    pub fn years(this: &Duration) -> i32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn months(this: &Duration) -> i32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn weeks(this: &Duration) -> i32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn days(this: &Duration) -> i32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn hours(this: &Duration) -> i32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn minutes(this: &Duration) -> i32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn seconds(this: &Duration) -> i32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn milliseconds(this: &Duration) -> i32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn microseconds(this: &Duration) -> i32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn nanoseconds(this: &Duration) -> i32;

    /// The read-only `sign` property has the value –1, 0, or 1, depending on whether
    /// the duration is negative, zero, or positive.
    #[wasm_bindgen(method, getter, structural)]
    pub fn sign(this: &Duration) -> i32;

    /// The read-only `blank` property is a convenience property that tells whether duration
    /// represents a zero length of time. In other words, `duration.blank === (duration.sign === 0)`.
    #[wasm_bindgen(method, getter, structural)]
    pub fn blank(this: &Duration) -> bool;

    /// In order to be valid, the resulting duration must not have fields with mixed signs, and
    /// so the result is balanced. For usage examples and a more complete explanation of how
    /// balancing works and why it is necessary, see [Duration balancing].
    ///
    /// [Duration balancing]: https://tc39.es/proposal-temporal/docs/balancing.html
    #[wasm_bindgen(method)]
    pub fn add(this: &Duration, other: &Duration, options: &JsValue) -> Duration;

    /// In order to be valid, the resulting duration must not have fields with mixed signs, and
    /// so the result is balanced. For usage examples and a more complete explanation of how
    /// balancing works and why it is necessary, see [Duration balancing].
    ///
    /// [Duration balancing]: https://tc39.es/proposal-temporal/docs/balancing.html
    #[wasm_bindgen(method)]
    pub fn subtract(this: &Duration, other: &Duration, options: &JsValue) -> Duration;

    /// This method gives the absolute value of duration. It returns a newly constructed
    /// `Temporal.Duration` with all the fields having the same magnitude as those of
    /// duration, but positive. If duration is already positive or zero, then the returned
    /// object is a copy of duration.
    #[wasm_bindgen(method)]
    pub fn abs(this: &Duration) -> Duration;

    /// Rounds and/or balances duration to the given largest and smallest units and rounding
    /// increment, and returns the result as a new `Temporal.Duration` object.
    #[wasm_bindgen(method)]
    pub fn round(this: &Duration, round_to: &JsValue) -> Duration;

    /// Calculates the number of units of time that can fit in a particular `Temporal.Duration`.
    /// If the duration IS NOT evenly divisible by the desired unit, then a fractional remainder
    /// will be present in the result. If the duration IS evenly divisible by the desired
    /// unit, then the integer result will be identical
    /// to `duration.round({ smallestUnit: unit, largestUnit: unit, relativeTo })[unit]`
    #[wasm_bindgen(method)]
    pub fn total(this: &Duration, total_of: &JsValue) -> f64;

    /// This method overrides `Object.prototype.toString()` and provides the ISO 8601
    /// description of the duration.
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &Duration, options: &JsValue) -> JsString;

    /// This method is the same as duration.toString(). It is usually not called directly, but
    /// it can be called automatically by JSON.stringify().
    #[wasm_bindgen(method, js_name = toJson)]
    pub fn to_json(this: &Duration) -> JsString;

    /// This method overrides `Object.prototype.toLocaleString()` to provide a human-readable,
    /// language-sensitive representation of duration.
    ///
    /// The locales and options arguments are the same as in the constructor to Intl.DurationFormat.
    #[wasm_bindgen(method, js_name = toLocaleString)]
    pub fn to_locale_string(this: &Duration, locales: &JsValue, options: &JsValue) -> JsString;

}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Temporal, extends = Object)]
    #[derive(Clone, Debug)]
    pub type PlainTime;

    #[wasm_bindgen(static_method_of = PlainTime, js_namespace = Temporal)]
    pub fn from(val: &JsValue) -> PlainTime;

    #[wasm_bindgen(static_method_of = PlainTime, js_namespace = Temporal)]
    pub fn compare(one: &PlainTime, two: &PlainTime) -> i32;

    #[wasm_bindgen(method, getter, structural)]
    pub fn hour(this: &PlainTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn minute(this: &PlainTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn second(this: &PlainTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn millisecond(this: &PlainTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn microsecond(this: &PlainTime) -> u32;
    #[wasm_bindgen(method, getter, structural)]
    pub fn nanosecond(this: &PlainTime) -> u32;

    /// This method adds duration to PlainTime.
    #[wasm_bindgen(method)]
    pub fn add(this: &PlainTime, duration: &Duration, options: &JsValue) -> PlainTime;

    /// This method subtracts duration to PlainTime.
    #[wasm_bindgen(method)]
    pub fn subtract(this: &PlainTime, duration: &Duration, options: &JsValue) -> PlainTime;

    /// This method computes the difference between the two times represented by PlainTime and
    /// other, optionally rounds it, and returns it as a Temporal.Duration object. If other is earlier
    /// than zonedDateTime then the resulting duration will be negative. The returned Temporal.Duration, when
    /// added to zonedDateTime with the same options, will yield other.
    #[wasm_bindgen(method)]
    pub fn until(this: &PlainTime, other: &PlainTime, options: &JsValue) -> Duration;

    /// This method computes the difference between the two times represented by PlainTime and
    /// other, optionally rounds it, and returns it as a Temporal.Duration object. If other is later
    /// than zonedDateTime then the resulting duration will be negative. The returned Temporal.Duration, when
    /// added to zonedDateTime with the same options, will yield other.    
    #[wasm_bindgen(method)]
    pub fn since(this: &PlainTime, other: &PlainTime, options: &JsValue) -> Duration;

    /// Returns a new Temporal.ZonedDateTime object which is PlainTime rounded to `roundTo` (if
    /// a string parameter is used) or `roundingIncrement` of `smallestUnit` (if an object parameter
    /// is used).
    #[wasm_bindgen(method)]
    pub fn round(this: &PlainTime, round_to: &JsValue) -> PlainTime;

    #[wasm_bindgen(method)]
    pub fn equals(this: &PlainTime, other: &PlainTime) -> bool;

    /// This method overrides `Object.prototype.toLocaleString()` to provide a human-readable,
    /// language-sensitive representation of duration.
    ///
    /// The locales and options arguments are the same as in the constructor to Intl.DateTimeFormat.
    #[wasm_bindgen(method, js_name = toLocaleString)]
    pub fn to_locale_string(this: &PlainTime, locales: &JsValue, options: &JsValue) -> JsString;

    /// Returns: a plain object with properties expressing PlainTime in the ISO 8601 calendar, including
    /// all date/time fields as well as the calendar, timeZone, and offset properties.
    ///
    /// This is an advanced method that's mainly useful if you are implementing a custom calendar. Most
    /// developers will not need to use it.
    #[wasm_bindgen(method, js_name = getISOFields)]
    pub fn get_iso_fields(this: &PlainTime) -> JsValue;

}

#[wasm_bindgen]
extern "C" {
    /// A Temporal.PlainYearMonth represents a particular month on the calendar. For example, it
    /// could be used to represent a particular instance of a monthly recurring event, like "the
    /// June 2019 meeting".
    #[wasm_bindgen(js_namespace = Temporal, extends = Object)]
    #[derive(Clone, Debug)]
    pub type PlainYearMonth;

    #[wasm_bindgen(static_method_of = PlainYearMonth, js_namespace = Temporal)]
    pub fn from(val: &JsValue) -> PlainYearMonth;

    #[wasm_bindgen(static_method_of = PlainYearMonth, js_namespace = Temporal)]
    pub fn compare(one: &PlainYearMonth, two: &PlainYearMonth) -> i32;

    /// `year` is a signed integer representing the number of years relative to a
    /// calendar-specific epoch. For calendars that use eras, the anchor is usually
    /// aligned with the latest era so that eraYear === year for all dates in that
    /// era. However, some calendars like Japanese may use a different anchor.
    #[wasm_bindgen(method, getter, structural)]
    pub fn year(this: &PlainYearMonth) -> i32;

    /// `month` is a positive integer representing the ordinal index of the month in
    /// the current year. For calendars like Hebrew or Chinese that use leap
    /// months, the same-named month may have a different month value depending on
    /// the year. The first month in every year has month equal to 1. The last month
    /// of every year has month equal to the monthsInYear property. month values
    /// start at 1, which is different from legacy Date where months are represented
    /// by zero-based indices (0 to 11).
    #[wasm_bindgen(method, getter, structural)]
    pub fn month(this: &PlainYearMonth) -> u32;

    /// `monthCode` is a calendar-specific string that identifies the month in
    /// a year-independent way. For common (non-leap) months, monthCode should
    /// be `M${month}`, where month is zero padded up to two digits. For uncommon
    /// (leap) months in lunisolar calendars like Hebrew or Chinese, the month code
    /// is the previous month's code with with an "L" suffix appended.
    /// Examples:
    /// * 'M02' => February;
    /// * 'M08L' => repeated 8th month in the Chinese calendar;
    /// * 'M05L' => Adar I in the Hebrew calendar.
    #[wasm_bindgen(method, getter, structural, js_name = monthCode)]
    pub fn month_code(this: &PlainYearMonth) -> JsString;

    /// In calendars that use eras, the era and eraYear read-only properties can be used
    /// together to resolve an era-relative year. Both properties are undefined when using
    /// the ISO 8601 calendar. As inputs to from or with, era and eraYear can be used instead
    /// of year. Unlike year, eraYear may decrease as time proceeds because some
    /// eras (like the BCE era in the Gregorian calendar) count years backwards.
    #[wasm_bindgen(method, getter, structural, js_name = eraYear)]
    pub fn era_year(this: &PlainYearMonth) -> JsValue;

    /// In calendars that use eras, the era and eraYear read-only properties can be used
    /// together to resolve an era-relative year. Both properties are undefined when using
    /// the ISO 8601 calendar. As inputs to from or with, era and eraYear can be used instead
    /// of year. Unlike year, eraYear may decrease as time proceeds because some
    /// eras (like the BCE era in the Gregorian calendar) count years backwards.
    #[wasm_bindgen(method, getter, structural)]
    pub fn era(this: &PlainYearMonth) -> JsValue;

    /// The daysInMonth read-only property gives the number of days in the month that the
    /// date falls in. For the ISO 8601 calendar, this is 28, 29, 30, or 31, depending
    /// on the month and whether the year is a leap year.
    #[wasm_bindgen(method, getter, structural, js_name = daysInMonth)]
    pub fn days_in_month(this: &PlainYearMonth) -> u32;

    /// The daysInYear read-only property gives the number of days in the year that the date
    /// falls in. For the ISO 8601 calendar, this is 365 or 366, depending on whether the year
    /// is a leap year.
    #[wasm_bindgen(method, getter, structural, js_name = daysInYear)]
    pub fn days_in_year(this: &PlainYearMonth) -> u32;

    /// The monthsInYear read-only property gives the number of months in the year that the date
    /// falls in. For the ISO 8601 calendar, this is always 12, but in other calendar systems
    /// it may differ from year to year.
    #[wasm_bindgen(method, getter, structural, js_name = monthsInWeek)]
    pub fn months_in_year(this: &PlainYearMonth) -> u32;

    /// The inLeapYear read-only property tells whether the year that the date falls in is a
    /// leap year or not. Its value is true if the year is a leap year, and false if not.
    #[wasm_bindgen(method, getter, structural, js_name = inLeapYear)]
    pub fn in_leap_year(this: &PlainYearMonth) -> bool;

    /// **Parameters:**
    ///
    ///- `duration` (`Temporal.Duration` or value convertible to one): The duration to add.
    ///- `options` (optional object): An object with properties representing options for the addition.
    ///The following options are recognized:
    ///- `overflow` (optional string): How to deal with additions that result in out-of-range values.
    ///    Allowed values are `constrain` and `reject`.
    ///    The default is `constrain`.
    ///
    ///**Returns:** a new `Temporal.PlainYearMonth` object which is the date indicated by `date` plus `duration`.
    ///
    ///This method adds `duration` to `date`, returning a date that is in the future relative to `date`.
    ///
    ///The `duration` argument is an object with properties denoting a duration, such as `{ days: 5 }`, or a string such as `P5D`, or a `Temporal.Duration` object.
    ///If `duration` is not a `Temporal.Duration` object, then it will be converted to one as if it were passed to `Temporal.Duration.from()`.
    ///
    ///Some additions may be ambiguous, because months have different lengths.
    ///For example, adding one month to August 31 would result in September 31, which doesn't exist.
    ///For these cases, the `overflow` option tells what to do:
    ///
    ///- In `constrain` mode (the default), out-of-range values are clamped to the nearest in-range value.
    ///- In `reject` mode, an addition that would result in an out-of-range value fails, and a `RangeError` is thrown.
    ///
    ///Additionally, if the result is earlier or later than the range of dates that `Temporal.PlainYearMonth` can represent (approximately half a million years centered on the [Unix epoch](https://en.wikipedia.org/wiki/Unix_time)), then this method will throw a `RangeError` regardless of `overflow`.
    ///
    ///Adding a negative duration is equivalent to subtracting the absolute value of that duration.
    #[wasm_bindgen(method)]
    pub fn add(this: &PlainYearMonth, duration: &Duration, options: &JsValue) -> PlainYearMonth;

    /// **Parameters:**
    ///- `duration` (`Temporal.Duration` or value convertible to one): The duration to subtract.
    ///- `options` (optional object): An object with properties representing options for the subtraction.
    ///The following options are recognized:
    ///- `overflow` (string): How to deal with subtractions that result in out-of-range values.
    ///    Allowed values are `constrain` and `reject`.
    ///    The default is `constrain`.
    ///
    ///**Returns:** a new `Temporal.PlainYearMonth` object which is the date indicated by `date` minus `duration`.
    ///
    ///This method subtracts `duration` from `date`, returning a date that is in the past relative to `date`.
    ///
    ///The `duration` argument is an object with properties denoting a duration, such as `{ days: 5 }`, or a string such as `P5D`, or a `Temporal.Duration` object.
    ///If `duration` is not a `Temporal.Duration` object, then it will be converted to one as if it were passed to `Temporal.Duration.from()`.
    ///
    ///Some subtractions may be ambiguous, because months have different lengths.
    ///For example, subtracting one month from July 31 would result in June 31, which doesn't exist.
    ///For these cases, the `overflow` option tells what to do:
    ///
    ///- In `constrain` mode (the default), out-of-range values are clamped to the nearest in-range value.
    ///- In `reject` mode, an addition that would result in an out-of-range value fails, and a `RangeError` is thrown.
    ///
    ///Additionally, if the result is earlier or later than the range of dates that `Temporal.PlainDate` can represent (approximately half a million years centered on the [Unix epoch](https://en.wikipedia.org/wiki/Unix_time)), then this method will throw a `RangeError` regardless of `overflow`.
    ///
    ///Subtracting a negative duration is equivalent to adding the absolute value of that duration.
    ///
    #[wasm_bindgen(method)]
    pub fn subtract(
        this: &PlainYearMonth,
        duration: &Duration,
        options: &JsValue,
    ) -> PlainYearMonth;

    /// This function exists because it's not possible to compare using date == other
    /// or date === other, due to ambiguity in the primitive representation and between
    /// Temporal types.
    ///
    /// If you don't need to know the order in which the two dates occur, then this function
    /// may be less typing and more efficient than Temporal.PlainDate.compare.
    #[wasm_bindgen(method)]
    pub fn equals(this: &PlainYearMonth, other: &PlainYearMonth) -> bool;

    /// This method overrides the Object.prototype.toString() method and provides a convenient, unambiguous
    /// string representation of date. The string can be passed to Temporal.PlainYearMonth.from() to
    /// create a new Temporal.PlainDate object.
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &PlainYearMonth) -> JsString;

    /// This method overrides `Object.prototype.toLocaleString()` to provide a human-readable,
    /// language-sensitive representation of duration.
    ///
    /// The locales and options arguments are the same as in the constructor to Intl.DateTimeFormat.
    #[wasm_bindgen(method, js_name = toLocaleString)]
    pub fn to_locale_string(
        this: &PlainYearMonth,
        locales: &JsValue,
        options: &JsValue,
    ) -> JsString;

    /// Returns: a plain object with properties expressing date in the ISO 8601
    /// calendar, as well as the value of date.calendar.
    ///
    /// This method is mainly useful if you are implementing a custom calendar. Most
    /// code will not need to use it.
    #[wasm_bindgen(method, js_name = getISOFields)]
    pub fn get_iso_fields(this: &PlainYearMonth) -> JsValue;

}

#[wasm_bindgen]
extern "C" {
    /// A date without a year component. This is useful to express
    /// things like "Bastille Day is on the 14th of July".
    #[wasm_bindgen(js_namespace = Temporal, extends = Object)]
    #[derive(Clone, Debug)]
    pub type PlainMonthDay;

    #[wasm_bindgen(static_method_of = PlainMonthDay, js_namespace = Temporal)]
    pub fn from(val: &JsValue) -> PlainMonthDay;

    #[wasm_bindgen(static_method_of = PlainMonthDay, js_namespace = Temporal)]
    pub fn compare(one: &PlainMonthDay, two: &PlainMonthDay) -> i32;

    /// `monthCode` is a calendar-specific string that identifies the month in
    /// a year-independent way. For common (non-leap) months, monthCode should
    /// be `M${month}`, where month is zero padded up to two digits. For uncommon
    /// (leap) months in lunisolar calendars like Hebrew or Chinese, the month code
    /// is the previous month's code with with an "L" suffix appended.
    /// Examples:
    /// * 'M02' => February;
    /// * 'M08L' => repeated 8th month in the Chinese calendar;
    /// * 'M05L' => Adar I in the Hebrew calendar.
    ///
    /// Note that this type has no month property, because month is ambiguous for some
    /// calendars without knowing the year. Instead, the monthCode property is used which
    /// is year-independent in all calendars.
    #[wasm_bindgen(method, getter, structural, js_name = monthCode)]
    pub fn month_code(this: &PlainMonthDay) -> JsString;

    /// day is a positive integer representing the day of the month.
    #[wasm_bindgen(method, getter, structural)]
    pub fn day(this: &PlainMonthDay) -> u32;

    /// This function exists because it's not possible to compare using date == other
    /// or date === other, due to ambiguity in the primitive representation and between
    /// Temporal types.
    ///
    /// If you don't need to know the order in which the two dates occur, then this function
    /// may be less typing and more efficient than Temporal.PlainMonthDay.compare.
    ///
    /// Note that this function will return false if the two objects have different calendar
    /// properties, even if the actual dates are equal.
    #[wasm_bindgen(method)]
    pub fn equals(this: &PlainMonthDay, other: &PlainMonthDay) -> bool;

    /// This method overrides the Object.prototype.toString() method and provides a convenient, unambiguous
    /// string representation of date. The string can be passed to Temporal.PlainMonthDay.from() to
    /// create a new Temporal.PlainMonthDay object.
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &PlainMonthDay) -> JsString;

    /// This method overrides `Object.prototype.toLocaleString()` to provide a human-readable,
    /// language-sensitive representation of duration.
    ///
    /// The locales and options arguments are the same as in the constructor to Intl.DateTimeFormat.
    #[wasm_bindgen(method, js_name = toLocaleString)]
    pub fn to_locale_string(this: &PlainMonthDay, locales: &JsValue, options: &JsValue)
        -> JsString;

    /// Returns: a plain object with properties expressing date in the ISO 8601
    /// calendar, as well as the value of date.calendar.
    ///
    /// This method is mainly useful if you are implementing a custom calendar. Most
    /// code will not need to use it.
    #[wasm_bindgen(method, js_name = getISOFields)]
    pub fn get_iso_fields(this: &PlainMonthDay) -> JsValue;

}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Temporal, extends = Object)]
    #[derive(Clone, Debug)]
    pub type PlainDate;

    #[wasm_bindgen(js_namespace = Temporal, constructor)]
    pub fn new(year: i32, month: u32, day: u32) -> PlainDate;

    #[wasm_bindgen(static_method_of = PlainDate, js_namespace = Temporal)]
    pub fn from(val: &JsValue) -> PlainDate;

    #[wasm_bindgen(static_method_of = PlainDate, js_namespace = Temporal)]
    pub fn compare(one: &PlainDate, two: &PlainDate) -> i32;

    /// `year` is a signed integer representing the number of years relative to a
    /// calendar-specific epoch. For calendars that use eras, the anchor is usually
    /// aligned with the latest era so that eraYear === year for all dates in that
    /// era. However, some calendars like Japanese may use a different anchor.
    #[wasm_bindgen(method, getter, structural)]
    pub fn year(this: &PlainDate) -> i32;

    /// `month` is a positive integer representing the ordinal index of the month in
    /// the current year. For calendars like Hebrew or Chinese that use leap
    /// months, the same-named month may have a different month value depending on
    /// the year. The first month in every year has month equal to 1. The last month
    /// of every year has month equal to the monthsInYear property. month values
    /// start at 1, which is different from legacy Date where months are represented
    /// by zero-based indices (0 to 11).
    #[wasm_bindgen(method, getter, structural)]
    pub fn month(this: &PlainDate) -> u32;

    /// `monthCode` is a calendar-specific string that identifies the month in
    /// a year-independent way. For common (non-leap) months, monthCode should
    /// be `M${month}`, where month is zero padded up to two digits. For uncommon
    /// (leap) months in lunisolar calendars like Hebrew or Chinese, the month code
    /// is the previous month's code with with an "L" suffix appended.
    /// Examples:
    /// * 'M02' => February;
    /// * 'M08L' => repeated 8th month in the Chinese calendar;
    /// * 'M05L' => Adar I in the Hebrew calendar.
    #[wasm_bindgen(method, getter, structural, js_name = monthCode)]
    pub fn month_code(this: &PlainDate) -> JsString;

    /// day is a positive integer representing the day of the month.
    #[wasm_bindgen(method, getter, structural)]
    pub fn day(this: &PlainDate) -> u32;

    /// In calendars that use eras, the era and eraYear read-only properties can be used
    /// together to resolve an era-relative year. Both properties are undefined when using
    /// the ISO 8601 calendar. As inputs to from or with, era and eraYear can be used instead
    /// of year. Unlike year, eraYear may decrease as time proceeds because some
    /// eras (like the BCE era in the Gregorian calendar) count years backwards.
    #[wasm_bindgen(method, getter, structural, js_name = eraYear)]
    pub fn era_year(this: &PlainDate) -> JsValue;

    /// In calendars that use eras, the era and eraYear read-only properties can be used
    /// together to resolve an era-relative year. Both properties are undefined when using
    /// the ISO 8601 calendar. As inputs to from or with, era and eraYear can be used instead
    /// of year. Unlike year, eraYear may decrease as time proceeds because some
    /// eras (like the BCE era in the Gregorian calendar) count years backwards.
    #[wasm_bindgen(method, getter, structural)]
    pub fn era(this: &PlainDate) -> JsValue;

    /// The dayOfWeek read-only property gives the weekday number that the date falls
    /// on. For the ISO 8601 calendar, the weekday number is defined as in the ISO 8601
    /// standard: a value between 1 and 7, inclusive, with Monday being 1, and Sunday 7.
    #[wasm_bindgen(method, getter, structural, js_name = dayOfWeek)]
    pub fn day_of_week(this: &PlainDate) -> u32;

    /// The dayOfYear read-only property gives the ordinal day of the year that the date
    /// falls on. For the ISO 8601 calendar, this is a value between 1 and 365, or 366 in
    /// a leap year.
    #[wasm_bindgen(method, getter, structural, js_name = dayOfYear)]
    pub fn day_of_year(this: &PlainDate) -> u32;

    /// The weekOfYear read-only property gives the ISO week number of the date. For the
    /// ISO 8601 calendar, this is normally a value between 1 and 52, but in a few cases it
    /// can be 53 as well. ISO week 1 is the week containing the first Thursday of the year.
    #[wasm_bindgen(method, getter, structural, js_name = weekOfYear)]
    pub fn week_of_year(this: &PlainDate) -> u32;

    /// The daysInWeek read-only property gives the number of days in the week that the
    /// date falls in. For the ISO 8601 calendar, this is always 7, but in other calendar
    /// systems it may differ from week to week.
    #[wasm_bindgen(method, getter, structural, js_name = daysInWeek)]
    pub fn days_in_week(this: &PlainDate) -> u32;

    /// The daysInMonth read-only property gives the number of days in the month that the
    /// date falls in. For the ISO 8601 calendar, this is 28, 29, 30, or 31, depending
    /// on the month and whether the year is a leap year.
    #[wasm_bindgen(method, getter, structural, js_name = daysInMonth)]
    pub fn days_in_month(this: &PlainDate) -> u32;

    /// The daysInYear read-only property gives the number of days in the year that the date
    /// falls in. For the ISO 8601 calendar, this is 365 or 366, depending on whether the year
    /// is a leap year.
    #[wasm_bindgen(method, getter, structural, js_name = daysInYear)]
    pub fn days_in_year(this: &PlainDate) -> u32;

    /// The monthsInYear read-only property gives the number of months in the year that the date
    /// falls in. For the ISO 8601 calendar, this is always 12, but in other calendar systems
    /// it may differ from year to year.
    #[wasm_bindgen(method, getter, structural, js_name = monthsInWeek)]
    pub fn months_in_year(this: &PlainDate) -> u32;

    /// The inLeapYear read-only property tells whether the year that the date falls in is a
    /// leap year or not. Its value is true if the year is a leap year, and false if not.
    #[wasm_bindgen(method, getter, structural, js_name = inLeapYear)]
    pub fn in_leap_year(this: &PlainDate) -> bool;

    /// Returns a new `Temporal.PlainDate` object which is the date indicated by date, projected
    /// into calendar.
    #[wasm_bindgen(method, js_name = withCalendar)]
    pub fn with_calendar(this: &PlainDate, calendar: &str) -> PlainDate;

    /// **Parameters:**
    ///
    ///- `duration` (`Temporal.Duration` or value convertible to one): The duration to add.
    ///- `options` (optional object): An object with properties representing options for the addition.
    ///The following options are recognized:
    ///- `overflow` (optional string): How to deal with additions that result in out-of-range values.
    ///    Allowed values are `constrain` and `reject`.
    ///    The default is `constrain`.
    ///
    ///**Returns:** a new `Temporal.PlainDate` object which is the date indicated by `date` plus `duration`.
    ///
    ///This method adds `duration` to `date`, returning a date that is in the future relative to `date`.
    ///
    ///The `duration` argument is an object with properties denoting a duration, such as `{ days: 5 }`, or a string such as `P5D`, or a `Temporal.Duration` object.
    ///If `duration` is not a `Temporal.Duration` object, then it will be converted to one as if it were passed to `Temporal.Duration.from()`.
    ///
    ///Some additions may be ambiguous, because months have different lengths.
    ///For example, adding one month to August 31 would result in September 31, which doesn't exist.
    ///For these cases, the `overflow` option tells what to do:
    ///
    ///- In `constrain` mode (the default), out-of-range values are clamped to the nearest in-range value.
    ///- In `reject` mode, an addition that would result in an out-of-range value fails, and a `RangeError` is thrown.
    ///
    ///Additionally, if the result is earlier or later than the range of dates that `Temporal.PlainDate` can represent (approximately half a million years centered on the [Unix epoch](https://en.wikipedia.org/wiki/Unix_time)), then this method will throw a `RangeError` regardless of `overflow`.
    ///
    ///Adding a negative duration is equivalent to subtracting the absolute value of that duration.
    #[wasm_bindgen(method)]
    pub fn add(this: &PlainDate, duration: &Duration, options: &JsValue) -> PlainDate;

    /// **Parameters:**
    ///- `duration` (`Temporal.Duration` or value convertible to one): The duration to subtract.
    ///- `options` (optional object): An object with properties representing options for the subtraction.
    ///The following options are recognized:
    ///- `overflow` (string): How to deal with subtractions that result in out-of-range values.
    ///    Allowed values are `constrain` and `reject`.
    ///    The default is `constrain`.
    ///
    ///**Returns:** a new `Temporal.PlainDate` object which is the date indicated by `date` minus `duration`.
    ///
    ///This method subtracts `duration` from `date`, returning a date that is in the past relative to `date`.
    ///
    ///The `duration` argument is an object with properties denoting a duration, such as `{ days: 5 }`, or a string such as `P5D`, or a `Temporal.Duration` object.
    ///If `duration` is not a `Temporal.Duration` object, then it will be converted to one as if it were passed to `Temporal.Duration.from()`.
    ///
    ///Some subtractions may be ambiguous, because months have different lengths.
    ///For example, subtracting one month from July 31 would result in June 31, which doesn't exist.
    ///For these cases, the `overflow` option tells what to do:
    ///
    ///- In `constrain` mode (the default), out-of-range values are clamped to the nearest in-range value.
    ///- In `reject` mode, an addition that would result in an out-of-range value fails, and a `RangeError` is thrown.
    ///
    ///Additionally, if the result is earlier or later than the range of dates that `Temporal.PlainDate` can represent (approximately half a million years centered on the [Unix epoch](https://en.wikipedia.org/wiki/Unix_time)), then this method will throw a `RangeError` regardless of `overflow`.
    ///
    ///Subtracting a negative duration is equivalent to adding the absolute value of that duration.
    ///
    #[wasm_bindgen(method)]
    pub fn subtract(this: &PlainDate, duration: &Duration, options: &JsValue) -> PlainDate;

    /// This function exists because it's not possible to compare using date == other
    /// or date === other, due to ambiguity in the primitive representation and between
    /// Temporal types.
    ///
    /// If you don't need to know the order in which the two dates occur, then this function
    /// may be less typing and more efficient than Temporal.PlainDate.compare.
    ///
    /// Note that this function will return false if the two objects have different calendar
    /// properties, even if the actual dates are equal.
    #[wasm_bindgen(method)]
    pub fn equals(this: &PlainDate, other: &PlainDate) -> bool;

    /// This method overrides the Object.prototype.toString() method and provides a convenient, unambiguous
    /// string representation of date. The string can be passed to Temporal.PlainDate.from() to
    /// create a new Temporal.PlainDate object.
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &PlainDate) -> JsString;

    /// This method overrides `Object.prototype.toLocaleString()` to provide a human-readable,
    /// language-sensitive representation of duration.
    ///
    /// The locales and options arguments are the same as in the constructor to Intl.DateTimeFormat.
    #[wasm_bindgen(method, js_name = toLocaleString)]
    pub fn to_locale_string(this: &PlainDate, locales: &JsValue, options: &JsValue) -> JsString;

    /// Returns: a plain object with properties expressing date in the ISO 8601
    /// calendar, as well as the value of date.calendar.
    ///
    /// This method is mainly useful if you are implementing a custom calendar. Most
    /// code will not need to use it.
    #[wasm_bindgen(method, js_name = getISOFields)]
    pub fn get_iso_fields(this: &PlainDate) -> JsValue;

}

pub mod Now {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["Temporal", "Now"])]
        pub fn instant() -> Instant;

        #[wasm_bindgen(js_namespace = ["Temporal", "Now"], js_name = zonedDateTime)]
        pub fn zoned_date_time(calendar: &str) -> ZonedDateTime;

        #[wasm_bindgen(js_namespace = ["Temporal", "Now"], js_name = zonedDateTimeISO)]
        pub fn zoned_date_time_iso() -> ZonedDateTime;

        #[wasm_bindgen(js_namespace = ["Temporal", "Now"], js_name = plainDate)]
        pub fn plain_date(calendar: &str) -> PlainDate;

        #[wasm_bindgen(js_namespace = ["Temporal", "Now"], js_name = plainDateISO)]
        pub fn plain_date_iso() -> PlainDate;

        #[wasm_bindgen(js_namespace = ["Temporal", "Now"], js_name = plainTimeISO)]
        pub fn plain_time_iso() -> PlainTime;

    }
}
