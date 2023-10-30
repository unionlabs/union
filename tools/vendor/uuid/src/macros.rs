macro_rules! define_uuid_macro {
    {$(#[$doc:meta])*} => {
        $(#[$doc])*
        #[cfg(feature = "macro-diagnostics")]
        #[macro_export]
        macro_rules! uuid {
            ($uuid:literal) => {{
                $crate::Uuid::from_bytes($crate::uuid_macro_internal::parse_lit!($uuid))
            }};
        }

        $(#[$doc])*
        #[cfg(not(feature = "macro-diagnostics"))]
        #[macro_export]
        macro_rules! uuid {
            ($uuid:literal) => {{
                const OUTPUT: $crate::Uuid = match $crate::Uuid::try_parse($uuid) {
                    Ok(u) => u,
                    Err(_) => {
                        // here triggers const_err
                        // const_panic requires 1.57
                        #[allow(unconditional_panic)]
                        let _ = ["invalid uuid representation"][1];

                        loop {} // -> never type
                    }
                };
                OUTPUT
            }};
        }
    }
}

define_uuid_macro! {
/// Parse [`Uuid`][uuid::Uuid]s from string literals at compile time.
///
/// ## Usage
///
/// This macro transforms the string literal representation of a
/// [`Uuid`][uuid::Uuid] into the bytes representation, raising a compilation
/// error if it cannot properly be parsed.
///
/// ## Examples
///
/// Setting a global constant:
///
/// ```
/// # use uuid::{uuid, Uuid};
/// pub const SCHEMA_ATTR_CLASS: Uuid = uuid!("00000000-0000-0000-0000-ffff00000000");
/// pub const SCHEMA_ATTR_UUID: Uuid = uuid!("00000000-0000-0000-0000-ffff00000001");
/// pub const SCHEMA_ATTR_NAME: Uuid = uuid!("00000000-0000-0000-0000-ffff00000002");
/// ```
///
/// Defining a local variable:
///
/// ```
/// # use uuid::uuid;
/// let uuid = uuid!("urn:uuid:F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4");
/// ```
///
/// ## Compilation Failures
///
/// Invalid UUIDs are rejected:
///
/// ```compile_fail
/// # use uuid::uuid;
/// let uuid = uuid!("F9168C5E-ZEB2-4FAA-B6BF-329BF39FA1E4");
/// ```
///
/// Enable the feature `macro-diagnostics` to see the error messages below.
///
/// Provides the following compilation error:
///
/// ```txt
/// error: invalid character: expected an optional prefix of `urn:uuid:` followed by [0-9a-fA-F-], found Z at 9
///     |
///     |     let id = uuid!("F9168C5E-ZEB2-4FAA-B6BF-329BF39FA1E4");
///     |                              ^
/// ```
///
/// Tokens that aren't string literals are also rejected:
///
/// ```compile_fail
/// # use uuid::uuid;
/// let uuid_str: &str = "550e8400e29b41d4a716446655440000";
/// let uuid = uuid!(uuid_str);
/// ```
///
/// Provides the following compilation error:
///
/// ```txt
/// error: expected string literal
///   |
///   |     let uuid = uuid!(uuid_str);
///   |                      ^^^^^^^^
/// ```
///
/// [uuid::Uuid]: https://docs.rs/uuid/*/uuid/struct.Uuid.html
}
