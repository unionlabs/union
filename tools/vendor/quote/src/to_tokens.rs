use super::TokenStreamExt;
use core::iter;
use proc_macro2::{Group, Ident, Literal, Punct, Span, TokenStream, TokenTree};
use std::borrow::Cow;
use std::rc::Rc;

/// Types that can be interpolated inside a `quote!` invocation.
///
/// [`quote!`]: macro.quote.html
pub trait ToTokens {
    /// Write `self` to the given `TokenStream`.
    ///
    /// The token append methods provided by the [`TokenStreamExt`] extension
    /// trait may be useful for implementing `ToTokens`.
    ///
    /// [`TokenStreamExt`]: trait.TokenStreamExt.html
    ///
    /// # Example
    ///
    /// Example implementation for a struct representing Rust paths like
    /// `std::cmp::PartialEq`:
    ///
    /// ```
    /// use proc_macro2::{TokenTree, Spacing, Span, Punct, TokenStream};
    /// use quote::{TokenStreamExt, ToTokens};
    ///
    /// pub struct Path {
    ///     pub global: bool,
    ///     pub segments: Vec<PathSegment>,
    /// }
    ///
    /// impl ToTokens for Path {
    ///     fn to_tokens(&self, tokens: &mut TokenStream) {
    ///         for (i, segment) in self.segments.iter().enumerate() {
    ///             if i > 0 || self.global {
    ///                 // Double colon `::`
    ///                 tokens.append(Punct::new(':', Spacing::Joint));
    ///                 tokens.append(Punct::new(':', Spacing::Alone));
    ///             }
    ///             segment.to_tokens(tokens);
    ///         }
    ///     }
    /// }
    /// #
    /// # pub struct PathSegment;
    /// #
    /// # impl ToTokens for PathSegment {
    /// #     fn to_tokens(&self, tokens: &mut TokenStream) {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// ```
    fn to_tokens(&self, tokens: &mut TokenStream);

    /// Convert `self` directly into a `TokenStream` object.
    ///
    /// This method is implicitly implemented using `to_tokens`, and acts as a
    /// convenience method for consumers of the `ToTokens` trait.
    fn to_token_stream(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        self.to_tokens(&mut tokens);
        tokens
    }

    /// Convert `self` directly into a `TokenStream` object.
    ///
    /// This method is implicitly implemented using `to_tokens`, and acts as a
    /// convenience method for consumers of the `ToTokens` trait.
    fn into_token_stream(self) -> TokenStream
    where
        Self: Sized,
    {
        self.to_token_stream()
    }
}

impl<'a, T: ?Sized + ToTokens> ToTokens for &'a T {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        (**self).to_tokens(tokens);
    }
}

impl<'a, T: ?Sized + ToTokens> ToTokens for &'a mut T {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        (**self).to_tokens(tokens);
    }
}

impl<'a, T: ?Sized + ToOwned + ToTokens> ToTokens for Cow<'a, T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        (**self).to_tokens(tokens);
    }
}

impl<T: ?Sized + ToTokens> ToTokens for Box<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        (**self).to_tokens(tokens);
    }
}

impl<T: ?Sized + ToTokens> ToTokens for Rc<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        (**self).to_tokens(tokens);
    }
}

impl<T: ToTokens> ToTokens for Option<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(ref t) = *self {
            t.to_tokens(tokens);
        }
    }
}

impl ToTokens for str {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Literal::string(self));
    }
}

impl ToTokens for String {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_str().to_tokens(tokens);
    }
}

macro_rules! primitive {
    ($($t:ident => $name:ident)*) => {
        $(
            impl ToTokens for $t {
                fn to_tokens(&self, tokens: &mut TokenStream) {
                    tokens.append(Literal::$name(*self));
                }
            }
        )*
    };
}

primitive! {
    i8 => i8_suffixed
    i16 => i16_suffixed
    i32 => i32_suffixed
    i64 => i64_suffixed
    i128 => i128_suffixed
    isize => isize_suffixed

    u8 => u8_suffixed
    u16 => u16_suffixed
    u32 => u32_suffixed
    u64 => u64_suffixed
    u128 => u128_suffixed
    usize => usize_suffixed

    f32 => f32_suffixed
    f64 => f64_suffixed
}

impl ToTokens for char {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Literal::character(*self));
    }
}

impl ToTokens for bool {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let word = if *self { "true" } else { "false" };
        tokens.append(Ident::new(word, Span::call_site()));
    }
}

impl ToTokens for Group {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(self.clone());
    }
}

impl ToTokens for Ident {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(self.clone());
    }
}

impl ToTokens for Punct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(self.clone());
    }
}

impl ToTokens for Literal {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(self.clone());
    }
}

impl ToTokens for TokenTree {
    fn to_tokens(&self, dst: &mut TokenStream) {
        dst.append(self.clone());
    }
}

impl ToTokens for TokenStream {
    fn to_tokens(&self, dst: &mut TokenStream) {
        dst.extend(iter::once(self.clone()));
    }

    fn into_token_stream(self) -> TokenStream {
        self
    }
}
