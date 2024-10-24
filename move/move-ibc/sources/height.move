module ibc::height {
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

    public fun decode_bcs(buf: &mut BcsBuf): Height {
        // let length = bcs_utils::peel_length_prefix(buf);
        // assert!(length == 2, 1); // TODO: Better error code here
        Height {
            revision_number: bcs_utils::peel_u64(buf),
            revision_height: bcs_utils::peel_u64(buf)
        }
    }
}
