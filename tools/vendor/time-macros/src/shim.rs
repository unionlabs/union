#![allow(unused_macros)]

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
    (@as_stmt $s:stmt) => { $s };
    (@collect () -> $($rest:tt)*) => {
        __guard_output!($($rest)*)
    };
    (@collect (($($inside:tt)*) $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($inside)* $($tail)*) -> $idents, $thru)
    };
    (@collect ({$($inside:tt)*} $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($inside)* $($tail)*) -> $idents, $thru)
    };
    (@collect ([$($inside:tt)*] $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($inside)* $($tail)*) -> $idents, $thru)
    };
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
    (@collect (:: <$($generic:tt),*> $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> $idents, $thru)
    };
    (@collect (:: $pathend:ident $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> $idents, $thru)
    };
    (@collect (| $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect () -> $idents, $thru)
    };
    (@collect ($id:ident: $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> $idents, $thru)
    };
    (@collect ($pathcomp:ident :: $pathend:ident $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> $idents, $thru)
    };
    (@collect ($id:ident ($($inside:tt)*) $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($inside)* $($tail)*) -> $idents, $thru)
    };
    (@collect ($id:ident {$($inside:tt)*} $($tail:tt)*) -> $idents:tt, $thru:tt) => {
        __guard_impl!(@collect ($($inside)* $($tail)*) -> $idents, $thru)
    };
    (@collect (ref mut $id:ident $($tail:tt)*) -> (($($imms:ident)*) $muts:tt), $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> (($($imms)* $id) $muts), $thru)
    };
    (@collect (ref $id:ident $($tail:tt)*) -> (($($imms:ident)*) $muts:tt), $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> (($($imms)* $id) $muts), $thru)
    };
    (@collect (mut $id:ident $($tail:tt)*) -> ($imms:tt ($($muts:ident)*)), $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> ($imms ($($muts)* $id)), $thru)
    };
    (@collect ($id:ident $($tail:tt)*) -> (($($imms:ident)*) $muts:tt), $thru:tt) => {
        __guard_impl!(@collect ($($tail)*) -> (($($imms)* $id) $muts), $thru)
    };
    (@split (else { $($diverge:tt)* } = $($tail:tt)*) -> ($pat:tt)) => {
        __guard_impl!(@collect $pat -> (() ()), [$pat ($($tail)*) ({ $($diverge)* })])
    };
    (@split (= $($tail:tt)*) -> ($pat:tt)) => {
        __guard_impl!(@split expr ($($tail)*) -> ($pat ()))
    };
    (@split ($head:tt $($tail:tt)*) -> (($($pat:tt)*))) => {
        __guard_impl!(@split ($($tail)*) -> (($($pat)* $head)))
    };
    (@split expr (else { $($tail:tt)* }) -> ($pat:tt $expr:tt)) => {
        __guard_impl!(@collect $pat -> (() ()), [$pat $expr ({ $($tail)* })])
    };
    (@split expr (else { $($body:tt)* } $($tail:tt)*) -> ($pat:tt ($($expr:tt)*))) => {
        __guard_impl!(@split expr ($($tail)*) -> ($pat ($($expr)* else { $($body)* })))
    };
    (@split expr ($head:tt $($tail:tt)*) -> ($pat:tt ($($expr:tt)*))) => {
        __guard_impl!(@split expr ($($tail)*) -> ($pat ($($expr)* $head)))
    };
    (let $($tail:tt)*) => {
        __guard_impl!(@split ($($tail)*) -> (()))
    };
}

macro_rules! guard {
    ($($input:tt)*) => {
        __guard_impl!($($input)*)
    };
}
