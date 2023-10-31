//! Converting between Windows GUIDs and UUIDs.
//!
//! Windows GUIDs are specified as using mixed endianness.
//! What you get will depend on the source of the GUID.
//! Functions like `CoCreateGuid` will generate a valid UUID so
//! the fields will be naturally ordered for `Uuid::from_fields`.
//! Other GUIDs might need to be passed to `Uuid::from_fields_le`
//! to have their ordering swapped.

#[test]
#[cfg(windows)]
fn guid_to_uuid() {
    use uuid::Uuid;
    use winapi::shared::guiddef;

    let guid_in = guiddef::GUID {
        Data1: 0x4a35229d,
        Data2: 0x5527,
        Data3: 0x4f30,
        Data4: [0x86, 0x47, 0x9d, 0xc5, 0x4e, 0x1e, 0xe1, 0xe8],
    };

    let uuid = Uuid::from_fields(
        guid_in.Data1,
        guid_in.Data2,
        guid_in.Data3,
        &guid_in.Data4,
    );

    let guid_out = {
        let fields = uuid.as_fields();

        guiddef::GUID {
            Data1: fields.0,
            Data2: fields.1,
            Data3: fields.2,
            Data4: *fields.3,
        }
    };

    assert_eq!(
        (guid_in.Data1, guid_in.Data2, guid_in.Data3, guid_in.Data4),
        (
            guid_out.Data1,
            guid_out.Data2,
            guid_out.Data3,
            guid_out.Data4
        )
    );
}

#[test]
#[cfg(windows)]
fn guid_to_uuid_le_encoded() {
    use uuid::Uuid;
    use winapi::shared::guiddef;

    // A GUID might not be encoded directly as a UUID
    // If its fields are stored in little-endian order they might
    // need to be flipped. Whether or not this is necessary depends
    // on the source of the GUID
    let guid_in = guiddef::GUID {
        Data1: 0x9d22354a,
        Data2: 0x2755,
        Data3: 0x304f,
        Data4: [0x86, 0x47, 0x9d, 0xc5, 0x4e, 0x1e, 0xe1, 0xe8],
    };

    let uuid = Uuid::from_fields_le(
        guid_in.Data1,
        guid_in.Data2,
        guid_in.Data3,
        &guid_in.Data4,
    );

    let guid_out = {
        let fields = uuid.to_fields_le();

        guiddef::GUID {
            Data1: fields.0,
            Data2: fields.1,
            Data3: fields.2,
            Data4: *fields.3,
        }
    };

    assert_eq!(
        (guid_in.Data1, guid_in.Data2, guid_in.Data3, guid_in.Data4),
        (
            guid_out.Data1,
            guid_out.Data2,
            guid_out.Data3,
            guid_out.Data4
        )
    );
}

#[test]
#[cfg(windows)]
fn uuid_from_cocreateguid() {
    use uuid::{Uuid, Variant, Version};
    use winapi::{shared::guiddef, um::combaseapi::CoCreateGuid};

    let mut guid = guiddef::GUID::default();

    unsafe {
        CoCreateGuid(&mut guid as *mut _);
    }

    let uuid =
        Uuid::from_fields(guid.Data1, guid.Data2, guid.Data3, &guid.Data4);

    assert_eq!(Variant::RFC4122, uuid.get_variant());
    assert_eq!(Some(Version::Random), uuid.get_version());
}

fn main() {}
