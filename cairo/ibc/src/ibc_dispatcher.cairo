use ibc::types::Packet;
use starknet::{ContractAddress, SyscallResultTrait, syscalls};
use crate::types::ChannelId;

pub trait IIBCDispatcherTrait<T> {
    fn send_packet(
        self: T,
        channel_id: ChannelId,
        timeout_height: u64,
        timeout_timestamp: u64,
        data: ByteArray,
    ) -> Result<Packet, ()>;
}

#[derive(Copy, Drop, starknet::Store, Serde)]
pub struct IIBCDispatcher {
    pub contract_address: ContractAddress,
}

impl IIBCDispatcherImpl of IIBCDispatcherTrait<IIBCDispatcher> {
    fn send_packet(
        self: IIBCDispatcher,
        channel_id: ChannelId,
        timeout_height: u64,
        timeout_timestamp: u64,
        data: ByteArray,
    ) -> Result<Packet, ()> {
        let mut calldata = Default::default();
        // TODO(aeryz): check if tuple serialization == serializing each values one by one
        (channel_id, timeout_height, timeout_timestamp, data).serialize(ref calldata);

        // TODO(aeryz): fallible
        let mut res = syscalls::call_contract_syscall(
            self.contract_address, selector!("send_packet"), calldata.span(),
        )
            .unwrap_syscall();

        Serde::deserialize(ref res).ok_or(())
    }
}

