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
    use windows_sys::core;

    let guid_in = core::GUID {
        data1: 0x4a35229d,
        data2: 0x5527,
        data3: 0x4f30,
        data4: [0x86, 0x47, 0x9d, 0xc5, 0x4e, 0x1e, 0xe1, 0xe8],
    };

    let uuid = Uuid::from_fields(guid_in.data1, guid_in.data2, guid_in.data3, &guid_in.data4);

    let guid_out = {
        let fields = uuid.as_fields();

        core::GUID {
            data1: fields.0,
            data2: fields.1,
            data3: fields.2,
            data4: *fields.3,
        }
    };

    assert_eq!(
        (guid_in.data1, guid_in.data2, guid_in.data3, guid_in.data4),
        (
            guid_out.data1,
            guid_out.data2,
            guid_out.data3,
            guid_out.data4
        )
    );
}

#[test]
#[cfg(windows)]
fn guid_to_uuid_le_encoded() {
    use uuid::Uuid;
    use windows_sys::core;

    // A GUID might not be encoded directly as a UUID
    // If its fields are stored in little-endian order they might
    // need to be flipped. Whether or not this is necessary depends
    // on the source of the GUID
    let guid_in = core::GUID {
        data1: 0x9d22354a,
        data2: 0x2755,
        data3: 0x304f,
        data4: [0x86, 0x47, 0x9d, 0xc5, 0x4e, 0x1e, 0xe1, 0xe8],
    };

    let uuid = Uuid::from_fields_le(guid_in.data1, guid_in.data2, guid_in.data3, &guid_in.data4);

    let guid_out = {
        let fields = uuid.to_fields_le();

        core::GUID {
            data1: fields.0,
            data2: fields.1,
            data3: fields.2,
            data4: *fields.3,
        }
    };

    assert_eq!(
        (guid_in.data1, guid_in.data2, guid_in.data3, guid_in.data4),
        (
            guid_out.data1,
            guid_out.data2,
            guid_out.data3,
            guid_out.data4
        )
    );
}

#[test]
#[cfg(windows)]
fn uuid_from_cocreateguid() {
    use uuid::{Uuid, Variant, Version};
    use windows_sys::core;
    use windows_sys::Win32::System::Com::CoCreateGuid;

    let mut guid = core::GUID {
        data1: 0,
        data2: 0,
        data3: 0,
        data4: [0u8; 8],
    };

    unsafe {
        CoCreateGuid(&mut guid);
    }

    let uuid = Uuid::from_fields(guid.data1, guid.data2, guid.data3, &guid.data4);

    assert_eq!(Variant::RFC4122, uuid.get_variant());
    assert_eq!(Some(Version::Random), uuid.get_version());
}

fn main() {}
