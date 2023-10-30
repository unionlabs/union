/// A macro for defining #[cfg] if-else statements.
///
/// This is similar to the `if/elif` C preprocessor macro by allowing definition
/// of a cascade of `#[cfg]` cases, emitting the implementation which matches
/// first.
///
/// This allows you to conveniently provide a long list #[cfg]'d blocks of code
/// without having to rewrite each clause multiple times.
macro_rules! cfg_if {
    // match if/else chains with a final `else`
    ($(
        if #[cfg($($meta:meta),*)] { $($it:item)* }
    ) else * else {
        $($it2:item)*
    }) => {
        cfg_if! {
            @__items
            () ;
            $( ( ($($meta),*) ($($it)*) ), )*
            ( () ($($it2)*) ),
        }
    };

    // match if/else chains lacking a final `else`
    (
        if #[cfg($($i_met:meta),*)] { $($i_it:item)* }
        $(
            else if #[cfg($($e_met:meta),*)] { $($e_it:item)* }
        )*
    ) => {
        cfg_if! {
            @__items
            () ;
            ( ($($i_met),*) ($($i_it)*) ),
            $( ( ($($e_met),*) ($($e_it)*) ), )*
            ( () () ),
        }
    };

    // Internal and recursive macro to emit all the items
    //
    // Collects all the negated `cfg`s in a list at the beginning and after the
    // semicolon is all the remaining items
    (@__items ($($not:meta,)*) ; ) => {};
    (@__items ($($not:meta,)*) ; ( ($($m:meta),*) ($($it:item)*) ),
     $($rest:tt)*) => {
        // Emit all items within one block, applying an appropriate #[cfg]. The
        // #[cfg] will require all `$m` matchers specified and must also negate
        // all previous matchers.
        cfg_if! { @__apply cfg(all($($m,)* not(any($($not),*)))), $($it)* }

        // Recurse to emit all other items in `$rest`, and when we do so add all
        // our `$m` matchers to the list of `$not` matchers as future emissions
        // will have to negate everything we just matched as well.
        cfg_if! { @__items ($($not,)* $($m,)*) ; $($rest)* }
    };

    // Internal macro to Apply a cfg attribute to a list of items
    (@__apply $m:meta, $($it:item)*) => {
        $(#[$m] $it)*
    };
}

macro_rules! s {
    ($($(#[$attr:meta])* pub $t:ident $i:ident { $($field:tt)* })*) => ($(
        s!(it: $(#[$attr])* pub $t $i { $($field)* });
    )*);
    (it: $(#[$attr:meta])* pub union $i:ident { $($field:tt)* }) => (
        compile_error!("unions cannot derive extra traits, use s_no_extra_traits instead");
    );
    (it: $(#[$attr:meta])* pub struct $i:ident { $($field:tt)* }) => (
        __item! {
            #[repr(C)]
            #[cfg_attr(feature = "extra_traits", derive(Debug, Eq, Hash, PartialEq))]
            #[allow(deprecated)]
            $(#[$attr])*
            pub struct $i { $($field)* }
        }
        #[allow(deprecated)]
        impl ::Copy for $i {}
        #[allow(deprecated)]
        impl ::Clone for $i {
            fn clone(&self) -> $i { *self }
        }
    );
}

macro_rules! s_no_extra_traits {
    ($($(#[$attr:meta])* pub $t:ident $i:ident { $($field:tt)* })*) => ($(
        s_no_extra_traits!(it: $(#[$attr])* pub $t $i { $($field)* });
    )*);
    (it: $(#[$attr:meta])* pub union $i:ident { $($field:tt)* }) => (
        cfg_if! {
            if #[cfg(libc_union)] {
                __item! {
                    #[repr(C)]
                    $(#[$attr])*
                    pub union $i { $($field)* }
                }

                impl ::Copy for $i {}
                impl ::Clone for $i {
                    fn clone(&self) -> $i { *self }
                }
            }
        }
    );
    (it: $(#[$attr:meta])* pub struct $i:ident { $($field:tt)* }) => (
        __item! {
            #[repr(C)]
            $(#[$attr])*
            pub struct $i { $($field)* }
        }
        #[allow(deprecated)]
        impl ::Copy for $i {}
        #[allow(deprecated)]
        impl ::Clone for $i {
            fn clone(&self) -> $i { *self }
        }
    );
}

macro_rules! e {
    ($($(#[$attr:meta])* pub enum $i:ident { $($field:tt)* })*) => ($(
        __item! {
            #[cfg_attr(feature = "extra_traits", derive(Debug, Eq, Hash, PartialEq))]
            $(#[$attr])*
            pub enum $i { $($field)* }
        }
        impl ::Copy for $i {}
        impl ::Clone for $i {
            fn clone(&self) -> $i { *self }
        }
    )*);
}

macro_rules! s_paren {
    ($($(#[$attr:meta])* pub struct $i:ident ( $($field:tt)* ); )* ) => ($(
        __item! {
            #[cfg_attr(feature = "extra_traits", derive(Debug, Eq, Hash, PartialEq))]
            $(#[$attr])*
            pub struct $i ( $($field)* );
        }
        impl ::Copy for $i {}
        impl ::Clone for $i {
            fn clone(&self) -> $i { *self }
        }
    )*);
}

// This is a pretty horrible hack to allow us to conditionally mark
// some functions as 'const', without requiring users of this macro
// to care about the "const-extern-fn" feature.
//
// When 'const-extern-fn' is enabled, we emit the captured 'const' keyword
// in the expanded function.
//
// When 'const-extern-fn' is disabled, we always emit a plain 'pub unsafe extern fn'.
// Note that the expression matched by the macro is exactly the same - this allows
// users of this macro to work whether or not 'const-extern-fn' is enabled
//
// Unfortunately, we need to duplicate most of this macro between the 'cfg_if' blocks.
// This is because 'const unsafe extern fn' won't even parse on older compilers,
// so we need to avoid emitting it at all of 'const-extern-fn'.
//
// Specifically, moving the 'cfg_if' into the macro body will *not* work.
// Doing so would cause the '#[cfg(feature = "const-extern-fn")]' to be emitted
// into user code. The 'cfg' gate will not stop Rust from trying to parse the
// 'pub const unsafe extern fn', so users would get a compiler error even when
// the 'const-extern-fn' feature is disabled
//
// Note that users of this macro need to place 'const' in a weird position
// (after the closing ')' for the arguments, but before the return type).
// This was the only way I could satisfy the following two requirements:
// 1. Avoid ambiguity errors from 'macro_rules!' (which happen when writing '$foo:ident fn'
// 2. Allow users of this macro to mix 'pub fn foo' and 'pub const fn bar' within the same
// 'f!' block
cfg_if! {
    if #[cfg(libc_const_extern_fn)] {
        macro_rules! f {
            ($($(#[$attr:meta])* pub $({$constness:ident})* fn $i:ident(
                        $($arg:ident: $argty:ty),*
            ) -> $ret:ty {
                $($body:stmt);*
            })*) => ($(
                #[inline]
                $(#[$attr])*
                pub $($constness)* unsafe extern fn $i($($arg: $argty),*
                ) -> $ret {
                    $($body);*
                }
            )*)
        }

        macro_rules! safe_f {
            ($($(#[$attr:meta])* pub $({$constness:ident})* fn $i:ident(
                        $($arg:ident: $argty:ty),*
            ) -> $ret:ty {
                $($body:stmt);*
            })*) => ($(
                #[inline]
                $(#[$attr])*
                pub $($constness)* extern fn $i($($arg: $argty),*
                ) -> $ret {
                    $($body);*
                }
            )*)
        }

        macro_rules! const_fn {
            ($($(#[$attr:meta])* $({$constness:ident})* fn $i:ident(
                        $($arg:ident: $argty:ty),*
            ) -> $ret:ty {
                $($body:stmt);*
            })*) => ($(
                #[inline]
                $(#[$attr])*
                $($constness)* fn $i($($arg: $argty),*
                ) -> $ret {
                    $($body);*
                }
            )*)
        }

    } else {
        macro_rules! f {
            ($($(#[$attr:meta])* pub $({$constness:ident})* fn $i:ident(
                        $($arg:ident: $argty:ty),*
            ) -> $ret:ty {
                $($body:stmt);*
            })*) => ($(
                #[inline]
                $(#[$attr])*
                pub unsafe extern fn $i($($arg: $argty),*
                ) -> $ret {
                    $($body);*
                }
            )*)
        }

        macro_rules! safe_f {
            ($($(#[$attr:meta])* pub $({$constness:ident})* fn $i:ident(
                        $($arg:ident: $argty:ty),*
            ) -> $ret:ty {
                $($body:stmt);*
            })*) => ($(
                #[inline]
                $(#[$attr])*
                pub extern fn $i($($arg: $argty),*
                ) -> $ret {
                    $($body);*
                }
            )*)
        }

        macro_rules! const_fn {
            ($($(#[$attr:meta])* $({$constness:ident})* fn $i:ident(
                        $($arg:ident: $argty:ty),*
            ) -> $ret:ty {
                $($body:stmt);*
            })*) => ($(
                #[inline]
                $(#[$attr])*
                fn $i($($arg: $argty),*
                ) -> $ret {
                    $($body);*
                }
            )*)
        }
    }
}

macro_rules! __item {
    ($i:item) => {
        $i
    };
}

macro_rules! align_const {
    ($($(#[$attr:meta])*
       pub const $name:ident : $t1:ty
       = $t2:ident { $($field:tt)* };)*) => ($(
        #[cfg(libc_align)]
        $(#[$attr])*
        pub const $name : $t1 = $t2 {
            $($field)*
        };
        #[cfg(not(libc_align))]
        $(#[$attr])*
        pub const $name : $t1 = $t2 {
            $($field)*
            __align: [],
        };
    )*)
}

// This macro is used to deprecate items that should be accessed via the mach2 crate
macro_rules! deprecated_mach {
    (pub const $id:ident: $ty:ty = $expr:expr;) => {
        #[deprecated(
            since = "0.2.55",
            note = "Use the `mach2` crate instead",
        )]
        #[allow(deprecated)]
        pub const $id: $ty = $expr;
    };
    ($(pub const $id:ident: $ty:ty = $expr:expr;)*) => {
        $(
            deprecated_mach!(
                pub const $id: $ty = $expr;
            );
        )*
    };
    (pub type $id:ident = $ty:ty;) => {
        #[deprecated(
            since = "0.2.55",
            note = "Use the `mach2` crate instead",
        )]
        #[allow(deprecated)]
        pub type $id = $ty;
    };
    ($(pub type $id:ident = $ty:ty;)*) => {
        $(
            deprecated_mach!(
                pub type $id = $ty;
            );
        )*
    }
}

#[cfg(not(libc_ptr_addr_of))]
macro_rules! ptr_addr_of {
    ($place:expr) => {
        &$place
    };
}

#[cfg(libc_ptr_addr_of)]
macro_rules! ptr_addr_of {
    ($place:expr) => {
        ::core::ptr::addr_of!($place)
    };
}
