// #[test_only]
// module IBC::IBCCommitmentTest {

//     use std::string;
//     use IBC::IBCCommitment;
//     use IBC::height;

//     const E_TEST_CHANNEL_COMMITMENT_KEY: u64 = 4001;
//     const E_TEST_CHANNEL_PATH: u64 = 4002;
//     const E_TEST_CLIENT_STATE_COMMITMENT_KEY: u64 = 4003;
//     const E_TEST_CONSENSUS_STATE_COMMITMENT_KEY: u64 = 4004;
//     const E_TEST_CONNECTION_COMMITMENT_KEY: u64 = 4005;
//     const E_TEST_PACKET_COMMITMENT_KEY: u64 = 4006;
//     const E_TEST_PACKET_ACKNOWLEDGEMENT_COMMITMENT_KEY: u64 = 4007;
//     const E_TEST_PACKET_RECEIPT_COMMITMENT_KEY: u64 = 4008;
//     const E_TEST_NEXT_SEQUENCE_SEND_COMMITMENT_KEY: u64 = 4009;
//     const E_TEST_NEXT_SEQUENCE_RECV_COMMITMENT_KEY: u64 = 4010;
//     const E_TEST_NEXT_SEQUENCE_ACK_COMMITMENT_KEY: u64 = 4011;

//     #[test]
//     public fun test_channel_path() {
//         let test_port = string::utf8(b"test_port");
//         let test_channel = string::utf8(b"test_channel");
//         let val = IBCCommitment::channel_path(test_port, test_channel);

//         assert!(val == string::utf8(b"channelEnds/ports/test_port/channels/test_channel"), E_TEST_CHANNEL_PATH);
//     }

//     #[test]
//     public fun test_channel_commitment_key() {
//         let test_port = string::utf8(b"test_port");
//         let test_channel = string::utf8(b"test_channel");
//         let val = IBCCommitment::channel_commitment_key(test_port, test_channel);
//         assert!(val == b"\x2e\xd3\x64\x82\xea\x34\x7a\xac\x52\xa9\x99\xab\x83\xdb\xb2\xed\x3a\x9d\xa9\xb8\xdc\x96\x14\x21\x9d\xa4\xe3\xc2\xda\x35\x7e\xc2",
//                 E_TEST_CHANNEL_COMMITMENT_KEY);
//     }

//     #[test]
//     public fun test_client_state_commitment_key() {
//         let client_id = string::utf8(b"client_id");
//         let key = IBCCommitment::client_state_commitment_key(client_id);

//         assert!(key == b"\xdd\x84\x7b\x9d\x42\x7b\xcc\x72\x55\x8c\xa9\xb7\x97\xbd\xa8\xbe\x80\x26\xa4\xfa\xcc\xc6\x2f\x82\x71\x72\x03\x67\x3c\xe6\xb9\xb2",
//                 E_TEST_CLIENT_STATE_COMMITMENT_KEY);
//     }

//     #[test]
//     public fun test_consensus_state_commitment_key() {
//         let client_id = string::utf8(b"client_id");
//         let key = IBCCommitment::consensus_state_commitment_key(client_id, height::new(1, 1));

//         assert!(key == b"\x96\xfd\xc8\xff\xf2\x8c\xda\xd1\xf9\x0d\x12\x71\x7a\x66\x7c\xa9\x04\x8f\x08\x15\xb3\x80\xbe\x6a\xba\x05\x8f\xc2\xde\x22\xcb\xf3",
//                 E_TEST_CONSENSUS_STATE_COMMITMENT_KEY);
//     }

//     #[test]
//     public fun test_connection_commitment_key() {
//         let connection_id = string::utf8(b"connection_id");
//         let key = IBCCommitment::connection_commitment_key(connection_id);

//         assert!(key == b"\x87\xbf\x3a\x39\xaf\x2c\xc2\x00\x29\xab\x91\x8d\x40\x74\x95\xe2\x7e\xf9\x2b\xb9\xcd\x54\xe7\xab\xca\xe4\xc9\xd3\x31\xf5\xde\xfe",
//                 E_TEST_CONNECTION_COMMITMENT_KEY);
//     }

