module IBC::height {
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

    public fun get_revision_number(height: &Height): u64 {
        height.revision_number
    }

    public fun get_revision_height(height: &Height): u64 {
        height.revision_height
    }

}