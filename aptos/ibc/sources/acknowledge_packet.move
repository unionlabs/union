// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's aptos subdirectory
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

module ibc::acknowledge_packet {

    use ibc::packet::{Self, Packet};
    use ibc::channel;
    use ibc::engine;
    use ibc::commitment;
    use ibc::helpers;
    use ibc::ibc;
    use ibc::dispatcher;

    use std::vector;

    const E_NOT_ENOUGH_PACKETS: u64 = 34001;

    public entry fun acknowledge_packet<T: key + store + drop>(
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_data: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        acknowledgements: vector<vector<u8>>,
        proof: vector<u8>,
        proof_height: u64,
        relayer: address
    ) {
        let packets: vector<Packet> = vector::empty();
        let i = 0;
        while (i < vector::length(&packet_source_channels)) {
            vector::push_back(
                &mut packets,
                packet::new(
                    *vector::borrow(&packet_source_channels, i),
                    *vector::borrow(&packet_destination_channels, i),
                    *vector::borrow(&packet_data, i),
                    *vector::borrow(&packet_timeout_heights, i),
                    *vector::borrow(&packet_timeout_timestamps, i)
                )
            );
            i = i + 1;
        };
        let l = vector::length(&packets);
        assert!(l > 0, E_NOT_ENOUGH_PACKETS);

        let source_channel_id = packet::source_channel_id(vector::borrow(&packets, 0));

        let channel = ibc::ensure_channel_state(source_channel_id);
        let client_id = ibc::ensure_connection_state(channel::connection_id(&channel));

        let commitment_key =
            commitment::batch_receipts_commitment_key(
                commitment::commit_packets(&packets)
            );
        let commitment_value = commitment::commit_acks(acknowledgements);

        let err =
            ibc::verify_commitment(
                client_id,
                proof_height,
                proof,
                commitment_key,
                commitment_value
            );

        if (err != 0) {
            abort err
        };

        let i = 0;
        while (i < l) {
            let packet = *vector::borrow(&packets, i);
            ibc::mark_packet_as_acknowledged(&packet);
            let acknowledgement = *vector::borrow(&acknowledgements, i);

            let param =
                helpers::pack_acknowledge_packet_params(packet, acknowledgement, relayer);
            engine::dispatch<T>(param);

            dispatcher::delete_storage<T>();

            ibc::emit_acknowledge_packet(
                packet,
                commitment::commit_packet(&packet),
                acknowledgement,
                relayer
            );

            i = i + 1;
        }
    }
}
