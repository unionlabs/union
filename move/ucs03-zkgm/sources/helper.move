module zkgm::zkgm_helpers {
    use ibc::packet::Packet;
    use std::copyable_any;

    struct RecvPacketParamsZKGM has copy, drop, store {
        packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>
    }

    struct AcknowledgePacketParamsZKGM has copy, drop, store {
        packet: Packet,
        acknowledgement: vector<u8>,
        relayer: address
    }

    struct TimeoutPacketParamsZKGM has copy, drop, store {
        packet: Packet,
        relayer: address
    }

    public fun pack_acknowledge_packet_params_zkgm(
        packet: Packet, acknowledgement: vector<u8>, relayer: address
    ): copyable_any::Any {
        copyable_any::pack<AcknowledgePacketParamsZKGM>(
            AcknowledgePacketParamsZKGM { packet, acknowledgement, relayer }
        )
    }

    public fun pack_timeout_packet_params_zkgm(
        packet: Packet, relayer: address
    ): copyable_any::Any {
        copyable_any::pack<TimeoutPacketParamsZKGM>(
            TimeoutPacketParamsZKGM { packet, relayer }
        )
    }

    // Getter for RecvPacketParams
    public fun get_packet_from_recv_param_zkgm(
        param: &RecvPacketParamsZKGM
    ): &Packet {
        &param.packet
    }

    public fun get_relayer_from_recv_param_zkgm(
        param: &RecvPacketParamsZKGM
    ): address {
        param.relayer
    }

    public fun get_relayer_msg_from_recv_param_zkgm(
        param: &RecvPacketParamsZKGM
    ): &vector<u8> {
        &param.relayer_msg
    }

    // Getters for AcknowledgePacketParams
    public fun get_packet_from_ack_param_zkgm(
        param: &AcknowledgePacketParamsZKGM
    ): &Packet {
        &param.packet
    }

    public fun get_acknowledgement_from_ack_param_zkgm(
        param: &AcknowledgePacketParamsZKGM
    ): &vector<u8> {
        &param.acknowledgement
    }

    public fun get_relayer_from_ack_param_zkgm(
        param: &AcknowledgePacketParamsZKGM
    ): address {
        param.relayer
    }

    // Getter for TimeoutPacketParams
    public fun get_packet_from_timeout_param_zkgm(
        param: &TimeoutPacketParamsZKGM
    ): &Packet {
        &param.packet
    }

    public fun get_relayer_from_timeout_param_zkgm(
        param: &TimeoutPacketParamsZKGM
    ): address {
        param.relayer
    }

    public fun on_recv_packet_zkgm_deconstruct(
        recv_param: RecvPacketParamsZKGM
    ): (Packet, address, vector<u8>) {
        let pack = get_packet_from_recv_param_zkgm(&recv_param);
        let relayer = get_relayer_from_recv_param_zkgm(&recv_param);
        let relayer_msg = get_relayer_msg_from_recv_param_zkgm(&recv_param);
        (*pack, relayer, *relayer_msg)
    }

    public fun on_acknowledge_packet_deconstruct_zkgm(
        ack_param: AcknowledgePacketParamsZKGM
    ): (Packet, vector<u8>, address) {
        let pack = get_packet_from_ack_param_zkgm(&ack_param);
        let acknowledgement = get_acknowledgement_from_ack_param_zkgm(&ack_param);
        let relayer = get_relayer_from_ack_param_zkgm(&ack_param);
        (*pack, *acknowledgement, relayer)
    }

    public fun on_timeout_packet_deconstruct_zkgm(
        timeout_param: TimeoutPacketParamsZKGM
    ): (Packet, address) {
        let pack = get_packet_from_timeout_param_zkgm(&timeout_param);
        let relayer = get_relayer_from_timeout_param_zkgm(&timeout_param);
        (*pack, relayer)
    }
}
