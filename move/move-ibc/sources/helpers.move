module ibc::helpers {
    use ibc::packet::Packet;
    use std::string::{Self, String};
    use std::copyable_any;

    struct RecvPacketParams has copy, drop, store {
        packet: Packet
    }

    struct RecvIntentPacketParams has copy, drop, store {
        packet: Packet
    }

    struct AcknowledgePacketParams has copy, drop, store {
        packet: Packet,
        acknowledgement: vector<u8>
    }

    struct TimeoutPacketParams has copy, drop, store {
        packet: Packet
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
        on_recv_packet: |Packet|,
        on_recv_intent_packet: |Packet|,
        on_acknowledge_packet: |Packet, vector<u8>|,
        on_timeout_packet: |Packet|,
        on_channel_close_init: |u32|,
        on_channel_close_confirm: |u32|
    ): u64 {
        let value: copyable_any::Any = ibc::dispatcher::get_data(witness);
        let type_name_output = *copyable_any::type_name(&value);

        if (type_name_output == std::type_info::type_name<RecvPacketParams>()) {
            let (pack) =
                on_recv_packet_deconstruct(
                    copyable_any::unpack<RecvPacketParams>(value)
                );
            on_recv_packet(pack);
        } else if (type_name_output
            == std::type_info::type_name<RecvIntentPacketParams>()) {
            let (pack) =
                on_recv_intent_packet_deconstruct(
                    copyable_any::unpack<RecvIntentPacketParams>(value)
                );
            on_recv_intent_packet(pack);
        } else if (type_name_output
            == std::type_info::type_name<AcknowledgePacketParams>()) {
            let (pack, acknowledgement) =
                on_acknowledge_packet_deconstruct(
                    copyable_any::unpack<AcknowledgePacketParams>(value)
                );
            on_acknowledge_packet(pack, acknowledgement);
        } else if (type_name_output == std::type_info::type_name<TimeoutPacketParams>()) {
            let (pack) =
                on_timeout_packet_deconstruct(
                    copyable_any::unpack<TimeoutPacketParams>(value)
                );
            on_timeout_packet(pack);
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
        packet: Packet, acknowledgement: vector<u8>
    ): copyable_any::Any {
        copyable_any::pack<AcknowledgePacketParams>(
            AcknowledgePacketParams { packet, acknowledgement }
        )
    }

    public fun pack_timeout_packet_params(packet: Packet): copyable_any::Any {
        copyable_any::pack<TimeoutPacketParams>(TimeoutPacketParams { packet })
    }

    public fun pack_recv_packet_params(packet: Packet): copyable_any::Any {
        copyable_any::pack<RecvPacketParams>(RecvPacketParams { packet })
    }

    public fun pack_recv_intent_packet_params(packet: Packet): copyable_any::Any {
        copyable_any::pack<RecvIntentPacketParams>(RecvIntentPacketParams { packet })
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

    public fun on_recv_packet_deconstruct(recv_param: RecvPacketParams): Packet {
        let pack = get_packet_from_recv_param(&recv_param);
        *pack
    }

    public fun on_recv_intent_packet_deconstruct(
        recv_intent_param: RecvIntentPacketParams
    ): Packet {
        let pack = get_packet_from_recv_intent_param(&recv_intent_param);
        *pack
    }

    public fun on_acknowledge_packet_deconstruct(
        ack_param: AcknowledgePacketParams
    ): (Packet, vector<u8>) {
        let pack = get_packet_from_ack_param(&ack_param);
        let acknowledgement = get_acknowledgement_from_ack_param(&ack_param);
        (*pack, *acknowledgement)
    }

    public fun on_timeout_packet_deconstruct(
        timeout_param: TimeoutPacketParams
    ): Packet {
        let pack = get_packet_from_timeout_param(&timeout_param);
        *pack
    }

    public fun on_channel_open_init_deconstruct(
        init_param: ChannelOpenInitParams
    ): (u32, u32, String) {
        let connection_id = get_connection_id_from_channel_open_init_param(&init_param);
        let channel_id = get_channel_id_from_channel_open_init_param(&init_param);
        let version = get_version_from_channel_open_init_param(&init_param);
        (connection_id, channel_id, *version)
    }

    public fun on_channel_open_try_deconstruct(
        try_param: ChannelOpenTryParams
    ): (u32, u32, u32, String, String) {
        let connection_id = get_connection_id_from_channel_open_try_param(&try_param);
        let channel_id = get_channel_id_from_channel_open_try_param(&try_param);
        let counterparty_channel_id =
            get_counterparty_channel_id_from_channel_open_try_param(&try_param);
        let version = get_version_from_channel_open_try_param(&try_param);
        let counterparty_version =
            get_counterparty_version_from_channel_open_try_param(&try_param);
        (
            connection_id,
            channel_id,
            counterparty_channel_id,
            *version,
            *counterparty_version
        )
    }

    public fun on_channel_open_ack_deconstruct(
        ack_param: ChannelOpenAckParams
    ): (u32, u32, String) {
        let channel_id = get_channel_id_from_channel_open_ack_param(&ack_param);
        let counterparty_version =
            get_counterparty_version_from_channel_open_ack_param(&ack_param);
        let counterparty_channel_id =
            get_counterparty_channel_id_from_channel_open_ack_param(&ack_param);
        (channel_id, counterparty_channel_id, *counterparty_version)
    }

    public fun on_channel_open_confirm_deconstruct(
        confirm_param: ChannelOpenConfirmParams
    ): u32 {
        let channel_id = get_channel_id_from_channel_open_confirm_param(&confirm_param);
        channel_id
    }

    public fun on_channel_close_init_deconstruct(
        close_init_param: ChannelCloseInitParams
    ): u32 {
        let channel_id = get_channel_id_from_channel_close_init_param(&close_init_param);
        channel_id
    }

    public fun on_channel_close_confirm_deconstruct(
        close_confirm_param: ChannelCloseConfirmParams
    ): u32 {
        let channel_id =
            get_channel_id_from_channel_close_confirm_param(&close_confirm_param);
        channel_id
    }
}
