pub mod event;

#[starknet::interface]
pub trait IUcs03Zkgm<TContractState> {
    /// Increase contract balance.
    fn increase_balance(ref self: TContractState, amount: felt252);
    /// Retrieve contract balance.
    fn get_balance(self: @TContractState) -> felt252;
}

/// Simple contract for managing balance.
#[starknet::contract]
mod Ucs03Zkgm {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
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
    impl Ucs03ZkgmImpl of super::IUcs03Zkgm<ContractState> {
        fn increase_balance(ref self: ContractState, amount: felt252) {
            assert(amount != 0, 'Amount cannot be 0');
            self.balance.write(self.balance.read() + amount);
        }

        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }
    }
}
