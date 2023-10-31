mod component;
pub(super) mod modifier;

use proc_macro::{Literal, TokenStream};

pub(crate) use self::component::Component;
use crate::to_tokens::ToTokenStream;

#[allow(variant_size_differences)]
pub(crate) enum OwnedFormatItem {
    Literal(Box<[u8]>),
    Component(Component),
    Compound(Box<[Self]>),
    Optional(Box<Self>),
    First(Box<[Self]>),
}

impl ToTokenStream for OwnedFormatItem {
    fn append_to(self, ts: &mut TokenStream) {
        match self {
            Self::Literal(bytes) => quote_append! { ts
                ::time::format_description::FormatItem::Literal {
                    0: #(Literal::byte_string(bytes.as_ref()))
                }
            },
            Self::Component(component) => quote_append! { ts
                ::time::format_description::FormatItem::Component { 0: #S(component) }
            },
            Self::Compound(items) => {
                let items = items
                    .into_vec()
                    .into_iter()
                    .map(|item| quote! { #S(item), })
                    .collect::<TokenStream>();
                quote_append! { ts
                    ::time::format_description::FormatItem::Compound { 0: &[#S(items)] }
                }
            }
            Self::Optional(item) => quote_append! {ts
                ::time::format_description::FormatItem::Optional { 0: &#S(*item) }
            },
            Self::First(items) => {
                let items = items
                    .into_vec()
                    .into_iter()
                    .map(|item| quote! { #S(item), })
                    .collect::<TokenStream>();
                quote_append! { ts
                    ::time::format_description::FormatItem::First { 0: &[#S(items)] }
                }
            }
        }
    }
}
