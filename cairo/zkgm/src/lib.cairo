pub mod event;
pub mod isolver;
pub mod izkgmerc20;
pub mod types;

const TOKEN_ORDER_KIND_INITIALIZE: u8 = 0x00;
const TOKEN_ORDER_KIND_ESCROW: u8 = 0x01;
const TOKEN_ORDER_KIND_UNESCROW: u8 = 0x02;
const TOKEN_ORDER_KIND_SOLVE: u8 = 0x03;

const WRAPPED_TOKEN_KIND_PROTOCOL: u8 = 0x00;
const WRAPPED_TOKEN_KIND_THIRD_PARTY: u8 = 0x01;

const FILL_TYPE_PROTOCOL: u256 = 0xB0CAD0;
const FILL_TYPE_MARKETMAKER: u256 = 0xD1CEC45E;

const INSTR_VERSION_0: u8 = 0x00;
const INSTR_VERSION_1: u8 = 0x01;
const INSTR_VERSION_2: u8 = 0x02;

#[starknet::contract]
mod Ucs03Zkgm {
    use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
    use alexandria_math::bitmap::Bitmap;
    use alexandria_math::opt_math::OptBitShift;
    use core::hash::HashStateTrait;
    use core::pedersen::PedersenTrait;
    use ibc::types::{ChannelId, Id, Packet};
    use openzeppelin::interfaces::erc20::{IERC20Dispatcher, IERC20DispatcherTrait};
    use starknet::storage::{Map, StorageMapReadAccess, StorageMapWriteAccess};
    use starknet::syscalls::{deploy_syscall, get_class_hash_at_syscall};
    use starknet::{ContractAddress, SyscallResultTrait, get_caller_address};
    use crate::event::CreateWrappedToken;
    use crate::isolver::{ISolverDispatcher, ISolverDispatcherTrait};
    use crate::izkgmerc20::{IZkgmERC20Dispatcher, IZkgmERC20DispatcherTrait};
    use crate::types::{
        Instruction, Opcode, SolverMetadata, TokenMetadata, TokenOrderAck, TokenOrderV2, Version,
        ZkgmPacket, ethabi_decode, ethabi_encode,
    };
    use crate::{
        FILL_TYPE_MARKETMAKER, FILL_TYPE_PROTOCOL, TOKEN_ORDER_KIND_ESCROW,
        TOKEN_ORDER_KIND_INITIALIZE, TOKEN_ORDER_KIND_SOLVE, TOKEN_ORDER_KIND_UNESCROW,
        WRAPPED_TOKEN_KIND_THIRD_PARTY,
    };

    #[storage]
    struct Storage {
        token_metadata_image_to_preimage: Map<u256, TokenMetadata>,
        metadata_image_of: Map<ContractAddress, u256>,
        token_origin: Map<ContractAddress, u256>,
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
                return Self::market_maker_fill(
                    packet, relayer, relayer_msg, path, quote_token, receiver, order, intent,
                );
            }

            let base_amount_covers_quote_amount = order.base_amount >= order.quote_amount;

