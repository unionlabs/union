package e2e_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
	"github.com/cometbft/cometbft/test/e2e/pkg/grammar"
)

func TestCheckABCIGrammar(t *testing.T) {
	checker := grammar.NewGrammarChecker(grammar.DefaultConfig())
	testNode(t, func(t *testing.T, node e2e.Node) {
		t.Helper()
		if !node.Testnet.ABCITestsEnabled {
			return
		}
		executions, err := fetchABCIRequests(t, node.Name)
		require.NoError(t, err)
		for i, e := range executions {
			isCleanStart := i == 0
			_, err := checker.Verify(e, isCleanStart)
			require.NoError(t, err)
		}
	})
}

func TestNodeNameExtracting(t *testing.T) {
	testNode(t, func(t *testing.T, node e2e.Node) {
		t.Helper()
		if !node.Testnet.ABCITestsEnabled {
			return
		}
		reqs, err := fetchABCIRequests(t, node.Name)
		require.NoError(t, err)
		require.NotZero(t, len(reqs))
	})
}
