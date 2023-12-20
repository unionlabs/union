use std::num::NonZeroU16;

use proc_macro::{Ident, Span, TokenStream, TokenTree};

use crate::to_tokens::{ToTokenStream, ToTokenTree};

macro_rules! to_tokens {
    (
        $(#[$struct_attr:meta])*
        $struct_vis:vis struct $struct_name:ident {$(
            $(#[$field_attr:meta])*
            $field_vis:vis $field_name:ident : $field_ty:ty
        ),* $(,)?}
    ) => {
        $(#[$struct_attr])*
        $struct_vis struct $struct_name {$(
            $(#[$field_attr])*
            $field_vis $field_name: $field_ty
        ),*}

        impl ToTokenTree for $struct_name {
            fn into_token_tree(self) -> TokenTree {
                let mut tokens = TokenStream::new();
                let Self {$($field_name),*} = self;

                quote_append! { tokens
                    let mut value = ::time::format_description::modifier::$struct_name::default();
                };
                $(
                    quote_append!(tokens value.$field_name =);
                    $field_name.append_to(&mut tokens);
                    quote_append!(tokens ;);
                )*
                quote_append!(tokens value);

                proc_macro::TokenTree::Group(proc_macro::Group::new(
                    proc_macro::Delimiter::Brace,
                    tokens,
                ))
            }
        }
    };

    (
        $(#[$enum_attr:meta])*
        $enum_vis:vis enum $enum_name:ident {$(
            $(#[$variant_attr:meta])*
            $variant_name:ident
        ),+ $(,)?}
    ) => {
        $(#[$enum_attr])*
        $enum_vis enum $enum_name {$(
            $(#[$variant_attr])*
            $variant_name
        ),+}

        impl ToTokenStream for $enum_name {
            fn append_to(self, ts: &mut TokenStream) {
                quote_append! { ts
                    ::time::format_description::modifier::$enum_name::
                };
                let name = match self {
                    $(Self::$variant_name => stringify!($variant_name)),+
                };
                ts.extend([TokenTree::Ident(Ident::new(name, Span::mixed_site()))]);
            }
        }
    }
}

to_tokens! {
    pub(crate) struct Day {
        pub(crate) padding: Padding,
    }
}

to_tokens! {
    pub(crate) enum MonthRepr {
        Numerical,
        Long,
        Short,
    }
}

to_tokens! {
    pub(crate) struct Month {
        pub(crate) padding: Padding,
        pub(crate) repr: MonthRepr,
        pub(crate) case_sensitive: bool,
    }
}

to_tokens! {
    pub(crate) struct Ordinal {
        pub(crate) padding: Padding,
    }
}

to_tokens! {
    pub(crate) enum WeekdayRepr {
        Short,
        Long,
        Sunday,
        Monday,
    }
}

to_tokens! {
    pub(crate) struct Weekday {
        pub(crate) repr: WeekdayRepr,
        pub(crate) one_indexed: bool,
        pub(crate) case_sensitive: bool,
    }
}

to_tokens! {
    pub(crate) enum WeekNumberRepr {
        Iso,
        Sunday,
        Monday,
    }
}

to_tokens! {
    pub(crate) struct WeekNumber {
        pub(crate) padding: Padding,
        pub(crate) repr: WeekNumberRepr,
    }
}

to_tokens! {
    pub(crate) enum YearRepr {
        Full,
        LastTwo,
    }
}

to_tokens! {
    pub(crate) struct Year {
        pub(crate) padding: Padding,
        pub(crate) repr: YearRepr,
        pub(crate) iso_week_based: bool,
        pub(crate) sign_is_mandatory: bool,
    }
}

to_tokens! {
    pub(crate) struct Hour {
        pub(crate) padding: Padding,
        pub(crate) is_12_hour_clock: bool,
    }
}

to_tokens! {
    pub(crate) struct Minute {
        pub(crate) padding: Padding,
    }
}

to_tokens! {
    pub(crate) struct Period {
        pub(crate) is_uppercase: bool,
        pub(crate) case_sensitive: bool,
    }
}

to_tokens! {
    pub(crate) struct Second {
        pub(crate) padding: Padding,
    }
}

to_tokens! {
    pub(crate) enum SubsecondDigits {
        One,
        Two,
        Three,
        Four,
        Five,
        Six,
        Seven,
        Eight,
        Nine,
        OneOrMore,
    }
}

to_tokens! {
    pub(crate) struct Subsecond {
        pub(crate) digits: SubsecondDigits,
    }
}

to_tokens! {
    pub(crate) struct OffsetHour {
        pub(crate) sign_is_mandatory: bool,
        pub(crate) padding: Padding,
    }
}

to_tokens! {
    pub(crate) struct OffsetMinute {
        pub(crate) padding: Padding,
    }
}

to_tokens! {
    pub(crate) struct OffsetSecond {
        pub(crate) padding: Padding,
    }
}

to_tokens! {
    pub(crate) enum Padding {
        Space,
        Zero,
        None,
    }
}

pub(crate) struct Ignore {
    pub(crate) count: NonZeroU16,
}

impl ToTokenTree for Ignore {
    fn into_token_tree(self) -> TokenTree {
        quote_group! {{
            ::time::format_description::modifier::Ignore::count(#(self.count))
        }}
    }
}

to_tokens! {
    pub(crate) enum UnixTimestampPrecision {
        Second,
        Millisecond,
        Microsecond,
        Nanosecond,
    }
}

to_tokens! {
    pub(crate) struct UnixTimestamp {
        pub(crate) precision: UnixTimestampPrecision,
        pub(crate) sign_is_mandatory: bool,
    }
}

to_tokens! {
    pub(crate) struct End {}
}
