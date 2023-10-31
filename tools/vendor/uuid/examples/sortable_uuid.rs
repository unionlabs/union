//! Generating a sortable UUID.
//!
//! If you enable the `v7` feature you can generate sortable UUIDs.

#[test]
#[cfg(feature = "v7")]
fn generate_sortable_uuid() {
    use uuid::Uuid;

    let uuid = Uuid::now_v7();

    assert_eq!(Some(uuid::Version::SortRand), uuid.get_version());
}

fn main() {}
