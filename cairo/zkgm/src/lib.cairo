pub mod event;
pub mod types;

const TOKEN_ORDER_KIND_INITIALIZE: u8 = 0x00;
const TOKEN_ORDER_KIND_ESCROW: u8 = 0x01;
const TOKEN_ORDER_KIND_UNESCROW: u8 = 0x02;
const TOKEN_ORDER_KIND_SOLVE: u8 = 0x03;

const INSTR_VERSION_0: u8 = 0x00;
const INSTR_VERSION_1: u8 = 0x01;
const INSTR_VERSION_2: u8 = 0x02;

#[starknet::contract]
mod Ucs03Zkgm {
    use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
    use core::hash::HashStateTrait;
    use core::pedersen::PedersenTrait;
    use ibc::types::{ChannelId, Id, Packet};
    use starknet::ContractAddress;
    use starknet::storage::{Map, StorageMapReadAccess};
    use crate::TOKEN_ORDER_KIND_SOLVE;
    use crate::event::CreateWrappedToken;
    use crate::types::{
        Instruction, Opcode, TokenMetadata, TokenOrderV2, Version, ZkgmPacket, ethabi_decode,
        ethabi_encode,
    };

    #[storage]
    struct Storage {
        token_metadata_image_to_preimage: Map<u256, TokenMetadata>,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        CreateWrappedToken: CreateWrappedToken,
    }

    #[constructor]
    fn constructor(ref self: ContractState, _name: ByteArray, _symbol: ByteArray, _decimals: u8) {}

    #[abi(embed_v0)]
    impl Ucs03ZkgmIbcImpl of ibc::app::IIbcModuleRecv<ContractState> {
        fn on_recv_packet(
            ref self: ContractState,
            caller: ContractAddress,
            packet: Packet,
            relayer: ContractAddress,
            relayer_msg: ByteArray,
        ) -> ByteArray {
            Default::default()
        }

        fn on_recv_intent_packet(
            ref self: ContractState,
            caller: ContractAddress,
            packet: Packet,
            market_maker: ContractAddress,
            market_maker_msg: ByteArray,
        ) -> ByteArray {
            Default::default()
        }
    }

    #[generate_trait]
    impl Ucs03ZkgmImpl of Ucs03ZkgmTrait {
        fn process_receive(
            ref self: ContractState,
            packet: ibc::types::Packet,
            relayer: ContractAddress,
            relayer_msg: ByteArray,
            intent: bool,
        ) {
            let zkgm_packet: ZkgmPacket = ethabi_decode(packet.data.clone()).unwrap();

            let _ = self
                .execute(
                    packet,
                    relayer,
                    relayer_msg,
                    zkgm_packet.salt,
                    zkgm_packet.path,
                    zkgm_packet.instruction,
                    intent,
                )
                .unwrap();
        }

        fn execute(
            ref self: ContractState,
            packet: ibc::types::Packet,
            relayer: ContractAddress,
            relayer_msg: ByteArray,
            salt: ByteArray,
            path: u256,
            instruction: Instruction,
            intent: bool,
        ) -> Result<ByteArray, ()> {
            match (instruction.opcode, instruction.version) {
                (
                    Opcode::TokenOrder, Version::V2,
                ) => {
                    let order: TokenOrderV2 = ethabi_decode(instruction.operand).unwrap();

                    self.execute_token_order(packet, relayer, relayer_msg, path, order, intent)
                },
                _ => { Err(()) },
            }
        }

        fn execute_token_order(
            ref self: ContractState,
            packet: ibc::types::Packet,
            relayer: ContractAddress,
            relayer_msg: ByteArray,
            path: u256,
            order: TokenOrderV2,
            intent: bool,
        ) -> Result<ByteArray, ()> {
            let (_, quote_token) = order.quote_token.read_address(0);
            let (_, receiver) = order.receiver.read_address(0);

            if intent || order.kind == TOKEN_ORDER_KIND_SOLVE {
                return self
                    .market_maker_fill(
                        packet, relayer, relayer_msg, path, quote_token, receiver, order, intent,
                    );
            }

            // let base_amount_covers_quote_amount = order.base_amount >= order.quote_amount;

            // if order.kind == TOKEN_ORDER_KIND_UNESCROW
            //     && base_amount_covers_quote_amount { // TODO(aeryz): rate limit?
            //     return self
            //         .protocol_fill_unescrow(
            //             packet.destination_channel_id,
            //             path,
            //             order.base_token,
            //             quote_token,
            //             receiver,
            //             relayer,
            //             order.base_amount,
            //             order.quote_amount,
            //         );
            // } else {
            //     if order
            //         .kind == TOKEN_ORDER_KIND_ESCROW { // (wrapped_token, wrapped_token_salt) =
            //         predict_wrapped_token_from_metadata_image(
            //     //     path,
            //     //     packet.destination_channel_id,
            //     //     order.base_token,
            //     //     metadata_image
            //     // );
            //     } else if order.kind == TOKEN_ORDER_KIND_INITIALIZE {
            //         let metadata: TokenMetadata = ethabi_decode(order.metadata).unwrap();
            //     }
            // }

            Err(())
        }

