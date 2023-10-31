use std::iter::Peekable;

use proc_macro::{token_stream, Ident, Span, TokenTree};

use crate::date::Date;
use crate::error::Error;
use crate::offset::Offset;
use crate::time::Time;
use crate::to_tokens::ToTokenTree;
use crate::{date, offset, time};

pub(crate) struct DateTime {
    date: Date,
    time: Time,
    offset: Option<Offset>,
}

pub(crate) fn parse(chars: &mut Peekable<token_stream::IntoIter>) -> Result<DateTime, Error> {
    let date = date::parse(chars)?;
    let time = time::parse(chars)?;
    let offset = match offset::parse(chars) {
        Ok(offset) => Some(offset),
        Err(Error::UnexpectedEndOfInput | Error::MissingComponent { name: "sign", .. }) => None,
        Err(err) => return Err(err),
    };

    if let Some(token) = chars.peek() {
        return Err(Error::UnexpectedToken {
            tree: token.clone(),
        });
    }

    Ok(DateTime { date, time, offset })
}

impl ToTokenTree for DateTime {
    fn into_token_tree(self) -> TokenTree {
        let (type_name, maybe_offset) = match self.offset {
            Some(offset) => (
                Ident::new("OffsetDateTime", Span::mixed_site()),
                quote!(.assume_offset(#(offset))),
            ),
            None => (
                Ident::new("PrimitiveDateTime", Span::mixed_site()),
                quote!(),
            ),
        };

        quote_group! {{
            const DATE_TIME: ::time::#(type_name) = ::time::PrimitiveDateTime::new(
                #(self.date),
                #(self.time),
            ) #S(maybe_offset);
            DATE_TIME
        }}
    }
}
