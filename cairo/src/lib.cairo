use core::ecdsa;
use core::fmt::Display;
use core::integer::NumericLiteral;
use core::num::traits::{BitSize, One, Pow, Zero};

#[starknet::interface]
pub trait IHelloStarknet<TContractState> {
    /// Increase contract balance.
    fn increase_balance(ref self: TContractState, amount: felt252);
    /// Retrieve contract balance.
    fn get_balance(self: @TContractState) -> felt252;

    fn to_be_bytess(self: @TContractState, n: u256) -> ByteArray;
}

// fn commit(key: u256, value: u256) {
//     let bz: ByteArray = array![].into_iter().collect();
//     bz.append_byte();
//     let raw_key = compute_keccak_byte_array(key.bytes());
//     // storage_write_syscall(0, storage_address_try_from_felt252(key).unwrap(), value.low.into())
// //     .unwrap();
// // storage_write_syscall(0, storage_address_try_from_felt252(key + 1).unwrap(),
// // value.low.into())
// //     .unwrap();
// }

pub fn to_be_bytes<
    N,
    +Div<N>,
    +Rem<N>,
    +PartialOrd<N>,
    +TryInto<N, u8>,
    +One<N>,
    +Zero<N>,
    +Add<N>,
    +Mul<N>,
    +Pow<N, usize>[Output: N],
    +Drop<N>,
    +Copy<N>,
    +BitSize<N>,
>(
    mut n: N,
) -> ByteArray {
    let modulus = (One::<N>::one() + One::<N>::one()).pow(8);

    ecdsa

    let mut bz: ByteArray = "";

    let mut len = BitSize::<N>::bits() / 8;

    while n > Zero::<N>::zero() {
        len -= 1;
        let b = n % modulus;
        n = n / modulus;
        bz.append_byte(b.try_into().unwrap());
    }

    for _ in 0..len {
        bz.append_byte(0);
    }

    bz.rev().into()
}

/// Simple contract for managing balance.
#[starknet::contract]
mod HelloStarknet {
    use starknet::storage::{Map, StoragePointerReadAccess, StoragePointerWriteAccess};
    use crate::to_be_bytes;


    #[storage]
    struct Storage {
        commitments: Map<u256, u256>,
        balance: felt252,
    }

    #[abi(embed_v0)]
    impl HelloStarknetImpl of super::IHelloStarknet<ContractState> {
        fn increase_balance(ref self: ContractState, amount: felt252) {
            assert(amount != 0, 'Amount cannot be 0');
            self.balance.write(self.balance.read() + amount);
        }

        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }

        fn to_be_bytess(self: @ContractState, n: u256) -> ByteArray {
            to_be_bytes(n)
        }
    }
}
