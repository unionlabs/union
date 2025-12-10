// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.
//
// Parameters
//
// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's cairo subdirectory
//                       The Licensed Work is (c) 2025 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//
//
// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.
//
// Notice
//
// Business Source License 1.1
//
// Terms
//
// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.
//
// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.
//
// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.
//
// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.
//
// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.
//
// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.
//
// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).
//
// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

use core::hash::{Hash, HashStateTrait};
use starknet::ContractAddress;

#[derive(Drop, Clone)]
struct ChannelBalanceKey {
    source_channel_id: u32,
    path: u256,
    base_token: ContractAddress,
    quote_token: ByteArray,
}

impl ChannelBalanceKeyHashImpl<S, +HashStateTrait<S>, +Drop<S>> of Hash<ChannelBalanceKey, S> {
    fn update_state(mut state: S, value: ChannelBalanceKey) -> S {
        state = state
            .update(value.source_channel_id.into())
            .update(value.path.try_into().unwrap())
            .update(value.base_token.into());

        let mut encoded_quote_token = Default::default();
        value.quote_token.serialize(ref encoded_quote_token);

        for i in encoded_quote_token {
            state = state.update(i);
        }

        state
    }
}

#[starknet::contract]
mod Ucs03Zkgm {
    use alexandria_math::bitmap::Bitmap;
    use ibc::app::{IIbcModuleSendDispatcher, IIbcModuleSendDispatcherTrait};
    use ibc::types::{ChannelId, Packet};
    use openzeppelin::interfaces::erc20::{IERC20Dispatcher, IERC20DispatcherTrait};
    use starknet::storage::{
        Map, StorageMapReadAccess, StorageMapWriteAccess, StoragePointerReadAccess,
    };
    use starknet::syscalls::{deploy_syscall, get_class_hash_at_syscall};
    use starknet::{ContractAddress, SyscallResultTrait, get_caller_address, get_contract_address};
    use crate::event::CreateWrappedToken;
    use crate::interfaces::{
        ISolverDispatcher, ISolverDispatcherTrait, IZkgmERC20Dispatcher, IZkgmERC20DispatcherTrait,
    };
    use crate::types::{
        AckTrait, Instruction, Opcode, SolverMetadata, TokenMetadata, TokenOrderAck, TokenOrderV2,
        Version, ZkgmPacket, ethabi_decode, ethabi_encode,
    };
    use crate::{
        *, ACK_ERR_ONLYMAKER, FILL_TYPE_MARKETMAKER, FILL_TYPE_PROTOCOL, TOKEN_ORDER_KIND_ESCROW,
        TOKEN_ORDER_KIND_INITIALIZE, TOKEN_ORDER_KIND_SOLVE, TOKEN_ORDER_KIND_UNESCROW,
        WRAPPED_TOKEN_KIND_THIRD_PARTY, pop_channel_from_path, predict_wrapped_token,
        reverse_channel_path,
    };
    use super::ChannelBalanceKey;

    #[storage]
    struct Storage {
        ibc: ContractAddress,
        token_metadata_image_to_preimage: Map<u256, TokenMetadata>,
        metadata_image_of: Map<ContractAddress, u256>,
        token_origin: Map<ContractAddress, u256>,
        channel_balance: Map<ChannelBalanceKey, u256>,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        CreateWrappedToken: CreateWrappedToken,
    }

    #[constructor]
    fn constructor(ref self: ContractState, _name: ByteArray, _symbol: ByteArray, _decimals: u8) {}

    #[abi(embed_v0)]
    impl Ucs03ZkgmIbcImpl of ibc::app::IIbcModule<ContractState> {
        fn on_recv_packet(
            ref self: ContractState,
            caller: ContractAddress,
            packet: Packet,
            relayer: ContractAddress,
            relayer_msg: ByteArray,
        ) -> ByteArray {
            self.process_receive(packet, relayer, relayer_msg, false)
        }

        fn on_recv_intent_packet(
            ref self: ContractState,
            caller: ContractAddress,
            packet: Packet,
            market_maker: ContractAddress,
            market_maker_msg: ByteArray,
        ) -> ByteArray {
            self.process_receive(packet, market_maker, market_maker_msg, true)
        }

        fn on_chan_open_init(
            ref self: ContractState,
            caller: ContractAddress,
            connection_id: ibc::types::ConnectionId,
            channel_id: ChannelId,
            version: ByteArray,
            relayer: ContractAddress,
        ) {}

