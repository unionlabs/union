use ibc::types::Packet;
use starknet::{ContractAddress, SyscallResultTrait, syscalls};
use crate::types::TokenOrderV2;

pub trait ISolverDispatcherTrait<T> {
    fn solve(
        self: T,
        packet: Packet,
        order: TokenOrderV2,
        path: u256,
        caller: ContractAddress,
        relayer: ContractAddress,
        relayer_msg: ByteArray,
        intent: bool,
    ) -> Result<ByteArray, ()>;
}

#[derive(Copy, Drop, starknet::Store, Serde)]
pub struct ISolverDispatcher {
    pub contract_address: ContractAddress,
}

impl ISolverImpl of ISolverDispatcherTrait<ISolverDispatcher> {
    fn solve(
        self: ISolverDispatcher,
        packet: Packet,
        order: TokenOrderV2,
        path: u256,
        caller: ContractAddress,
        relayer: ContractAddress,
        relayer_msg: ByteArray,
        intent: bool,
    ) -> Result<ByteArray, ()> {
        let mut calldata = Default::default();
        // TODO(aeryz): check if tuple serialization == serializing each values one by one
        (packet, order, path, caller, relayer, relayer_msg, intent).serialize(ref calldata);

        // TODO(aeryz): fallible
        let mut res = syscalls::call_contract_syscall(
            self.contract_address, selector!("solve"), calldata.span(),
        )
            .unwrap_syscall();

        Serde::deserialize(ref res).ok_or(())
    }
}

