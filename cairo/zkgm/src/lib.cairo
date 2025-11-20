pub mod event;
pub mod types;

/// Simple contract for managing balance.
#[starknet::contract]
mod Ucs03Zkgm {
    use ibc::types::Packet;
    use starknet::ContractAddress;
    use crate::event::CreateWrappedToken;

    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        CreateWrappedToken: CreateWrappedToken,
    }

    #[abi(embed_v0)]
    impl Ucs03ZkgmImpl of ibc::app::IIbcModuleRecv<ContractState> {
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
}