        fn on_chan_open_try(
            ref self: ContractState,
            caller: ContractAddress,
            connection_id: ibc::types::ConnectionId,
            channel_id: ChannelId,
            counterparty_channel_id: ChannelId,
            version: ByteArray,
            counterparty_version: ByteArray,
            relayer: ContractAddress,
        ) {}

        fn on_chan_open_ack(
            ref self: ContractState,
            caller: ContractAddress,
            channel_id: ChannelId,
            counterparty_channel_id: ChannelId,
            counterparty_version: ByteArray,
            relayer: ContractAddress,
        ) {}

        fn on_chan_open_confirm(
            ref self: ContractState,
            caller: ContractAddress,
            channel_id: ChannelId,
            relayer: ContractAddress,
        ) {}

        fn on_chan_close_init(
            ref self: ContractState,
            caller: ContractAddress,
            channel_id: ChannelId,
            relayer: ContractAddress,
        ) {}

        fn on_chan_close_confirm(
            ref self: ContractState,
            caller: ContractAddress,
            channel_id: ChannelId,
            relayer: ContractAddress,
        ) {}

        fn on_acknowledge_packet(
            ref self: ContractState,
            caller: ContractAddress,
            packet: Packet,
            acknowledgement: ByteArray,
            relayer: ContractAddress,
        ) {}

        fn on_timeout_packet(
            ref self: ContractState,
            caller: ContractAddress,
            packet: Packet,
            relayer: ContractAddress,
        ) {}
    }

    #[generate_trait]
    impl Ucs03ZkgmImpl of Ucs03ZkgmTrait {
        fn process_receive(
            ref self: ContractState,
            packet: ibc::types::Packet,
            relayer: ContractAddress,
            relayer_msg: ByteArray,
            intent: bool,
        ) -> ByteArray {
            let zkgm_packet: ZkgmPacket = ethabi_decode(packet.data.clone()).unwrap();

            match self
                .execute(
                    packet,
                    relayer,
                    relayer_msg,
                    zkgm_packet.salt,
                    zkgm_packet.path,
                    zkgm_packet.instruction,
                    intent,
                ) {
                Ok(ack) => {
                    // For the async ack
                    if ack == "" {
                        return "";
                    }

                    let mut only_maker: ByteArray = Default::default();
                    only_maker.append_u256(ACK_ERR_ONLYMAKER);

                    // we panic if the error is onlymaker so that the marketmakers can fill later
                    assert!(ack != only_maker);

                    ethabi_encode(@AckTrait::new_success(ack))
                },
                Err(_) => { ethabi_encode(@AckTrait::new_failure()) },
            }
        }

        fn send(
            ref self: ContractState,
            channel_id: ChannelId,
            timeout_height: u64,
            timeout_timestamp: u64,
            salt: ByteArray,
            instruction: Instruction,
        ) {
            assert!(salt.len() == 32);

            self.verify(channel_id, 0, instruction);

            assert!(
                IIbcModuleSendDispatcher { contract_address: self.ibc.read() }
                    .send_packet(channel_id, timeout_height, timeout_timestamp, Default::default())
                    .is_ok(),
            );
        }

        fn verify(
            ref self: ContractState, channel_id: ChannelId, path: u256, instruction: Instruction,
        ) {
            match (instruction.opcode, instruction.version) {
                (
                    Opcode::TokenOrder, Version::V2,
                ) => {
                    let order = ethabi_decode(instruction.operand).unwrap();
                    self.verify_token_order(channel_id, path, order);
                },
                _ => panic!("unsupported (instruction, version)"),
            }
        }