            if order.kind == TOKEN_ORDER_KIND_UNESCROW
                && base_amount_covers_quote_amount { // TODO(aeryz): rate limit?
                return self
                    .protocol_fill_unescrow(
                        packet.destination_channel_id,
                        path,
                        order.base_token,
                        quote_token,
                        receiver,
                        relayer,
                        order.base_amount,
                        order.quote_amount,
                    );
            } else {
                let (wrapped_token, _) = if order
                    .kind == TOKEN_ORDER_KIND_ESCROW { // (wrapped_token, wrapped_token_salt) =
                    let metadata_image = self.metadata_image_of.read(quote_token);
                    self
                        .predict_wrapped_token_from_metadata_image(
                            path,
                            packet.destination_channel_id,
                            order.base_token.clone(),
                            metadata_image,
                        )
                } else if order.kind == TOKEN_ORDER_KIND_INITIALIZE {
                    let metadata: TokenMetadata = ethabi_decode(order.metadata.clone()).unwrap();

                    let (wrapped_token, wrapped_token_salt, calldata) = Self::predict_wrapped_token(
                        path,
                        packet.destination_channel_id,
                        order.base_token.clone(),
                        metadata.clone(),
                        true,
                    );

                    if quote_token != wrapped_token {
                        // ErrInvalidTokenOrderKind
                        return Err(());
                    }

                    self
                        .deploy_wrapped_token(
                            packet.destination_channel_id,
                            path,
                            order.base_token.clone(),
                            wrapped_token,
                            wrapped_token_salt,
                            metadata,
                            calldata,
                            true,
                        )?;

                    (wrapped_token, wrapped_token_salt)
                    //
                } else {
                    return Err(());
                };

                if quote_token == wrapped_token && base_amount_covers_quote_amount {
                    // TODO(aeryz): rate limit
                    self
                        .protocol_fill_mint(
                            packet.destination_channel_id,
                            path,
                            wrapped_token,
                            receiver,
                            relayer,
                            order.base_amount,
                            order.quote_amount,
                        )
                } else {
                    Self::market_maker_fill(
                        packet, relayer, relayer_msg, path, quote_token, receiver, order, intent,
                    )
                }
            }
        }

        fn predict_wrapped_token(
            path: u256,
            channel: ChannelId,
            token: ByteArray,
            metadata: TokenMetadata,
            with_calldata: bool,
        ) -> (ContractAddress, felt252, Array<felt252>) {
            let metadata_image = ethabi_encode(@metadata).keccak_be();

            Self::predict_wrapped_token_from_metadata_and_image(
                path, channel, token, metadata, metadata_image, with_calldata,
            )
        }

        fn predict_wrapped_token_from_metadata_and_image(
            path: u256,
            channel: ChannelId,
            token: ByteArray,
            metadata: TokenMetadata,
            metadata_image: u256,
            with_calldata: bool,
        ) -> (ContractAddress, felt252, Array<felt252>) {
            const PATRICIA_KEY_UPPER_BOUND: u256 =
                0x800000000000000000000000000000000000000000000000000000000000000;
            const CONTRACT_ADDRESS_DOMAIN_SIZE: u256 = PATRICIA_KEY_UPPER_BOUND;
            const L2_ADDRESS_UPPER_BOUND: u256 = CONTRACT_ADDRESS_DOMAIN_SIZE - 256;

            let (_, class_hash) = metadata.implementation.read_felt252(0);

            let mut offset = 0;
            let mut calldata_hash = PedersenTrait::new(0);
            let mut calldata: Array<felt252> = Default::default();
            while offset < metadata.initializer.len() {
                let (o, i) = metadata.initializer.read_felt252(offset);
                offset = o;
                calldata_hash = calldata_hash.update(i);
                if (with_calldata) {
                    calldata.append(i);
                }
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

            (address.try_into().unwrap(), salt, calldata)
        }

        fn deploy_wrapped_token(
            ref self: ContractState,
            channel_id: ChannelId,
            path: u256,
            unwrapped_token: ByteArray,
            wrapped_token: ContractAddress,
            wrapped_token_salt: felt252,
            metadata: TokenMetadata,
            calldata: Array<felt252>,
            can_deploy: bool,
        ) -> Result<(), ()> {
            if (get_class_hash_at_syscall(wrapped_token).is_err()) {
                if (!can_deploy) {
                    // revert ZkgmLib.ErrCannotDeploy();
                    return Err(());
                }

                // aka `class_hash`
                let (_, implementation) = metadata.implementation.read_felt252(0);

                let (contract_addr, _) = deploy_syscall(
                    implementation.try_into().unwrap(), wrapped_token_salt, calldata.into(), true,
                )
                    .unwrap_syscall();

                if contract_addr != wrapped_token {
                    // invalid?
                    return Err(());
                }

                self
                    .token_origin
                    .write(wrapped_token, Self::update_channel_path(path, channel_id)?);

                // TODO(aeryz):
                // bytes memory encodedMetadata = ZkgmLib.encodeTokenMetadata(metadata);
                // metadataImageOf[wrappedToken] = EfficientHashLib.hash(encodedMetadata);

                self.metadata_image_of.write(wrapped_token, Default::default());

                let kind = WRAPPED_TOKEN_KIND_THIRD_PARTY;

                // TODO(aeryz): do we have a similar thing with native starknet token?

                // if (implementation == address(ERC20_IMPL)) {
                //     try this.decodeZkgmERC20InitializeCall(metadata.initializer)
                //     returns (
                //         address tokenAuthority,
                //         address tokenMinter,
                //         string memory,
                //         string memory,
                //         uint8
                //     ) {
                //         if (
                //             tokenAuthority == authority()
                //                 && tokenMinter == address(this)
                //         ) {
                //             kind = ZkgmLib.WRAPPED_TOKEN_KIND_PROTOCOL;
                //         }
                //     } catch {}
                // }

                self
                    .emit(
                        CreateWrappedToken {
                            path,
                            channel_id,
                            base_token: unwrapped_token,
                            quote_token: wrapped_token,
                            metadata: Default::default(),
                            kind,
                        },
                    )
            }
            Ok(())
        }

        fn update_channel_path(path: u256, next_channel_id: ChannelId) -> Result<u256, ()> {
            let next_channel_id = next_channel_id.raw().into();
            if path == 0 {
                return Ok(next_channel_id);
            }

            // unwrap since this is guaranteed to be nonzero
            let next_hop_index = Bitmap::most_significant_bit(path).unwrap() / 32 + 1;
            if next_hop_index > 7 {
                // InvalidHops
                return Err(());
            }

            Ok((OptBitShift::shl(next_channel_id, 32) & next_hop_index.into()) | path)
        }

        fn predict_wrapped_token_from_metadata_image(
            self: @ContractState,
            path: u256,
            channel: ChannelId,
            token: ByteArray,
            metadata_image: u256,
        ) -> (ContractAddress, felt252) {
            let metadata = self.token_metadata_image_to_preimage.read(metadata_image);

            let (ca, salt, _) = Self::predict_wrapped_token_from_metadata_and_image(
                path, channel, token, metadata, metadata_image, false,
            );

            (ca, salt)
        }

        fn market_maker_fill(
            packet: ibc::types::Packet,
            relayer: ContractAddress,
            relayer_msg: ByteArray,
            path: u256,
            quote_token: ContractAddress,
            receiver: ContractAddress,
            order: TokenOrderV2,
            intent: bool,
        ) -> Result<ByteArray, ()> {
            if order.kind == TOKEN_ORDER_KIND_SOLVE {
                Self::solver_fill(packet, relayer, relayer_msg, path, order, intent)
            } else {
                let quote_amount = order.quote_amount;

                // TODO(aeryz): NATIVE_TOKEN_ERC_7528_ADDRESS?
                if quote_amount > 0
                    && !(IERC20Dispatcher { contract_address: quote_token }
                        .transfer_from(get_caller_address(), receiver, quote_amount)) {
                    // onlymaker
                    return Err(());
                }

                Ok(
                    ethabi_encode(
                        @TokenOrderAck {
                            fill_type: FILL_TYPE_MARKETMAKER, market_maker: relayer_msg,
                        },
                    ),
                )
            }
        }

        fn solver_fill(
            packet: ibc::types::Packet,
            relayer: ContractAddress,
            relayer_msg: ByteArray,
            path: u256,
            order: TokenOrderV2,
            intent: bool,
        ) -> Result<ByteArray, ()> {
            let metadata: SolverMetadata = ethabi_decode(order.metadata.clone())?;

            let (_, solver) = metadata.solver_address.read_felt252(0);

            let ret = ISolverDispatcher { contract_address: solver.try_into().unwrap() }
                .solve(packet, order, path, get_caller_address(), relayer, relayer_msg, intent);
            match ret {
                Ok(ret) => {
                    Ok(
                        ethabi_encode(
                            @TokenOrderAck { fill_type: FILL_TYPE_MARKETMAKER, market_maker: ret },
                        ),
                    )
                },
                Err(_) => Err(()),
            }
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

        fn protocol_fill_mint(
            ref self: ContractState,
            channel_id: ChannelId,
            path: u256,
            wrapped_token: ContractAddress,
            receiver: ContractAddress,
            relayer: ContractAddress,
            base_amount: u256,
            quote_amount: u256,
        ) -> Result<ByteArray, ()> {
            let fee = base_amount - quote_amount;
            if quote_amount > 0 {
                IZkgmERC20Dispatcher { contract_address: wrapped_token }
                    .mint(receiver, quote_amount);
            }

            if fee > 0 {
                IZkgmERC20Dispatcher { contract_address: wrapped_token }.mint(relayer, fee);
            }

            Ok(
                ethabi_encode(
                    @TokenOrderAck {
                        fill_type: FILL_TYPE_PROTOCOL, market_maker: Default::default(),
                    },
                ),
            )
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

            let mut implementation: ByteArray = Default::default();
            implementation.append_felt252((*contract.class_hash).try_into().unwrap());

            let mut initializer = Default::default();
            for o in @out {
                initializer.append_felt252(*o);
            }

            let metadata = TokenMetadata { initializer, implementation };

            let (address, salt, _) = Ucs03ZkgmTrait::predict_wrapped_token_from_metadata_and_image(
                0, Id::new(1_u32.try_into().unwrap()), "AAAAAA", metadata, 100, false,
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
