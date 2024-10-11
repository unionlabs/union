package tendermint

import (
	storetypes "cosmossdk.io/store/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// ExportMetadata exports all the consensus metadata in the client store so they can be included in clients genesis
// and imported by a ClientKeeper
func (ClientState) ExportMetadata(store storetypes.KVStore) []exported.GenesisMetadata {
	gm := make([]exported.GenesisMetadata, 0)
	IterateConsensusMetadata(store, func(key, val []byte) bool {
		gm = append(gm, clienttypes.NewGenesisMetadata(key, val))
		return false
	})
	if len(gm) == 0 {
		return nil
	}
	return gm
}