        fn verify_token_order(
            ref self: ContractState, channel_id: ChannelId, path: u256, order: TokenOrderV2,
        ) {
            let (_, base_token) = order.base_token.read_felt252(0);
            let base_token: ContractAddress = base_token.try_into().unwrap();

            if order.kind == TOKEN_ORDER_KIND_UNESCROW {
                let (intermediate_channel_path, destination_channel_id) = pop_channel_from_path(
                    self.token_origin.read(base_token),
                );

                let is_inverse_intermediate_path = path == reverse_channel_path(
                    intermediate_channel_path,
                )
                    .unwrap();

                let is_sending_back_to_same_channel = destination_channel_id == channel_id.raw();

                let metadata_image = self.metadata_image_of.read(base_token);
                let (wrapped_token, _) = self
                    .predict_wrapped_token_from_metadata_image(
                        path, channel_id, order.quote_token, metadata_image,
                    );

                let is_unwrapping = base_token == wrapped_token;

                assert!(
                    is_unwrapping
                        && is_inverse_intermediate_path
                        && is_sending_back_to_same_channel,
                );

                IZkgmERC20Dispatcher { contract_address: base_token }
                    .burn(get_caller_address(), order.base_amount);
            } else {
                self
                    .increase_outstanding(
                        channel_id.raw(), path, base_token, order.quote_token, order.base_amount,
                    );
                // TODO(aeryz): gas station
                assert!(
                    IERC20Dispatcher { contract_address: base_token }
                        .transfer_from(
                            get_caller_address(), get_contract_address(), order.base_amount,
                        ),
                );
            }
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

        // NOTE: TokenOrderV2 impl
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

            // For intent packets, the protocol is not allowed to provide any fund
            // as the packet has not been checked for membership proof. Instead, we
            // know the market maker will be repaid on the source chain, if and only
            // if the currently executing packet hash had been registered as sent on
            // the source. In other words, the market maker is unable to lie.
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
                let (wrapped_token, _) = if order.kind == TOKEN_ORDER_KIND_ESCROW {
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

                    let (wrapped_token, wrapped_token_salt, calldata) = predict_wrapped_token(
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
                    .write(wrapped_token, update_channel_path(path, channel_id.raw())?);

                self.metadata_image_of.write(wrapped_token, ethabi_encode(@metadata).keccak_be());

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

        fn predict_wrapped_token_from_metadata_image(
            self: @ContractState,
            path: u256,
            channel: ChannelId,
            token: ByteArray,
            metadata_image: u256,
        ) -> (ContractAddress, felt252) {
            let metadata = self.token_metadata_image_to_preimage.read(metadata_image);

            let (ca, salt, _) = predict_wrapped_token_from_metadata_and_image(
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

                // We want the top level handler in onRecvPacket to know we need to
                // revert for another MM to get a chance to fill. If we revert now
                // the entire packet would be considered to be "failed" and refunded
                // at origin, which we want to avoid.
                // Hence, in case of transfer failure, we yield the ack to notify the onRecvPacket.

                // TODO(aeryz): gas station?
                if quote_amount > 0
                    && !(IERC20Dispatcher { contract_address: quote_token }
                        .transfer_from(get_caller_address(), receiver, quote_amount)) {
                    let mut out: ByteArray = Default::default();
                    out.append_u256(ACK_ERR_ONLYMAKER);
                    return Ok(out);
                }

                Ok(
                    ethabi_encode(
                        // The relayer has to provide it's maker address using the
                        // relayerMsg. This address is specific to the counterparty
                        // chain and is where the protocol will pay back the base amount
                        // on acknowledgement.
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
                            // The solver has to provide it's maker address that the
                            // counterparty chain will repay on acknowledgement with the
                            // base token.
                            @TokenOrderAck { fill_type: FILL_TYPE_MARKETMAKER, market_maker: ret },
                        ),
                    )
                },
                Err(_) => {
                    let mut out: ByteArray = Default::default();
                    out.append_u256(ACK_ERR_ONLYMAKER);
                    Ok(out)
                },
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
            let fee = base_amount - quote_amount;

            // If the base token path is being unwrapped, it's escrowed balance will be non zero.
            self
                .decrease_outstanding(
                    channel_id.raw(),
                    reverse_channel_path(path)?,
                    quote_token,
                    base_token,
                    base_amount,
                );

            // TODO(aeryz): gas station?
            if quote_amount > 0
                && !(IERC20Dispatcher { contract_address: quote_token }
                    .transfer(receiver, quote_amount)) {
                return Err(());
            }

            if fee > 0
                && !(IERC20Dispatcher { contract_address: quote_token }.transfer(relayer, fee)) {
                return Err(());
            }

            Ok(ethabi_encode(@TokenOrderAck { fill_type: FILL_TYPE_PROTOCOL, market_maker: "" }))
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

            Ok(ethabi_encode(@TokenOrderAck { fill_type: FILL_TYPE_PROTOCOL, market_maker: "" }))
        }

        fn decrease_outstanding(
            ref self: ContractState,
            source_channel_id: u32,
            path: u256,
            base_token: ContractAddress,
            quote_token: ByteArray,
            amount: u256,
        ) {
            let key = ChannelBalanceKey { source_channel_id, path, base_token, quote_token };
            let val = self.channel_balance.read(key.clone());
            self.channel_balance.write(key, val - amount);
        }

        fn increase_outstanding(
            ref self: ContractState,
            source_channel_id: u32,
            path: u256,
            base_token: ContractAddress,
            quote_token: ByteArray,
            amount: u256,
        ) {
            let key = ChannelBalanceKey { source_channel_id, path, base_token, quote_token };
            let val = self.channel_balance.read(key.clone());
            self.channel_balance.write(key, val + amount);
        }
    }
}
