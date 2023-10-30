//! Using the `uuid!` macro.
//!
//! `uuid!` will parse encoded UUIDs at compile time instead of at runtime.
//! If you've got a fixed UUID string handy then consider using `uuid!` instead
//! of `Uuid::parse_str` or `str::parse`.
//!
//! If you enable the `macro-diagnostics` feature, you can see much better
//! error messages.

#[test]
fn parse_uuid_at_compile_time() {
    use uuid::uuid;

    let uuid = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");

    assert_eq!(Some(uuid::Version::Random), uuid.get_version());
}

fn main() {}