//     #[test]
//     public fun test_packet_commitment_key() {
//         let port_id = string::utf8(b"port_id");
//         let channel_id = string::utf8(b"channel_id");
//         let key = IBCCommitment::packet_commitment_key(port_id, channel_id, 1);

//         assert!(key == b"\x71\xda\x69\xec\x4a\xc8\x25\x54\xd9\xf6\x68\xba\x8f\xce\x24\x6a\xb4\x3f\xaa\x68\xed\xdd\x9b\x09\xd5\x88\xfc\x28\x0d\xb0\x23\x9e",
//                 E_TEST_PACKET_COMMITMENT_KEY);
//     }

//     #[test]
//     public fun test_packet_acknowledgement_commitment_key() {
//         let port_id = string::utf8(b"port_id");
//         let channel_id = string::utf8(b"channel_id");
//         let key = IBCCommitment::packet_acknowledgement_commitment_key(port_id, channel_id, 1);

//         assert!(key == b"\x97\x83\x34\x32\x54\xe7\x54\xb4\x1f\x2a\xf9\x54\xbc\xf1\xa6\x14\x8b\x73\xcb\xf5\x57\x63\xea\x21\x4c\x31\x5e\x54\xd5\xae\xd3\xb8",
//                 E_TEST_PACKET_ACKNOWLEDGEMENT_COMMITMENT_KEY);
//     }

//     #[test]
//     public fun test_packet_receipt_commitment_key() {
//         let port_id = string::utf8(b"port_id");
//         let channel_id = string::utf8(b"channel_id");
//         let key = IBCCommitment::packet_receipt_commitment_key(port_id, channel_id, 1);

//         assert!(key == b"\x2e\xad\x16\x16\xe7\x4e\xd6\x4d\x0f\x90\xd8\x08\x60\xd3\xdc\x7b\xc6\x0e\x7a\x30\xa1\xbb\x5c\xd1\x18\x1d\x18\x6a\xe6\x6b\xd6\xcb",
//                 E_TEST_PACKET_RECEIPT_COMMITMENT_KEY);
//     }

//     #[test]
//     public fun test_next_sequence_send_commitment_key() {
//         let port_id = string::utf8(b"port_id");
//         let channel_id = string::utf8(b"channel_id");
//         let key = IBCCommitment::next_sequence_send_commitment_key(port_id, channel_id);

//         assert!(key == b"\x36\x8d\x19\x1a\xb0\x65\x8e\xa0\x31\x42\xee\xd5\xac\xdd\x6a\x66\x91\xa7\x26\xe7\x2e\x8e\xc8\xfe\x0d\x52\xff\x4e\x7d\xa8\x0f\x5a",
//                 E_TEST_NEXT_SEQUENCE_SEND_COMMITMENT_KEY);
//     }

//     #[test]
//     public fun test_next_sequence_recv_commitment_key() {
//         let port_id = string::utf8(b"port_id");
//         let channel_id = string::utf8(b"channel_id");
//         let key = IBCCommitment::next_sequence_recv_commitment_key(port_id, channel_id);

//         assert!(key == b"\x35\xa5\x63\x70\xc0\x18\xdd\xae\xac\xa7\x44\x44\xc3\x7f\x33\x8c\x4a\xbd\x1b\x7b\xda\x70\xd5\xcc\x2c\x22\xcd\x17\xb9\xfc\x2d\x8c",
//                 E_TEST_NEXT_SEQUENCE_RECV_COMMITMENT_KEY);

//     }

//     #[test]
//     public fun test_next_sequence_ack_commitment_key() {
//         let port_id = string::utf8(b"port_id");
//         let channel_id = string::utf8(b"channel_id");
//         let key = IBCCommitment::next_sequence_ack_commitment_key(port_id, channel_id);

//         assert!(key == b"\xc6\x67\x96\x63\x96\xec\x78\x83\xbf\x29\x0b\x25\x2c\x28\x9f\xf9\xd0\xe2\x55\x9b\xea\xd7\xc3\x46\x16\x01\xef\xbc\xc8\x92\x66\x5b",
//                 E_TEST_NEXT_SEQUENCE_ACK_COMMITMENT_KEY);
//     }

// }
