package test

import (
	"github.com/cometbft/cometbft/types"
)

// ConsensusParams returns a default set of ConsensusParams that are suitable
// for use in testing.
func ConsensusParams() *types.ConsensusParams {
	c := types.DefaultConsensusParams()
	// enable vote extensions
	c.Feature.VoteExtensionsEnableHeight = 1
	// enabled PBTS
	c.Feature.PbtsEnableHeight = 1
	return c
}
