       if ."@type" == "data" then
                ."@value" as $data |

                if $data."@type" == "ibc_event" then
                    $data."@value".chain_id as $chain_id |
                    $data."@value".event."@type" as $event_type |
                    $data."@value".event."@value" as $event |

                    if $event_type == "send_packet" then
                        (
                ($chain_id | test(".*"))
                and (
                    ($event.packet_src_port | test("^wasm.union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h$"))
                    or ($event.packet_dst_port | test("^wasm.union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h$"))
                )
                and (
                    ($event.packet_src_channel | test(".*"))
                    or ($event.packet_dst_channel | test(".*"))
                )
                and ($event.connection_id | test(".*"))
            )
                    elif $event_type == "recv_packet" then
                        (
                ($chain_id | test(".*"))
                and (
                    ($event.packet_src_port | test("^wasm.union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h$"))
                    or ($event.packet_dst_port | test("^wasm.union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h$"))
                )
                and (
                    ($event.packet_src_channel | test(".*"))
                    or ($event.packet_dst_channel | test(".*"))
                )
                and ($event.connection_id | test(".*"))
            )
                    elif $event_type == "write_acknowledgement" then
                        (
                ($chain_id | test(".*"))
                and (
                    ($event.packet_src_port | test("^wasm.union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h$"))
                    or ($event.packet_dst_port | test("^wasm.union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h$"))
                )
                and (
                    ($event.packet_src_channel | test(".*"))
                    or ($event.packet_dst_channel | test(".*"))
                )
                and ($event.connection_id | test(".*"))
            )

                    elif $event_type == "channel_open_init" then
                        (
                ($chain_id | test(".*"))
                and (
                    ($event.port_id | test(".*"))
                    or ($event.counterparty_port_id | test(".*"))
                )
                and ($event.connection_id | test(".*"))
            )
                    elif $event_type == "channel_open_try" then
                        (
                ($chain_id | test(".*"))
                and (
                    ($event.port_id | test(".*"))
                    or ($event.counterparty_port_id | test(".*"))
                )
                and ($event.connection_id | test(".*"))
            )
                    elif $event_type == "channel_open_ack" then
                        (
                ($chain_id | test(".*"))
                and (
                    ($event.port_id | test(".*"))
                    or ($event.counterparty_port_id | test(".*"))
                )
                and ($event.connection_id | test(".*"))
            )

                    elif $event_type == "connection_open_init" then
                        (
                ($chain_id | test(".*"))
                and (
                    ($event.client_id | test(".*"))
                    or ($event.counterparty_client_id | test(".*"))
                )
            )
                    elif $event_type == "connection_open_try" then
                        (
                ($chain_id | test(".*"))
                and (
                    ($event.client_id | test(".*"))
                    or ($event.counterparty_client_id | test(".*"))
                )
            )
                    elif $event_type == "connection_open_ack" then
                        (
                ($chain_id | test(".*"))
                and (
                    ($event.client_id | test(".*"))
                    or ($event.counterparty_client_id | test(".*"))
                )
            )
                    end
                else
                    false
                end
            else
                false
            end
