use unionlabs_primitives::{Bytes, H160};

/// Client state for a client tracking an ethermint chain.
///
/// The consensus verification of an ethermint chain is identical to that of a tendermint chain. The
/// only addition is the additional configuration parameters used to verify the EVM state, which is
/// verified with `[store_key, [...key_prefix_storage, ...ibc_contract_address]]`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientState {
    /// The client state of the underlying tendermint consensus.
    pub tendermint_client_state: tendermint_light_client_types::ClientState,
    /// The module that the ethermint state is stored in.
    ///
    /// For standard ethermint, this is `b"evm"`: <https://github.com/0glabs/ethermint/blob/fd8c2d25cf80e7d2d2a142e7b374f979f8f51981/x/evm/types/key.go#L24>
    pub store_key: Bytes,
    /// The store prefix for smart contract storage values.
    ///
    /// For standard ethermint, this is `[0x2]`: <https://github.com/0glabs/ethermint/blob/fd8c2d25cf80e7d2d2a142e7b374f979f8f51981/x/evm/types/key.go#L57>
    pub key_prefix_storage: Bytes,
    /// The contract address of the `IbcHandler` contract running on the ethermint EVM.
    pub ibc_contract_address: H160,
}
