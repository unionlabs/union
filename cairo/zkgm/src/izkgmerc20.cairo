use starknet::{ContractAddress, SyscallResultTrait, syscalls};

pub trait IZkgmERC20DispatcherTrait<T> {
    fn mint(self: T, recipient: ContractAddress, amount: u256);
    fn burn(self: T, account: ContractAddress, amount: u256);
}

#[derive(Copy, Drop, starknet::Store, Serde)]
pub struct IZkgmERC20Dispatcher {
    pub contract_address: ContractAddress,
}

impl IZkgmERC20Impl of IZkgmERC20DispatcherTrait<IZkgmERC20Dispatcher> {
    // TODO(aeryz): we might wanna make these functions fallible
    fn mint(self: IZkgmERC20Dispatcher, recipient: ContractAddress, amount: u256) {
        let mut calldata = Default::default();
        recipient.serialize(ref calldata);
        amount.serialize(ref calldata);

        let _ = syscalls::call_contract_syscall(
            self.contract_address, selector!("mint"), calldata.span(),
        )
            .unwrap_syscall();
    }

    fn burn(self: IZkgmERC20Dispatcher, account: ContractAddress, amount: u256) {
        let mut calldata = Default::default();
        account.serialize(ref calldata);
        amount.serialize(ref calldata);

        let _ = syscalls::call_contract_syscall(
            self.contract_address, selector!("burn"), calldata.span(),
        )
            .unwrap_syscall();
    }
}

