use proc_macro::{Ident, Span, TokenStream};

use super::modifier;
use crate::to_tokens::ToTokenStream;

pub(crate) enum Component {
    Day(modifier::Day),
    Month(modifier::Month),
    Ordinal(modifier::Ordinal),
    Weekday(modifier::Weekday),
    WeekNumber(modifier::WeekNumber),
    Year(modifier::Year),
    Hour(modifier::Hour),
    Minute(modifier::Minute),
    Period(modifier::Period),
    Second(modifier::Second),
    Subsecond(modifier::Subsecond),
    OffsetHour(modifier::OffsetHour),
    OffsetMinute(modifier::OffsetMinute),
    OffsetSecond(modifier::OffsetSecond),
}

impl ToTokenStream for Component {
    fn append_to(self, ts: &mut TokenStream) {
        let mut mts = TokenStream::new();

        macro_rules! component_name_and_append {
            ($($name:ident)*) => {
                match self {
                    $(Self::$name(modifier) => {
                        modifier.append_to(&mut mts);
                        stringify!($name)
                    })*
                }
            };
        }

        let component = component_name_and_append![
            Day
            Month
            Ordinal
            Weekday
            WeekNumber
            Year
            Hour
            Minute
            Period
            Second
            Subsecond
            OffsetHour
            OffsetMinute
            OffsetSecond
        ];
        let component = Ident::new(component, Span::mixed_site());

        quote_append! { ts
            ::time::format_description::Component::#(component)(#S(mts))
        }
    }
}
