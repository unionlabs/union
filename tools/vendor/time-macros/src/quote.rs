macro_rules! quote {
    () => (::proc_macro::TokenStream::new());
    ($($x:tt)*) => {{
        let mut ts = ::proc_macro::TokenStream::new();
        let ts_mut = &mut ts;
        quote_inner!(ts_mut $($x)*);
        ts
    }};
}

#[cfg(any(feature = "formatting", feature = "parsing"))]
macro_rules! quote_append {
    ($ts:ident $($x:tt)*) => {{
        quote_inner!($ts $($x)*);
    }};
}

macro_rules! quote_group {
    ({ $($x:tt)* }) => {
        ::proc_macro::TokenTree::Group(::proc_macro::Group::new(
            ::proc_macro::Delimiter::Brace,
            quote!($($x)*)
        ))
    };
}

macro_rules! sym {
    ($ts:ident $x:tt $y:tt) => {
        $ts.extend([
            ::proc_macro::TokenTree::from(::proc_macro::Punct::new(
                $x,
                ::proc_macro::Spacing::Joint,
            )),
            ::proc_macro::TokenTree::from(::proc_macro::Punct::new(
                $y,
                ::proc_macro::Spacing::Alone,
            )),
        ]);
    };
    ($ts:ident $x:tt) => {
        $ts.extend([::proc_macro::TokenTree::from(::proc_macro::Punct::new(
            $x,
            ::proc_macro::Spacing::Alone,
        ))]);
    };
}

macro_rules! quote_inner {
    // Base case
    ($ts:ident) => {};

    // Single or double symbols
    ($ts:ident :: $($tail:tt)*) => { sym!($ts ':' ':'); quote_inner!($ts $($tail)*); };
    ($ts:ident .. $($tail:tt)*) => { sym!($ts '.' '.'); quote_inner!($ts $($tail)*); };
    ($ts:ident : $($tail:tt)*) => { sym!($ts ':'); quote_inner!($ts $($tail)*); };
    ($ts:ident = $($tail:tt)*) => { sym!($ts '='); quote_inner!($ts $($tail)*); };
    ($ts:ident ; $($tail:tt)*) => { sym!($ts ';'); quote_inner!($ts $($tail)*); };
    ($ts:ident , $($tail:tt)*) => { sym!($ts ','); quote_inner!($ts $($tail)*); };
    ($ts:ident . $($tail:tt)*) => { sym!($ts '.'); quote_inner!($ts $($tail)*); };
    ($ts:ident & $($tail:tt)*) => { sym!($ts '&'); quote_inner!($ts $($tail)*); };
    ($ts:ident << $($tail:tt)*) => { sym!($ts '<' '<'); quote_inner!($ts $($tail)*); };
    ($ts:ident < $($tail:tt)*) => { sym!($ts '<'); quote_inner!($ts $($tail)*); };
    ($ts:ident >> $($tail:tt)*) => { sym!($ts '>' '>'); quote_inner!($ts $($tail)*); };
    ($ts:ident > $($tail:tt)*) => { sym!($ts '>'); quote_inner!($ts $($tail)*); };
    ($ts:ident -> $($tail:tt)*) => { sym!($ts '-' '>'); quote_inner!($ts $($tail)*); };
    ($ts:ident ? $($tail:tt)*) => { sym!($ts '?'); quote_inner!($ts $($tail)*); };
    ($ts:ident ! $($tail:tt)*) => { sym!($ts '!'); quote_inner!($ts $($tail)*); };
    ($ts:ident | $($tail:tt)*) => { sym!($ts '|'); quote_inner!($ts $($tail)*); };
    ($ts:ident * $($tail:tt)*) => { sym!($ts '*'); quote_inner!($ts $($tail)*); };
    ($ts:ident + $($tail:tt)*) => { sym!($ts '+'); quote_inner!($ts $($tail)*); };

    // Identifier
    ($ts:ident $i:ident $($tail:tt)*) => {
        $ts.extend([::proc_macro::TokenTree::from(::proc_macro::Ident::new(
            &stringify!($i),
            ::proc_macro::Span::mixed_site(),
        ))]);
        quote_inner!($ts $($tail)*);
    };

    // Literal
    ($ts:ident 0 $($tail:tt)*) => {
        $ts.extend([::proc_macro::TokenTree::from(::proc_macro::Literal::usize_unsuffixed(0))]);
        quote_inner!($ts $($tail)*);
    };
    ($ts:ident $l:literal $($tail:tt)*) => {
        $ts.extend([::proc_macro::TokenTree::from(::proc_macro::Literal::string(&$l))]);
        quote_inner!($ts $($tail)*);
    };

    // Lifetime
    ($ts:ident $l:lifetime $($tail:tt)*) => {
        $ts.extend([
            ::proc_macro::TokenTree::from(
                ::proc_macro::Punct::new('\'', ::proc_macro::Spacing::Joint)
            ),
            ::proc_macro::TokenTree::from(::proc_macro::Ident::new(
                stringify!($l).trim_start_matches(|c| c == '\''),
                ::proc_macro::Span::mixed_site(),
            )),
        ]);
        quote_inner!($ts $($tail)*);
    };

    // Groups
    ($ts:ident ($($inner:tt)*) $($tail:tt)*) => {
        $ts.extend([::proc_macro::TokenTree::Group(::proc_macro::Group::new(
            ::proc_macro::Delimiter::Parenthesis,
            quote!($($inner)*)
        ))]);
        quote_inner!($ts $($tail)*);
    };
    ($ts:ident [$($inner:tt)*] $($tail:tt)*) => {
        $ts.extend([::proc_macro::TokenTree::Group(::proc_macro::Group::new(
            ::proc_macro::Delimiter::Bracket,
            quote!($($inner)*)
        ))]);
        quote_inner!($ts $($tail)*);
    };
    ($ts:ident {$($inner:tt)*} $($tail:tt)*) => {
        $ts.extend([::proc_macro::TokenTree::Group(::proc_macro::Group::new(
            ::proc_macro::Delimiter::Brace,
            quote!($($inner)*)
        ))]);
        quote_inner!($ts $($tail)*);
    };

    // Interpolated values
    // TokenTree by default
    ($ts:ident #($e:expr) $($tail:tt)*) => {
        $ts.extend([$crate::to_tokens::ToTokenTree::into_token_tree($e)]);
        quote_inner!($ts $($tail)*);
    };
    // Allow a TokenStream by request. It's more expensive, so avoid if possible.
    ($ts:ident #S($e:expr) $($tail:tt)*) => {
        $crate::to_tokens::ToTokenStream::append_to($e, $ts);
        quote_inner!($ts $($tail)*);
    };
}
