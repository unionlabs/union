package e2e_test

import (
	"bytes"
	"testing"

	"github.com/stretchr/testify/require"

	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
	"github.com/cometbft/cometbft/types"
)

// Tests that validator sets are available and correct according to
// scheduled validator updates.
func TestValidator_Sets(t *testing.T) {
	t.Helper()
	testNode(t, func(t *testing.T, node e2e.Node) {
		t.Helper()
		if node.Mode == e2e.ModeSeed || node.EnableCompanionPruning {
			return
		}

		client, err := node.Client()
		require.NoError(t, err)
		status, err := client.Status(ctx)
		require.NoError(t, err)

		first := status.SyncInfo.EarliestBlockHeight
		last := status.SyncInfo.LatestBlockHeight

		// skip first block if node is pruning blocks, to avoid race conditions
		if node.RetainBlocks > 0 {
			// This was done in case pruning is activated.
			// As it happens in the background this lowers the chances
			// that the block at height=first will be pruned by the time we test
			// this. If this test starts to fail often, it is worth revisiting this logic.
			// To reproduce this failure locally, it is advised to set the storage.pruning.interval
			// to 1s instead of 10s.
			first += int64(node.RetainBlocks)
		}

		valSchedule := newValidatorSchedule(*node.Testnet)
		valSchedule.Increment(first - node.Testnet.InitialHeight)

		for h := first; h <= last; h++ {
			validators := []*types.Validator{}
			perPage := 100
			for page := 1; ; page++ {
				resp, err := client.Validators(ctx, &(h), &(page), &perPage)
				require.NoError(t, err)
				validators = append(validators, resp.Validators...)
				if len(validators) == resp.Total {
					break
				}
			}
			require.Equal(t, valSchedule.Set.Validators, validators,
				"incorrect validator set at height %v", h)
			valSchedule.Increment(1)
		}
	})
}

// Tests that a validator proposes blocks when it's supposed to. It tolerates some
// missed blocks, e.g. due to testnet perturbations.
func TestValidator_Propose(t *testing.T) {
	t.Helper()
	blocks := fetchBlockChain(t)
	testNode(t, func(t *testing.T, node e2e.Node) {
		t.Helper()
		if node.Mode != e2e.ModeValidator {
			return
		}
		address := node.PrivvalKey.PubKey().Address()
		valSchedule := newValidatorSchedule(*node.Testnet)

		expectCount := 0
		proposeCount := 0
		for _, block := range blocks {
			if bytes.Equal(valSchedule.Set.Proposer.Address, address) {
				expectCount++
				if bytes.Equal(block.ProposerAddress, address) {
					proposeCount++
				}
			}
			valSchedule.Increment(1)
		}

		if expectCount == 0 {
			return
		}

		if node.ClockSkew != 0 && node.Testnet.PbtsEnableHeight != 0 {
			t.Logf("node with skewed clock (by %v), proposed %v, expected %v",
				node.ClockSkew, proposeCount, expectCount)
			return
		}
		require.Greater(t, proposeCount, 0,
			"node did not propose any blocks (expected %v)", expectCount)
		require.False(t, expectCount > 5 && proposeCount < 3, "node only proposed  %v blocks, expected %v", proposeCount, expectCount)
	})
}

// Tests that a validator signs blocks when it's supposed to. It tolerates some
// missed blocks, e.g. due to testnet perturbations.
func TestValidator_Sign(t *testing.T) {
	t.Helper()
	blocks := fetchBlockChain(t)
	testNode(t, func(t *testing.T, node e2e.Node) {
		t.Helper()
		if node.Mode != e2e.ModeValidator {
			return
		}
		address := node.PrivvalKey.PubKey().Address()
		valSchedule := newValidatorSchedule(*node.Testnet)

		expectCount := 0
		signCount := 0
		for _, block := range blocks[1:] { // Skip first block, since it has no signatures
			signed := false
			for _, sig := range block.LastCommit.Signatures {
				if bytes.Equal(sig.ValidatorAddress, address) {
					signed = true
					break
				}
			}
			if valSchedule.Set.HasAddress(address) {
				expectCount++
				if signed {
					signCount++
				}
			} else {
				require.False(t, signed, "unexpected signature for block %v", block.LastCommit.Height)
			}
			valSchedule.Increment(1)
		}

		require.False(t, signCount == 0 && expectCount > 0,
			"validator did not sign any blocks (expected %v)", expectCount)
		if expectCount > 7 {
			require.GreaterOrEqual(t, signCount, 3, "validator didn't sign even 3 blocks (expected %v)", expectCount)
		}
	})
}

// validatorSchedule is a validator set iterator, which takes into account
// validator set updates.
type validatorSchedule struct {
	Set     *types.ValidatorSet
	height  int64
	updates map[int64]map[*e2e.Node]int64
}

func newValidatorSchedule(testnet e2e.Testnet) *validatorSchedule {
	valMap := testnet.Validators                  // genesis validators
	if v, ok := testnet.ValidatorUpdates[0]; ok { // InitChain validators
		valMap = v
	}
	return &validatorSchedule{
		height:  testnet.InitialHeight,
		Set:     types.NewValidatorSet(makeVals(valMap)),
		updates: testnet.ValidatorUpdates,
	}
}

func (s *validatorSchedule) Increment(heights int64) {
	for i := int64(0); i < heights; i++ {
		s.height++
		if s.height > 2 {
			// validator set updates are offset by 2, since they only take effect
			// two blocks after they're returned.
			if update, ok := s.updates[s.height-2]; ok {
				if err := s.Set.UpdateWithChangeSet(makeVals(update)); err != nil {
					panic(err)
				}
			}
		}
		s.Set.IncrementProposerPriority(1)
	}
}

func makeVals(valMap map[*e2e.Node]int64) []*types.Validator {
	vals := make([]*types.Validator, 0, len(valMap))
	for node, power := range valMap {
		vals = append(vals, types.NewValidator(node.PrivvalKey.PubKey(), power))
	}
	return vals
}