        fn predict_wrapped_token(
            path: u256, channel: ChannelId, token: ByteArray, metadata: TokenMetadata,
        ) -> (ContractAddress, felt252) {
            let metadata_image = ethabi_encode(@metadata).keccak_be();

            Self::predict_wrapped_token_from_metadata_and_image(
                path, channel, token, metadata, metadata_image,
            )
        }

        fn predict_wrapped_token_from_metadata_and_image(
            path: u256,
            channel: ChannelId,
            token: ByteArray,
            metadata: TokenMetadata,
            metadata_image: u256,
        ) -> (ContractAddress, felt252) {
            const PATRICIA_KEY_UPPER_BOUND: u256 =
                0x800000000000000000000000000000000000000000000000000000000000000;
            const CONTRACT_ADDRESS_DOMAIN_SIZE: u256 = PATRICIA_KEY_UPPER_BOUND;
            const L2_ADDRESS_UPPER_BOUND: u256 = CONTRACT_ADDRESS_DOMAIN_SIZE - 256;

            let (_, class_hash) = metadata.implementation.read_felt252(0);

            let mut offset = 0;
            let mut calldata_hash = PedersenTrait::new(0);
            println!("initializer len bro: {}", metadata.initializer.len());
            while offset < metadata.initializer.len() {
                let (o, i) = metadata.initializer.read_felt252(offset);
                offset = o;
                calldata_hash = calldata_hash.update(i);
            }
            // NOTE(aeryz): `pedersen_hash_array` function always postfixes the values with
            // the given array length.
            // https://docs.rs/crate/starknet-types-core/latest/source/src/hash/pedersen.rs#23
            let calldata_hash = calldata_hash.update((offset / 32).try_into().unwrap()).finalize();

            let mut salt = PedersenTrait::new(0)
                .update(path.low.into())
                .update(path.high.into())
                .update(channel.raw().into())
                .update(metadata_image.low.try_into().unwrap())
                .update(metadata_image.high.try_into().unwrap());

            for i in token {
                salt = salt.update(i.into());
            }

            let salt = salt.finalize();

            let mut address = PedersenTrait::new(0)
                .update(0x535441524b4e45545f434f4e54524143545f41444452455353)
                .update(0) // deployer is empty
                .update(salt)
                .update(class_hash) // class hash
                .update(calldata_hash)
                // Postfix with the array length
                .update(5)
                .finalize();

            // TODO(aeryz): check this logic one more time
            // safu because its guaranteed to be smaller than the felt252::Max
            address = (address.into() % L2_ADDRESS_UPPER_BOUND).try_into().unwrap();

            (address.try_into().unwrap(), salt)
        }

        fn predict_wrapped_token_from_metadata_image(
            self: @ContractState,
            path: u256,
            channel: ChannelId,
            token: ByteArray,
            metadata_image: u256,
        ) -> (ContractAddress, felt252) {
            let metadata = self.token_metadata_image_to_preimage.read(metadata_image);

            Self::predict_wrapped_token_from_metadata_and_image(
                path, channel, token, metadata, metadata_image,
            )
        }

        fn market_maker_fill(
            ref self: ContractState,
            packet: ibc::types::Packet,
            relayer: ContractAddress,
            relayer_msg: ByteArray,
            path: u256,
            quote_token: ContractAddress,
            receiver: ContractAddress,
            order: TokenOrderV2,
            intent: bool,
        ) -> Result<ByteArray, ()> {
            Err(())
        }

        fn protocol_fill_unescrow(
            ref self: ContractState,
            channel_id: ChannelId,
            path: u256,
            base_token: ByteArray,
            quote_token: ContractAddress,
            receiver: ContractAddress,
            relayer: ContractAddress,
            base_amount: u256,
            quote_amount: u256,
        ) -> Result<ByteArray, ()> {
            Err(())
        }
    }

    #[cfg(test)]
    mod tests {
        use snforge_std::{DeclareResultTrait, declare};
        use starknet::syscalls::deploy_syscall;
        use super::{ByteArrayTraitExt, Id, TokenMetadata, Ucs03ZkgmTrait};

        #[derive(Serde, Drop)]
        struct ConstructorArgs {
            name: ByteArray,
            symbol: ByteArray,
            decimals: u8,
        }

        #[test]
        fn test_address_prediction_works() {
            let contract = declare("Ucs03Zkgm").unwrap().contract_class();

            let args = ConstructorArgs { name: "Union Token", symbol: "U", decimals: 18 };

            let mut out = array![];
            args.serialize(ref out);

            println!("out bro: {out:?}");

            let mut implementation: ByteArray = Default::default();
            implementation.append_felt252((*contract.class_hash).try_into().unwrap());

            let mut initializer = Default::default();
            for o in @out {
                initializer.append_felt252(*o);
            }

            let metadata = TokenMetadata { initializer, implementation };

            let (address, salt) = Ucs03ZkgmTrait::predict_wrapped_token_from_metadata_and_image(
                0, Id::new(1_u32.try_into().unwrap()), "AAAAAA", metadata, 100,
            );

            let (got, _) = deploy_syscall(*contract.class_hash, salt, out.into(), true).unwrap();

            assert!(got == address);
        }

        #[test]
        fn test_check_ctor() {
            let args = ConstructorArgs { name: "Union Token", symbol: "U", decimals: 18 };

            let mut out = array![];
            args.serialize(ref out);

            println!("{out:?}");
        }
    }
}
