module ibc::height {
    use std::vector;

    use ibc::proto_utils;
    use ibc::bcs_utils::{Self, BcsBuf};

    struct Height has key, drop, copy, store {
        revision_number: u64,
        revision_height: u64
    }

    public fun new(revision_number: u64, revision_height: u64): Height {
        Height {
            revision_number: revision_number,
            revision_height: revision_height
        }
    }

    public fun default(): Height {
        new(0, 0)
    }

    public fun get_revision_number(height: &Height): u64 {
        height.revision_number
    }

    public fun get_revision_height(height: &Height): u64 {
        height.revision_height
    }

    public fun is_zero(height: &Height): bool {
        height.revision_number == 0 && height.revision_height == 0
    }

    public fun gte(height1: &Height, height2: &Height): bool {
        height1.revision_number > height2.revision_number
            || (
                height1.revision_number == height2.revision_number
                    && height1.revision_height >= height2.revision_height
            )
    }

    public fun lt(height1: &Height, height2: &Height): bool {
        height1.revision_number < height2.revision_number
            || (
                height1.revision_number == height2.revision_number
                    && height1.revision_height < height2.revision_height
            )
    }

    public fun set_revision_height(
        height: &mut Height, revision_height: u64
    ) {
        height.revision_height = revision_height;
    }

    public fun encode_proto(height: Height): vector<u8> {
        let buf = vector::empty();

        if (height.revision_number != 0) {
            vector::append(&mut buf, proto_utils::encode_u64(1, height.revision_number));
        };

        if (height.revision_height != 0) {
            vector::append(&mut buf, proto_utils::encode_u64(2, height.revision_height));
        };

        buf
    }

    public fun decode_proto(
        buf: &vector<u8>,
        cursor: u64,
        len: u64,
        height: &mut Height
    ): (u64, u64) {
        let first_pos = cursor;
        while (cursor - first_pos < len) {
            let (tag, wire_type, advance, err) = proto_utils::decode_prefix(buf, cursor);
            if (err != 0) {
                return (0, err)
            };
            cursor = cursor + advance;
            let advance =
                if (tag == 1) {
                    let (num, advance, err) =
                        proto_utils::decode_varint(wire_type, buf, cursor);
                    if (err != 0) {
                        return (0, err)
                    };
                    height.revision_number = num;
                    advance
                } else if (tag == 2) {
                    let (num, advance, err) =
                        proto_utils::decode_varint(wire_type, buf, cursor);
                    if (err != 0) {
                        return (0, err)
                    };
                    height.revision_height = num;
                    advance
                } else {
                    return (0, 1)
                };
            cursor = cursor + advance;
        };

        (cursor - first_pos, 0)
    }

    public fun decode_bcs(buf: &mut BcsBuf): Height {
        // let length = bcs_utils::peel_length_prefix(buf);
        // assert!(length == 2, 1); // TODO: Better error code here
        Height {
            revision_number: bcs_utils::peel_u64(buf),
            revision_height: bcs_utils::peel_u64(buf)
        }
    }
}
