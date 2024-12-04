module ibc::helpers {
    use ibc::ibc;
    use ibc::packet::Packet;
    use std::vector;
    use std::copyable_any;

    public fun on_recv_packet_deconstruct(
        recv_param: ibc::RecvPacketParams
    ): Packet {
        let pack = ibc::get_packet_from_recv_param(&recv_param);
        *pack
    }

    public fun on_recv_intent_packet_deconstruct(
        recv_intent_param: ibc::RecvIntentPacketParams
    ): Packet {
        let pack = ibc::get_packet_from_recv_intent_param(&recv_intent_param);
        *pack
    }

    public fun on_acknowledge_packet_deconstruct(
        ack_param: ibc::AcknowledgePacketParams
    ): (Packet, vector<u8>) {
        let pack = ibc::get_packet_from_ack_param(&ack_param);
        let acknowledgement =
            ibc::get_acknowledgement_from_ack_param(&ack_param);
        (*pack, *acknowledgement)
    }

    public fun on_timeout_packet_deconstruct(
        timeout_param: ibc::TimeoutPacketParams
    ): Packet {
        let pack = ibc::get_packet_from_timeout_param(&timeout_param);
        *pack
    }

    public fun on_channel_open_init_deconstruct(
        init_param: ibc::ChannelOpenInitParams
    ): (u8, u32, u32, vector<u8>) {
        let ordering =
            ibc::get_ordering_from_channel_open_init_param(&init_param);
        let connection_id =
            ibc::get_connection_id_from_channel_open_init_param(&init_param);
        let channel_id =
            ibc::get_channel_id_from_channel_open_init_param(&init_param);
        let version = ibc::get_version_from_channel_open_init_param(&init_param);
        (ordering, connection_id, channel_id, *version)
    }

    public fun on_channel_open_try_deconstruct(
        try_param: ibc::ChannelOpenTryParams
    ): (u8, u32, u32, u32, vector<u8>, vector<u8>) {
        let ordering = ibc::get_ordering_from_channel_open_try_param(&try_param);
        let connection_id =
            ibc::get_connection_id_from_channel_open_try_param(&try_param);
        let channel_id =
            ibc::get_channel_id_from_channel_open_try_param(&try_param);
        let counterparty_channel_id =
            ibc::get_counterparty_channel_id_from_channel_open_try_param(
                &try_param
            );
        let version = ibc::get_version_from_channel_open_try_param(&try_param);
        let counterparty_version =
            ibc::get_counterparty_version_from_channel_open_try_param(&try_param);
        (
            ordering,
            connection_id,
            channel_id,
            counterparty_channel_id,
            *version,
            *counterparty_version
        )
    }

    public fun on_channel_open_ack_deconstruct(
        ack_param: ibc::ChannelOpenAckParams
    ): (u32, u32, vector<u8>) {
        let channel_id =
            ibc::get_channel_id_from_channel_open_ack_param(&ack_param);
        let counterparty_version =
            ibc::get_counterparty_version_from_channel_open_ack_param(&ack_param);
        let counterparty_channel_id =
            ibc::get_counterparty_channel_id_from_channel_open_ack_param(
                &ack_param
            );
        (channel_id, counterparty_channel_id, *counterparty_version)
    }

    public fun on_channel_open_confirm_deconstruct(
        confirm_param: ibc::ChannelOpenConfirmParams
    ): u32 {
        let channel_id =
            ibc::get_channel_id_from_channel_open_confirm_param(&confirm_param);
        channel_id
    }

    public fun on_channel_close_init_deconstruct(
        close_init_param: ibc::ChannelCloseInitParams
    ): u32 {
        let channel_id =
            ibc::get_channel_id_from_channel_close_init_param(&close_init_param);
        channel_id
    }

    public fun on_channel_close_confirm_deconstruct(
        close_confirm_param: ibc::ChannelCloseConfirmParams
    ): u32 {
        let channel_id =
            ibc::get_channel_id_from_channel_close_confirm_param(
                &close_confirm_param
            );
        channel_id
    }
}
