#![allow(unused_macros)]

/// Check that the size and alignment of a type match the `sys` bindings.
macro_rules! check_type {
    ($struct:ident) => {
        assert_eq!(
            (
                core::mem::size_of::<$struct>(),
                core::mem::align_of::<$struct>()
            ),
            (
                core::mem::size_of::<c::$struct>(),
                core::mem::align_of::<c::$struct>()
            )
        );
    };
}

/// The same as `check_type`, but for unions and anonymous structs we've
/// renamed to avoid having types like `bindgen_ty_1` in the API.
macro_rules! check_renamed_type {
    ($to:ident, $from:ident) => {
        assert_eq!(
            (core::mem::size_of::<$to>(), core::mem::align_of::<$to>()),
            (
                core::mem::size_of::<c::$from>(),
                core::mem::align_of::<c::$from>()
            )
        );
    };
}

/// Check that the field of a struct has the same offset as the
/// corresponding field in the `sys` bindings.
macro_rules! check_struct_field {
    ($struct:ident, $field:ident) => {
        assert_eq!(
            (
                memoffset::offset_of!($struct, $field),
                memoffset::span_of!($struct, $field)
            ),
            (
                memoffset::offset_of!(c::$struct, $field),
                memoffset::span_of!(c::$struct, $field)
            )
        );
    };
}

/// The same as `check_struct_field`, but for unions and anonymous structs
/// we've renamed to avoid having types like `bindgen_ty_1` in the API.
macro_rules! check_struct_renamed_field {
    ($struct:ident, $to:ident, $from:ident) => {
        assert_eq!(
            (
                memoffset::offset_of!($struct, $to),
                memoffset::span_of!($struct, $to)
            ),
            (
                memoffset::offset_of!(c::$struct, $from),
                memoffset::span_of!(c::$struct, $from)
            )
        );
    };
}

/// The same as `check_struct_renamed_field`, but for when both the struct
/// and a field are renamed.
macro_rules! check_renamed_struct_renamed_field {
    ($to_struct:ident, $from_struct:ident, $to:ident, $from:ident) => {
        assert_eq!(
            (
                memoffset::offset_of!($to_struct, $to),
                memoffset::span_of!($to_struct, $to)
            ),
            (
                memoffset::offset_of!(c::$from_struct, $from),
                memoffset::span_of!(c::$from_struct, $from)
            )
        );
    };
}

/// For the common case of no renaming, check all fields of a struct.
macro_rules! check_struct {
    ($name:ident, $($field:ident),*) => {
        // Check the size and alignment.
        check_type!($name);

        // Check that we have all the fields.
        if false {
            let _test = $name {
                // SAFETY: This code is guarded by `if false`.
                $($field: unsafe { core::mem::zeroed() }),*
            };
        }

        // Check that the fields have the right sizes and offsets.
        $(check_struct_field!($name, $field));*
    };
}
