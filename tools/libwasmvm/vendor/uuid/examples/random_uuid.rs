//! Generating a random UUID.
//!
//! If you enable the `v4` feature you can generate random UUIDs.

#[test]
#[cfg(feature = "v4")]
fn generate_random_uuid() {
    use uuid::Uuid;

    let uuid = Uuid::new_v4();

    assert_eq!(Some(uuid::Version::Random), uuid.get_version());
}

fn main() {}
