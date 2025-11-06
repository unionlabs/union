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

module zkgm::solver_metadata {
    use zkgm::zkgm_ethabi;

    public struct SolverMetadata has copy, drop, store {
        solver_address: vector<u8>,
        metadata: vector<u8>,
    }

    public fun new(
        solver_address: vector<u8>,
        metadata: vector<u8>,
    ): SolverMetadata {
        SolverMetadata {
            solver_address,
            metadata
        }
    }

    public fun solver_address(metadata: &SolverMetadata): &vector<u8> {
        &metadata.solver_address
    }

    public fun metadata(metadata: &SolverMetadata): &vector<u8> {
        &metadata.metadata
    }
    
    public fun decode(buf: &vector<u8>): SolverMetadata {
        let mut index = 0;
        SolverMetadata {
            solver_address: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            metadata: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
        }
    }

    public fun encode(meta: &SolverMetadata): vector<u8> {
        let mut buf = vector::empty();

        let mut solver_address = vector::empty();
        zkgm_ethabi::encode_bytes(&mut solver_address, &meta.solver_address);

        let mut metadata = vector::empty();
        zkgm_ethabi::encode_bytes(&mut metadata, &meta.metadata);

        // Each static slot is 32 bytes. There are 2 dynamic fields, so we store offsets.
        let mut dyn_offset = 0x20 * 2;

        // offset for solver_address
        zkgm_ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + vector::length(&solver_address);

        // offset for metadata
        zkgm_ethabi::encode_uint<u64>(&mut buf, dyn_offset);

        vector::append(&mut buf, solver_address);
        vector::append(&mut buf, metadata);

        buf
    }

    /// Encode into an existing buffer using *absolute offsets* from the start of `buf`.
    public fun encode_into(buf: &mut vector<u8>, m: &SolverMetadata) {

        let mut body_solver = vector::empty<u8>();
        zkgm_ethabi::encode_bytes(&mut body_solver, solver_address(m));
        let mut body_meta = vector::empty<u8>();
        zkgm_ethabi::encode_bytes(&mut body_meta, metadata(m));

        let base: u64 = vector::length(buf) as u64;


        let off_solver: u64 = base + 64;

        let off_meta: u64 = off_solver + (vector::length(&body_solver) as u64);

        zkgm_ethabi::encode_uint<u64>(buf, off_solver);
        zkgm_ethabi::encode_uint<u64>(buf, off_meta);

        vector::append(buf, body_solver);
        vector::append(buf, body_meta);
    }

    #[test]
    fun test_solver_metadata_roundtrip() {
        use sui::bcs;

        let m = new(
            bcs::to_bytes(&@0x1111111111111111111111111111111111111111),
            b"hello-world"
        );
        let enc = encode(&m);
        let dec = decode(&enc);

        assert!(*solver_address(&dec)
                == *solver_address(&m), 1);
        assert!(*metadata(&dec)
                == *metadata(&m), 2);
    }

    #[test]
    fun test_solver_metadata_empty_metadata() {
        use sui::bcs;

        let m = new(
            bcs::to_bytes(&@0x0c8C6f58156D10d18193A8fFdD853e1b9F8D8836),
            x""
        );
        let enc = encode(&m);


        std::debug::print(&std::string::utf8(b"encoded solvermetadata::"));
        std::debug::print(&enc);

        let dec = decode(&enc);

        assert!(*solver_address(&dec)
                == *solver_address(&m), 1);
        assert!(*metadata(&dec)
                == *metadata(&m), 2);


        let data_is = x"000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020d7d5df03683eeb343cb259ff2bab8dc590ea8d4c0c0d96bfdcf86846f7dcb2aa";

        let decoded = decode(&data_is);
        
        std::debug::print(&std::string::utf8(b"decoded::"));
        std::debug::print(&decoded);
    }


}
