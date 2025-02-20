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

module ibc::helpers {
    use ibc::packet::Packet;
    use std::string::{Self, String};
    use std::copyable_any;

    struct RecvPacketParams has copy, drop, store {
        packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>
    }

    struct RecvIntentPacketParams has copy, drop, store {
        packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>
    }

    struct AcknowledgePacketParams has copy, drop, store {
        packet: Packet,
        acknowledgement: vector<u8>,
        relayer: address
    }

    struct TimeoutPacketParams has copy, drop, store {
        packet: Packet,
        relayer: address
    }

    struct ChannelOpenInitParams has copy, drop, store {
        connection_id: u32,
        channel_id: u32,
        version: String
    }

    struct ChannelOpenTryParams has copy, drop, store {
        connection_id: u32,
        channel_id: u32,
        counterparty_channel_id: u32,
        version: String,
        counterparty_version: String
    }

    struct ChannelOpenAckParams has copy, drop, store {
        channel_id: u32,
        counterparty_channel_id: u32,
        counterparty_version: String
    }

    struct ChannelOpenConfirmParams has copy, drop, store {
        channel_id: u32
    }

    struct ChannelCloseInitParams has copy, drop, store {
        channel_id: u32
    }

    struct ChannelCloseConfirmParams has copy, drop, store {
        channel_id: u32
    }

    public inline fun on_packet<W: drop>(
        witness: W,
        on_channel_open_init: |u32, u32, String|,
        on_channel_open_try: |u32, u32, u32, String, String|,
        on_channel_open_ack: |u32, u32, String|,
        on_channel_open_confirm: |u32|,
        on_recv_packet: |Packet, address, vector<u8>|,
        on_recv_intent_packet: |Packet, address, vector<u8>|,
        on_acknowledge_packet: |Packet, vector<u8>, address|,
        on_timeout_packet: |Packet, address|,
        on_channel_close_init: |u32|,
        on_channel_close_confirm: |u32|
    ): u64 {
        let value: copyable_any::Any = ibc::dispatcher::get_data(witness);
        let type_name_output = *copyable_any::type_name(&value);

        if (type_name_output == std::type_info::type_name<RecvPacketParams>()) {
            let (packet, relayer, relayer_msg) =
                on_recv_packet_deconstruct(copyable_any::unpack<RecvPacketParams>(value));
            on_recv_packet(packet, relayer, relayer_msg);
        } else if (type_name_output
            == std::type_info::type_name<RecvIntentPacketParams>()) {
            let (packet, relayer, relayer_msg) =
                on_recv_intent_packet_deconstruct(
                    copyable_any::unpack<RecvIntentPacketParams>(value)
                );
            on_recv_intent_packet(packet, relayer, relayer_msg);
        } else if (type_name_output
            == std::type_info::type_name<AcknowledgePacketParams>()) {
            let (packet, acknowledgement, relayer) =
                on_acknowledge_packet_deconstruct(
                    copyable_any::unpack<AcknowledgePacketParams>(value)
                );
            on_acknowledge_packet(packet, acknowledgement, relayer);
        } else if (type_name_output == std::type_info::type_name<TimeoutPacketParams>()) {
            let (packet, relayer) =
                on_timeout_packet_deconstruct(
                    copyable_any::unpack<TimeoutPacketParams>(value)
                );
            on_timeout_packet(packet, relayer);
        } else if (type_name_output
            == std::type_info::type_name<ChannelOpenInitParams>()) {
            let (connection_id, channel_id, version) =
                on_channel_open_init_deconstruct(
                    copyable_any::unpack<ChannelOpenInitParams>(value)
                );
            on_channel_open_init(connection_id, channel_id, version);
        } else if (type_name_output
            == std::type_info::type_name<ChannelOpenTryParams>()) {
            let (
                connection_id,
                channel_id,
                counterparty_channel_id,
                version,
                counterparty_version
            ) =
                on_channel_open_try_deconstruct(
                    copyable_any::unpack<ChannelOpenTryParams>(value)
                );
            on_channel_open_try(
                connection_id,
                channel_id,
                counterparty_channel_id,
                version,
                counterparty_version
            );
        } else if (type_name_output
            == std::type_info::type_name<ChannelOpenAckParams>()) {
            let (channel_id, counterparty_channel_id, counterparty_version) =
                on_channel_open_ack_deconstruct(
                    copyable_any::unpack<ChannelOpenAckParams>(value)
                );
            on_channel_open_ack(
                channel_id, counterparty_channel_id, counterparty_version
            );
        } else if (type_name_output
            == std::type_info::type_name<ChannelOpenConfirmParams>()) {
            let channel_id =
                on_channel_open_confirm_deconstruct(
                    copyable_any::unpack<ChannelOpenConfirmParams>(value)
                );
            on_channel_open_confirm(channel_id);
        } else if (type_name_output
            == std::type_info::type_name<ChannelCloseInitParams>()) {
            let channel_id =
                on_channel_close_init_deconstruct(
                    copyable_any::unpack<ChannelCloseInitParams>(value)
                );
            on_channel_close_init(channel_id);
        } else if (type_name_output
            == std::type_info::type_name<ChannelCloseConfirmParams>()) {
            let channel_id =
                on_channel_close_confirm_deconstruct(
                    copyable_any::unpack<ChannelCloseConfirmParams>(value)
                );
            on_channel_close_confirm(channel_id);
        } else {
            std::debug::print(
                &string::utf8(b"Invalid function type detected in on_packet function!")
            );
        };

        0
    }

