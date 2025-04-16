// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's sui subdirectory
//                       The Licensed Work is (c) 2024 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//

// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.

// Notice

// Business Source License 1.1

// Terms

// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.

// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.

// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.

// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.

// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.

// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.

// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).

// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

module ibc::height {
    use ibc::bcs_utils::{Self, BcsBuf};

    public struct Height has drop, copy, store {
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
