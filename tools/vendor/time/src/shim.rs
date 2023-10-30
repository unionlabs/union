//! Macro for simulating let-else on older compilers.
//!
//! This module and its macros will be removed once the MSRV is 1.65 (NET 2023-05-03).

#![allow(unused_macros)]
#![allow(clippy::missing_docs_in_private_items)]

// The following code is copyright 2016 Alex Burka. Available under the MIT OR Apache-2.0 license.
// Some adaptations have been made to the original code.

pub(crate) enum LetElseBodyMustDiverge {}

#[allow(clippy::missing_docs_in_private_items)]
macro_rules! __guard_output {
    ((($($imms:ident)*) ($($muts:ident)*)),
    [($($pattern:tt)*) ($rhs:expr) ($diverge:expr)]) => {
        __guard_impl!(@as_stmt
            let ($($imms,)* $(mut $muts,)*) = {
                #[allow(unused_mut)]
                match $rhs {
                    $($pattern)* => {
                        ($($imms,)* $($muts,)*)
                    },
                    _ => {
                        let _: $crate::shim::LetElseBodyMustDiverge = $diverge;
                    },
                }
            }
        )
    };
}

macro_rules! __guard_impl {
    // 0. cast a series of token trees to a statement
    (@as_stmt $s:stmt) => { $s };

    // 1. output stage
    (@collect () -> $($rest:tt)*) => {
        __guard_output!($($rest)*)
    };


    // 2. identifier collection stage
    //      The pattern is scanned destructively. Anything that looks like a capture (including
    //      false positives, like un-namespaced/empty structs or enum variants) is copied into the
    //      appropriate identifier list. Irrelevant symbols are discarded. The scanning descends
    //      recursively into bracketed structures.

    // unwrap brackets and prepend their contents to the pattern remainder, in case there are captures inside
    (@collect (($($inside:tt)*) $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($inside)* $($tail)*) -> $idents, $thru)
    };
    (@collect ({$($inside:tt)*} $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($inside)* $($tail)*) -> $idents, $thru)
    };
    (@collect ([$($inside:tt)*] $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($inside)* $($tail)*) -> $idents, $thru)
    };

    // discard irrelevant symbols
    (@collect (, $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> $idents, $thru)
    };
    (@collect (.. $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> $idents, $thru)
    };
    (@collect (@ $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> $idents, $thru)
    };
    (@collect (_ $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> $idents, $thru)
    };
    (@collect (& $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> $idents, $thru)
    };

    // ignore generic parameters
    (@collect (:: <$($generic:tt),*> $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> $idents, $thru)
    };
    // a path can't be a capture, and a path can't end with ::, so the ident after :: is never a capture
    (@collect (:: $pathend:ident $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> $idents, $thru)
    };

    // alternative patterns may be given with | as long as the same captures (including type) appear on each side
    // due to this property, if we see a | we've already parsed all the captures and can simply stop
    (@collect (| $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect () -> $idents, $thru) // discard the rest of the pattern, proceed to output stage
    };

    // throw away some identifiers that do not represent captures

    // an ident followed by a colon is the name of a structure member
    (@collect ($id:ident: $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> $idents, $thru)
    };
    // paths do not represent captures
    (@collect ($pathcomp:ident :: $pathend:ident $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> $idents, $thru)
    };
    // an ident followed by parentheses is the name of a tuple-like struct or enum variant
    // (unwrap the parens to parse the contents)
    (@collect ($id:ident ($($inside:tt)*) $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($inside)* $($tail)*) -> $idents, $thru)
    };
    // an ident followed by curly braces is the name of a struct or struct-like enum variant
    // (unwrap the braces to parse the contents)
    (@collect ($id:ident {$($inside:tt)*} $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($inside)* $($tail)*) -> $idents, $thru)
    };

    // actually identifier collection happens here!

    // capture by mutable reference!
    (@collect (ref mut $id:ident $($tail:tt)*) -> (($($imms:ident)*) $muts:tt), $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> (($($imms)* $id) $muts), $thru)
    };
    // capture by immutable reference!
    (@collect (ref $id:ident $($tail:tt)*) -> (($($imms:ident)*) $muts:tt), $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> (($($imms)* $id) $muts), $thru)
    };
    // capture by move into mutable binding!
    (@collect (mut $id:ident $($tail:tt)*) -> ($imms:tt ($($muts:ident)*)), $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> ($imms ($($muts)* $id)), $thru)
    };
    // capture by move into an immutable binding!
    (@collect ($id:ident $($tail:tt)*) -> (($($imms:ident)*) $muts:tt), $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> (($($imms)* $id) $muts), $thru)
    };

    // 3. splitting (for new syntax)

    // done with pattern (and it's LPED=X)
    (@split (else { $($diverge:tt)* } = $($tail:tt)*) -> ($pat:tt)) => {
        __guard_impl!(@collect $pat -> (() ()), [$pat ($($tail)*) ({ $($diverge)* })])
    };

    // done with pattern (and it's LP=XED)
    (@split (= $($tail:tt)*) -> ($pat:tt)) => {
        __guard_impl!(@split expr ($($tail)*) -> ($pat ()))
    };

    // found a token in the pattern
    (@split ($head:tt $($tail:tt)*) -> (($($pat:tt)*))) => {
        __guard_impl!(@split ($($tail)*) -> (($($pat)* $head)))
    };

    // found an "else DIVERGE" in the expr
    (@split expr (else { $($tail:tt)* }) -> ($pat:tt $expr:tt)) => {
        __guard_impl!(@collect $pat -> (() ()), [$pat $expr ({ $($tail)* })])
    };

    // found an else in the expr with more stuff after it
    (@split expr (else { $($body:tt)* } $($tail:tt)*) -> ($pat:tt ($($expr:tt)*))) => {
        __guard_impl!(@split expr ($($tail)*) -> ($pat ($($expr)* else { $($body)* })))
    };

    // found another token in the expr
    (@split expr ($head:tt $($tail:tt)*) -> ($pat:tt ($($expr:tt)*))) => {
        __guard_impl!(@split expr ($($tail)*) -> ($pat ($($expr)* $head)))
    };

    // 4. entry points

    // new syntax
    (let $($tail:tt)*) => {
        __guard_impl!(@split ($($tail)*) -> (()))
        //            |      |               |
        //            |      |               ^ pattern
        //            |      ^ tail to be split into "PAT = EXPR else DIVERGE"
        //            ^ first pass will do the parsing
    };
}

macro_rules! guard {
    ($($input:tt)*) => {
        __guard_impl!($($input)*)
    };
}