    public fun pack_channel_open_init_params(
        connection_id: u32, channel_id: u32, version: String
    ): copyable_any::Any {
        copyable_any::pack<ChannelOpenInitParams>(
            ChannelOpenInitParams { connection_id, channel_id, version }
        )
    }

    public fun pack_channel_open_try_params(
        connection_id: u32,
        channel_id: u32,
        counterparty_channel_id: u32,
        version: String,
        counterparty_version: String
    ): copyable_any::Any {
        copyable_any::pack<ChannelOpenTryParams>(
            ChannelOpenTryParams {
                connection_id,
                channel_id,
                counterparty_channel_id,
                version,
                counterparty_version
            }
        )
    }

    public fun pack_channel_open_ack_params(
        channel_id: u32, counterparty_channel_id: u32, counterparty_version: String
    ): copyable_any::Any {
        copyable_any::pack<ChannelOpenAckParams>(
            ChannelOpenAckParams {
                channel_id,
                counterparty_channel_id,
                counterparty_version
            }
        )
    }

    public fun pack_channel_open_confirm_params(channel_id: u32): copyable_any::Any {
        copyable_any::pack<ChannelOpenConfirmParams>(
            ChannelOpenConfirmParams { channel_id }
        )
    }

    public fun pack_channel_close_init_params(channel_id: u32): copyable_any::Any {
        copyable_any::pack<ChannelCloseInitParams>(ChannelCloseInitParams { channel_id })
    }

    public fun pack_channel_close_confirm_params(channel_id: u32): copyable_any::Any {
        copyable_any::pack<ChannelCloseConfirmParams>(
            ChannelCloseConfirmParams { channel_id }
        )
    }

    public fun pack_acknowledge_packet_params(
        packet: Packet, acknowledgement: vector<u8>, relayer: address
    ): copyable_any::Any {
        copyable_any::pack<AcknowledgePacketParams>(
            AcknowledgePacketParams { packet, acknowledgement, relayer }
        )
    }

    public fun pack_timeout_packet_params(
        packet: Packet, relayer: address
    ): copyable_any::Any {
        copyable_any::pack<TimeoutPacketParams>(TimeoutPacketParams { packet, relayer })
    }

    public fun pack_recv_packet_params(
        packet: Packet, relayer: address, relayer_msg: vector<u8>
    ): copyable_any::Any {
        copyable_any::pack<RecvPacketParams>(
            RecvPacketParams { packet, relayer, relayer_msg }
        )
    }

    public fun pack_recv_intent_packet_params(
        packet: Packet, relayer: address, relayer_msg: vector<u8>
    ): copyable_any::Any {
        copyable_any::pack<RecvIntentPacketParams>(
            RecvIntentPacketParams { packet, relayer, relayer_msg }
        )
    }

    public fun new_channel_open_confirm_params(channel_id: u32): ChannelOpenConfirmParams {
        ChannelOpenConfirmParams { channel_id }
    }

    // Getter for RecvPacketParams
    public fun get_packet_from_recv_param(param: &RecvPacketParams): &Packet {
        &param.packet
    }

    // Getter for RecvPacketParams
    public fun get_packet_from_recv_intent_param(
        param: &RecvIntentPacketParams
    ): &Packet {
        &param.packet
    }

    // Getters for AcknowledgePacketParams
    public fun get_packet_from_ack_param(param: &AcknowledgePacketParams): &Packet {
        &param.packet
    }

    public fun get_acknowledgement_from_ack_param(
        param: &AcknowledgePacketParams
    ): &vector<u8> {
        &param.acknowledgement
    }

    // Getter for TimeoutPacketParams
    public fun get_packet_from_timeout_param(param: &TimeoutPacketParams): &Packet {
        &param.packet
    }

