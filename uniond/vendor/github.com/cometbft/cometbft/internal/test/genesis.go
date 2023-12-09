package test

import (
	"time"

	"github.com/cometbft/cometbft/types"
)

func GenesisDoc(
	time time.Time,
	validators []*types.Validator,
	consensusParams *types.ConsensusParams,
	chainID string,
) *types.GenesisDoc {

	genesisValidators := make([]types.GenesisValidator, len(validators))

	for i := range validators {
		genesisValidators[i] = types.GenesisValidator{
			Power:  validators[i].VotingPower,
			PubKey: validators[i].PubKey,
		}
	}

	return &types.GenesisDoc{
		GenesisTime:     time,
		InitialHeight:   1,
		ChainID:         chainID,
		Validators:      genesisValidators,
		ConsensusParams: consensusParams,
	}
}
