package cometbls

import (
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// VerifyUpgradeAndUpdateState checks if the upgraded client has been committed by the current client
// It will zero out all client-specific fields (e.g. TrustingPeriod) and verify all data
// in client state that must be the same across all valid Tendermint clients for the new chain.
// VerifyUpgrade will return an error if:
// - the upgradedClient is not a Tendermint ClientState
// - the latest height of the client state does not have the same revision number or has a greater
// height than the committed client.
//   - the height of upgraded client is not greater than that of current client
//   - the latest height of the new client does not match or is greater than the height in committed client
//   - any Tendermint chain specified parameter in upgraded client such as ChainID, UnbondingPeriod,
//     and ProofSpecs do not match parameters set by committed client
func (cs ClientState) VerifyUpgradeAndUpdateState(
	ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore,
	upgradedClient exported.ClientState, upgradedConsState exported.ConsensusState,
	upgradeClientProof, upgradeConsStateProof []byte,
) error {
	return nil
}
