package e2e_test

import (
	"testing"

	"github.com/stretchr/testify/require"
)

// assert that all nodes that have blocks at the height of a misbehavior has evidence
// for that misbehavior.
func TestEvidence_Misbehavior(t *testing.T) {
	blocks := fetchBlockChain(t)
	testnet := loadTestnet(t)
	seenEvidence := 0
	for _, block := range blocks {
		if len(block.Evidence.Evidence) != 0 {
			seenEvidence += len(block.Evidence.Evidence)
		}
	}
	require.Equal(t, testnet.Evidence, seenEvidence,
		"difference between the amount of evidence produced and committed")
}
