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

module zkgm::fungible_asset_metadata {
    use zkgm::zkgm_ethabi;

    public struct FungibleAssetMetadata has copy, drop, store {
        implementation: vector<u8>,
        initializer: vector<u8>,
    }

    public fun new(
        implementation: vector<u8>,
        initializer: vector<u8>,
    ): FungibleAssetMetadata {
        FungibleAssetMetadata {
            implementation,
            initializer
        }
    }

    public fun implementation(metadata: &FungibleAssetMetadata): &vector<u8> {
        &metadata.implementation
    }

    public fun initializer(metadata: &FungibleAssetMetadata): &vector<u8> {
        &metadata.initializer
    }

    public fun encode(metadata: &FungibleAssetMetadata): vector<u8> {
        let mut buf = vector::empty();

        let mut implementation = vector::empty();
        zkgm_ethabi::encode_bytes(&mut implementation, &metadata.implementation);

        let mut initializer = vector::empty();
        zkgm_ethabi::encode_bytes(&mut initializer, &metadata.initializer);

            
        let mut dyn_offset: u64 = 0x20 * 2;

        // `implementation` offset
        zkgm_ethabi::encode_uint(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + implementation.length();

        // `initializer` offset
        zkgm_ethabi::encode_uint(&mut buf, dyn_offset);

        buf.append(implementation);
        buf.append(initializer);

        buf
    }

    public fun decode(buf: &vector<u8>): FungibleAssetMetadata {
        let mut index = 0;
        FungibleAssetMetadata {
            implementation: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            initializer: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
        }
    }
    
    #[test]
    fun test_encode_decode() {
        let encoded = x"000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000021313233656c6e656c6e6176656c6e76626c656e61736c6765316c3265336e6c656e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006761736466736e6564666c6561736e64666c656e6173646c66656e6173656c646e6c6561736e64666c65616e7364666c656e6173646c65666e616c73656e64666c656e61736466656c6e61736c6564666c6561736e64666c656e61736c6465666e6c65616e73646600000000000000000000000000000000000000000000000000";
        let expected_metadata = FungibleAssetMetadata {
            implementation: b"123elnelnavelnvblenaslge1l2e3nlen",
            initializer: b"asdfsnedfleasndflenasdlfenaseldnleasndfleansdflenasdlefnalsendflenasdfelnasledfleasndflenasldefnleansdf",
        };

        let metadata = decode(&encoded);
        assert!(metadata == expected_metadata, 1);
        assert!(encode(&metadata) == encoded, 1);
    }
}