    // Getters for ChannelOpenInitParams
    public fun get_connection_id_from_channel_open_init_param(
        param: &ChannelOpenInitParams
    ): u32 {
        param.connection_id
    }

    public fun get_channel_id_from_channel_open_init_param(
        param: &ChannelOpenInitParams
    ): u32 {
        param.channel_id
    }

    public fun get_version_from_channel_open_init_param(
        param: &ChannelOpenInitParams
    ): &String {
        &param.version
    }

    // Getters for ChannelOpenTryParams
    public fun get_connection_id_from_channel_open_try_param(
        param: &ChannelOpenTryParams
    ): u32 {
        param.connection_id
    }

    public fun get_channel_id_from_channel_open_try_param(
        param: &ChannelOpenTryParams
    ): u32 {
        param.channel_id
    }

    public fun get_counterparty_channel_id_from_channel_open_try_param(
        param: &ChannelOpenTryParams
    ): u32 {
        param.counterparty_channel_id
    }

    public fun get_version_from_channel_open_try_param(
        param: &ChannelOpenTryParams
    ): &String {
        &param.version
    }

    public fun get_counterparty_version_from_channel_open_try_param(
        param: &ChannelOpenTryParams
    ): &String {
        &param.counterparty_version
    }

    // Getters for ChannelOpenAckParams
    public fun get_channel_id_from_channel_open_ack_param(
        param: &ChannelOpenAckParams
    ): u32 {
        param.channel_id
    }

    public fun get_counterparty_channel_id_from_channel_open_ack_param(
        param: &ChannelOpenAckParams
    ): u32 {
        param.counterparty_channel_id
    }

    public fun get_counterparty_version_from_channel_open_ack_param(
        param: &ChannelOpenAckParams
    ): &String {
        &param.counterparty_version
    }

    // Getter for ChannelOpenConfirmParams
    public fun get_channel_id_from_channel_open_confirm_param(
        param: &ChannelOpenConfirmParams
    ): u32 {
        param.channel_id
    }

    // Getter for ChannelCloseInitParams
    public fun get_channel_id_from_channel_close_init_param(
        param: &ChannelCloseInitParams
    ): u32 {
        param.channel_id
    }

    // Getter for ChannelCloseConfirmParams
    public fun get_channel_id_from_channel_close_confirm_param(
        param: &ChannelCloseConfirmParams
    ): u32 {
        param.channel_id
    }

    public fun on_recv_packet_deconstruct(
        recv_param: RecvPacketParams
    ): (Packet, address, vector<u8>) {
        (recv_param.packet, recv_param.relayer, recv_param.relayer_msg)
    }

    public fun on_recv_intent_packet_deconstruct(
        recv_intent_param: RecvIntentPacketParams
    ): (Packet, address, vector<u8>) {
        (
            recv_intent_param.packet,
            recv_intent_param.relayer,
            recv_intent_param.relayer_msg
        )
    }

    public fun on_acknowledge_packet_deconstruct(
        ack_param: AcknowledgePacketParams
    ): (Packet, vector<u8>, address) {
        (ack_param.packet, ack_param.acknowledgement, ack_param.relayer)
    }

    public fun on_timeout_packet_deconstruct(
        timeout_param: TimeoutPacketParams
    ): (Packet, address) {
        (timeout_param.packet, timeout_param.relayer)
    }

    public fun on_channel_open_init_deconstruct(
        init_param: ChannelOpenInitParams
    ): (u32, u32, String) {
        (init_param.connection_id, init_param.channel_id, init_param.version)
    }

    public fun on_channel_open_try_deconstruct(
        try_param: ChannelOpenTryParams
    ): (u32, u32, u32, String, String) {
        (
            try_param.connection_id,
            try_param.channel_id,
            try_param.counterparty_channel_id,
            try_param.version,
            try_param.counterparty_version
        )
    }

    public fun on_channel_open_ack_deconstruct(
        ack_param: ChannelOpenAckParams
    ): (u32, u32, String) {
        (
            ack_param.channel_id,
            ack_param.counterparty_channel_id,
            ack_param.counterparty_version
        )
    }

    public fun on_channel_open_confirm_deconstruct(
        confirm_param: ChannelOpenConfirmParams
    ): u32 {
        confirm_param.channel_id
    }

    public fun on_channel_close_init_deconstruct(
        close_init_param: ChannelCloseInitParams
    ): u32 {
        close_init_param.channel_id
    }

    public fun on_channel_close_confirm_deconstruct(
        close_confirm_param: ChannelCloseConfirmParams
    ): u32 {
        close_confirm_param.channel_id
    }
}
