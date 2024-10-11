package state

import (
	mempl "github.com/cometbft/cometbft/mempool"
	"github.com/cometbft/cometbft/types"
)

// TxPreCheck returns a function to filter transactions before processing.
// The function limits the size of a transaction to the block's maximum data size.
func TxPreCheck(state State) mempl.PreCheckFunc {
	maxBytes := state.ConsensusParams.Block.MaxBytes
	if maxBytes == -1 {
		maxBytes = int64(types.MaxBlockSizeBytes)
	}
	maxDataBytes := types.MaxDataBytesNoEvidence(
		maxBytes,
		state.Validators.Size(),
	)
	return mempl.PreCheckMaxBytes(maxDataBytes)
}

// TxPostCheck returns a function to filter transactions after processing.
// The function limits the gas wanted by a transaction to the block's maximum total gas.
func TxPostCheck(state State) mempl.PostCheckFunc {
	return mempl.PostCheckMaxGas(state.ConsensusParams.Block.MaxGas)
}
