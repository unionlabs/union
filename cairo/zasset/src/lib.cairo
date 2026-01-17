use ibc::types::ClientId;
/// Interface representing `HelloContract`.
/// This interface allows modification and retrieval of the contract balance.
use starknet::ContractAddress;

#[starknet::interface]
pub trait IZAsset<TContractState> {
    /// Increase contract balance.
    fn redeem(
        ref self: TContractState,
        client_id: ClientId,
        height: u64,
        nullifier: u256,
        redeem_amount: u256,
        beneficiary: ContractAddress,
        attested_message: felt252,
        signature: (felt252, felt252),
        unwrap: bool,
    );
}

/// Simple contract for managing balance.
#[starknet::contract]
mod ZAsset {
    use core::ecdsa;
    use core::keccak::compute_keccak_byte_array;
    use ibc::lightclient::{ILightClientSafeDispatcher, ILightClientSafeDispatcherTrait};
    use ibc::types::{ChainId, ClientId};
    use loopback_light_client::types::ConsensusState;
    use openzeppelin::token::erc20::{DefaultConfig, ERC20Component, ERC20HooksEmptyImpl};
    use starknet::storage::{
        Map, StorageMapReadAccess, StorageMapWriteAccess, StoragePointerReadAccess,
        StoragePointerWriteAccess,
    };
    use starknet::syscalls::keccak_syscall;
    use starknet::{ContractAddress, SyscallResultTrait};

    component!(path: ERC20Component, storage: erc20, event: ERC20Event);
    #[abi(embed_v0)]
    impl ERC20MixinImpl = ERC20Component::ERC20MixinImpl<ContractState>;
    impl ERC20InternalImpl = ERC20Component::InternalImpl<ContractState>;

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        #[flat]
        ERC20Event: ERC20Component::Event,
    }

    #[storage]
    struct Storage {
        #[substorage(v0)]
        erc20: ERC20Component::Storage,
        client_address: ContractAddress,
        nullifier_balance: Map<u256, u256>,
        attestor_pubkey: felt252,
        chain_id: ChainId,
    }

    #[abi(embed_v0)]
    impl ZAssetImpl of super::IZAsset<ContractState> {
        fn redeem(
            ref self: ContractState,
            client_id: ClientId,
            height: u64,
            nullifier: u256,
            redeem_amount: u256,
            beneficiary: ContractAddress,
            attested_message: felt252,
            signature: (felt252, felt252),
            unwrap: bool,
        ) {
            // TODO(aeryz): correct scalar R
            const SCALAR_R: u256 = 1_000_000;

            #[feature("safe_dispatcher")]
            let mut res = ILightClientSafeDispatcher {
                contract_address: self.client_address.read(),
            }
                .get_consensus_state(client_id, height)
                .unwrap_syscall()
                .span();

            let consensus_state: ConsensusState = Serde::deserialize(ref res).unwrap();

            let already_redeemed = self.nullifier_balance.read(nullifier);
            assert(already_redeemed + redeem_amount >= SCALAR_R, 'NULLIFIER_EXCEED');
            self.nullifier_balance.write(nullifier, already_redeemed + redeem_amount);

            let (signature_r, signature_s) = signature;
            assert(
                self
                    .attestor_pubkey
                    .read() == ecdsa::recover_public_key(
                        attested_message, signature_r, signature_s, false,
                    )
                    .expect('INVALID_ATTESTOR_SIGNATURE'),
                'INVALID_ATTESTOR_SIGNATURE',
            );

            let inputs_hash = calculate_inputs_hash(
                consensus_state.contracts_trie_root,
                key,
                value,
                self.chain_id.read(),
                nullifier,
                redeem_amount,
                already_redeemed,
                attested_message,
                beneficiary,
            )
                & 0x00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;

            // TODO(aeryz): verify proof

            self.erc20.mint(beneficiary, redeem_amount);

            if unwrap {
                withdraw(@self);
            }
        }
    }

    fn calculate_inputs_hash(
        state_root: felt252,
        key: felt252,
        value: felt252,
        dest_chain_id: ChainId,
        nullifier: u256,
        redeem_amount: u256,
        already_redeemed: u256,
        attested_message: felt252,
        selected_beneficiary: ContractAddress,
    ) -> u256 {
        let mut bz: ByteArray = Default::default();
        compute_keccak_byte_array(@bz)
    }

    fn withdraw(ref self: ContractState, account: ContractAddress, amount: u256) {
        self.erc20.burn(account, amount);
        // TODO(aeryz): emit Withdrawn(account, amount);
    }
}
