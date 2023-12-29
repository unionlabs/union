use std::iter::Peekable;

use proc_macro::{token_stream, Span, TokenTree};

use crate::helpers::{consume_any_ident, consume_number, consume_punct};
use crate::to_tokens::ToTokenTree;
use crate::Error;

pub(crate) struct Offset {
    pub(crate) hours: i8,
    pub(crate) minutes: i8,
    pub(crate) seconds: i8,
}

pub(crate) fn parse(chars: &mut Peekable<token_stream::IntoIter>) -> Result<Offset, Error> {
    if consume_any_ident(&["utc", "UTC"], chars).is_ok() {
        return Ok(Offset {
            hours: 0,
            minutes: 0,
            seconds: 0,
        });
    }

    let sign = if consume_punct('+', chars).is_ok() {
        1
    } else if consume_punct('-', chars).is_ok() {
        -1
    } else if let Some(tree) = chars.next() {
        return Err(Error::UnexpectedToken { tree });
    } else {
        return Err(Error::MissingComponent {
            name: "sign",
            span_start: None,
            span_end: None,
        });
    };

    let (hours_span, hours) = consume_number::<i8>("hour", chars)?;
    let (mut minutes_span, mut minutes) = (Span::mixed_site(), 0);
    let (mut seconds_span, mut seconds) = (Span::mixed_site(), 0);

    if consume_punct(':', chars).is_ok() {
        let min = consume_number::<i8>("minute", chars)?;
        minutes_span = min.0;
        minutes = min.1;

        if consume_punct(':', chars).is_ok() {
            let sec = consume_number::<i8>("second", chars)?;
            seconds_span = sec.0;
            seconds = sec.1;
        }
    }

    if hours >= 24 {
        Err(Error::InvalidComponent {
            name: "hour",
            value: hours.to_string(),
            span_start: Some(hours_span),
            span_end: Some(hours_span),
        })
    } else if minutes >= 60 {
        Err(Error::InvalidComponent {
            name: "minute",
            value: minutes.to_string(),
            span_start: Some(minutes_span),
            span_end: Some(minutes_span),
        })
    } else if seconds >= 60 {
        Err(Error::InvalidComponent {
            name: "second",
            value: seconds.to_string(),
            span_start: Some(seconds_span),
            span_end: Some(seconds_span),
        })
    } else {
        Ok(Offset {
            hours: sign * hours,
            minutes: sign * minutes,
            seconds: sign * seconds,
        })
    }
}

impl ToTokenTree for Offset {
    fn into_token_tree(self) -> TokenTree {
        quote_group! {{
            const OFFSET: ::time::UtcOffset = ::time::UtcOffset::__from_hms_unchecked(
                #(self.hours),
                #(self.minutes),
                #(self.seconds),
            );
            OFFSET
        }}
    }
}
