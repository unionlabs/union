package doublylinkedtree

import (
	"context"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

// Helper function to simulate the block being on time or delayed for proposer
// boost. It alters the genesisTime tracked by the store.
func driftGenesisTime(f *ForkChoice, slot primitives.Slot, delay uint64) {
	f.SetGenesisTime(uint64(time.Now().Unix()) - uint64(slot)*params.BeaconConfig().SecondsPerSlot - delay)
}

// Simple, ex-ante attack mitigation using proposer boost.
// In a nutshell, an adversarial block proposer in slot n+1 keeps its proposal hidden.
// The honest block proposer in slot n+2 will then propose an honest block. The
// adversary can now use its committee members’ votes from both slots n+1 and n+2.
// and release their withheld block of slot n+2 in an attempt to win fork choice.
// If the honest proposal is boosted at slot n+2, it will win against this attacker.
func TestForkChoice_BoostProposerRoot_PreventsExAnteAttack(t *testing.T) {
	ctx := context.Background()
	jEpoch, fEpoch := primitives.Epoch(0), primitives.Epoch(0)
	zeroHash := params.BeaconConfig().ZeroHash
	balances := make([]uint64, 64) // 64 active validators.
	for i := 0; i < len(balances); i++ {
		balances[i] = 10
	}
	t.Run("back-propagates boost score to ancestors after proposer boosting", func(t *testing.T) {
		f := setup(jEpoch, fEpoch)
		f.justifiedBalances = balances
		f.store.committeeWeight = uint64(len(balances)*10) / uint64(params.BeaconConfig().SlotsPerEpoch)
		f.numActiveValidators = uint64(len(balances))

		// The head should always start at the finalized block.
		headRoot, err := f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, zeroHash, headRoot, "Incorrect head with genesis")

		// Insert block at slot 1 into the tree and verify head is at that block:
		//         0
		//         |
		//         1 <- HEAD
		slot := primitives.Slot(1)
		driftGenesisTime(f, slot, 0)
		newRoot := indexToHash(1)
		f.store.proposerBoostRoot = [32]byte{}
		state, blkRoot, err := prepareForkchoiceState(
			ctx,
			slot,
			newRoot,
			headRoot,
			zeroHash,
			jEpoch,
			fEpoch,
		)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))
		f.ProcessAttestation(ctx, []uint64{0}, newRoot, fEpoch)
		headRoot, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, newRoot, headRoot, "Incorrect head for justified epoch at slot 1")

		// Insert block at slot 2 into the tree and verify head is at that block:
		//         0
		//         |
		//         1
		//         |
		//         2 <- HEAD
		slot = primitives.Slot(2)
		driftGenesisTime(f, slot, 0)
		newRoot = indexToHash(2)
		f.store.proposerBoostRoot = [32]byte{}
		state, blkRoot, err = prepareForkchoiceState(
			ctx,
			slot,
			newRoot,
			headRoot,
			zeroHash,
			jEpoch,
			fEpoch,
		)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))
		f.ProcessAttestation(ctx, []uint64{1}, newRoot, fEpoch)
		headRoot, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, newRoot, headRoot, "Incorrect head for justified epoch at slot 2")

		// Insert block at slot 3 into the tree and verify head is at that block:
		//         0
		//         |
		//         1
		//         |
		//         2
		//         |
		//         3 <- HEAD
		slot = primitives.Slot(3)
		driftGenesisTime(f, slot, 0)
		newRoot = indexToHash(3)
		f.store.proposerBoostRoot = [32]byte{}
		state, blkRoot, err = prepareForkchoiceState(
			ctx,
			slot,
			newRoot,
			headRoot,
			zeroHash,
			jEpoch,
			fEpoch,
		)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))
		f.ProcessAttestation(ctx, []uint64{2}, newRoot, fEpoch)
		headRoot, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, newRoot, headRoot, "Incorrect head for justified epoch at slot 3")

		// Insert a second block at slot 4 into the tree and boost its score.
		//         0
		//         |
		//         1
		//         |
		//         2
		//        / \
		//       3   |
		//           4 <- HEAD
		slot = primitives.Slot(4)
		driftGenesisTime(f, slot, 0)
		newRoot = indexToHash(4)
		f.store.proposerBoostRoot = [32]byte{}
		state, blkRoot, err = prepareForkchoiceState(
			ctx,
			slot,
			newRoot,
			indexToHash(2),
			zeroHash,
			jEpoch,
			fEpoch,
		)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))
		f.ProcessAttestation(ctx, []uint64{3}, newRoot, fEpoch)
		headRoot, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, newRoot, headRoot, "Incorrect head for justified epoch at slot 3")

		// Check the ancestor scores from the store.
		require.Equal(t, 5, f.NodeCount())

		// Expect nodes to have a boosted, back-propagated score.
		// Ancestors have the added weights of their children. Genesis is a special exception at 0 weight,
		require.Equal(t, f.store.treeRootNode.weight, uint64(0))

		// Proposer boost score with these test parameters is 8
		// Each of the nodes received one attestation accounting for 10.
		// Node D is the only one with proposer boost still applied:
		//
		// (1: 48) -> (2: 38) -> (3: 10)
		//		    \--------------->(4: 18)
		//
		node1 := f.store.nodeByRoot[indexToHash(1)]
		require.Equal(t, node1.weight, uint64(48))
		node2 := f.store.nodeByRoot[indexToHash(2)]
		require.Equal(t, node2.weight, uint64(38))
		node3 := f.store.nodeByRoot[indexToHash(3)]
		require.Equal(t, node3.weight, uint64(10))
		node4 := f.store.nodeByRoot[indexToHash(4)]
		require.Equal(t, node4.weight, uint64(18))

		// Regression: process attestations for C, check that it
		// becomes head, we need two attestations to have C.weight = 30 > 24 = D.weight
		f.ProcessAttestation(ctx, []uint64{4, 5}, indexToHash(3), fEpoch)
		headRoot, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, indexToHash(3), headRoot, "Incorrect head for justified epoch at slot 4")
	})
	t.Run("vanilla ex ante attack", func(t *testing.T) {
		f := setup(jEpoch, fEpoch)
		f.justifiedBalances = balances

		// The head should always start at the finalized block.
		r, err := f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, zeroHash, r, "Incorrect head with genesis")

		// Proposer from slot 1 does not reveal their block, B, at slot 1.
		// Proposer at slot 2 does reveal their block, C, and it becomes the head.
		// C builds on A, as proposer at slot 1 did not reveal B.
		//         A
		//        / \
		//      (B?) \
		//            \
		//             C <- Slot 2 HEAD
		honestBlockSlot := primitives.Slot(2)
		driftGenesisTime(f, honestBlockSlot, 0)
		honestBlock := indexToHash(2)
		state, blkRoot, err := prepareForkchoiceState(
			ctx,
			honestBlockSlot,
			honestBlock,
			zeroHash,
			zeroHash,
			jEpoch,
			fEpoch,
		)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))
		r, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, honestBlock, r, "Incorrect head for justified epoch at slot 2")

		maliciouslyWithheldBlockSlot := primitives.Slot(1)
		maliciouslyWithheldBlock := indexToHash(1)
		state, blkRoot, err = prepareForkchoiceState(
			ctx,
			maliciouslyWithheldBlockSlot,
			maliciouslyWithheldBlock,
			zeroHash,
			zeroHash,
			jEpoch,
			fEpoch,
		)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))

		// Ensure the head is C, the honest block.
		r, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, honestBlock, r, "Incorrect head for justified epoch at slot 2")

		// The maliciously withheld block has one vote.
		votes := []uint64{1}
		f.ProcessAttestation(ctx, votes, maliciouslyWithheldBlock, fEpoch)
		// The honest block has one vote.
		votes = []uint64{2}
		f.ProcessAttestation(ctx, votes, honestBlock, fEpoch)

		// Ensure the head is STILL C, the honest block, as the honest block had proposer boost.
		r, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, honestBlock, r, "Incorrect head for justified epoch at slot 2")
	})
	t.Run("adversarial attestations > proposer boosting", func(t *testing.T) {
		f := setup(jEpoch, fEpoch)
		f.justifiedBalances = balances

		// The head should always start at the finalized block.
		r, err := f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, zeroHash, r, "Incorrect head with genesis")

		// Proposer from slot 1 does not reveal their block, B, at slot 1.
		// Proposer at slot 2 does reveal their block, C, and it becomes the head.
		// C builds on A, as proposer at slot 1 did not reveal B.
		//         A
		//        / \
		//	(B?) \
		//            \
		//             C <- Slot 2 HEAD
		honestBlockSlot := primitives.Slot(2)
		driftGenesisTime(f, honestBlockSlot, 0)
		honestBlock := indexToHash(2)
		state, blkRoot, err := prepareForkchoiceState(
			ctx,
			honestBlockSlot,
			honestBlock,
			zeroHash,
			zeroHash,
			jEpoch,
			fEpoch,
		)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))

		// Ensure C is the head.
		r, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, honestBlock, r, "Incorrect head for justified epoch at slot 2")

		maliciouslyWithheldBlockSlot := primitives.Slot(1)
		maliciouslyWithheldBlock := indexToHash(1)
		state, blkRoot, err = prepareForkchoiceState(
			ctx,
			maliciouslyWithheldBlockSlot,
			maliciouslyWithheldBlock,
			zeroHash,
			zeroHash,
			jEpoch,
			fEpoch,
		)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))

		// Ensure C is still the head after the malicious proposer reveals their block.
		r, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, honestBlock, r, "Incorrect head for justified epoch at slot 2")

		// An attestation is received for B that has more voting power than C with the proposer boost,
		// allowing B to then become the head if their attestation has enough adversarial votes.
		votes := []uint64{1, 2}
		f.ProcessAttestation(ctx, votes, maliciouslyWithheldBlock, fEpoch)

		// Expect the head to have switched to B.
		r, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, maliciouslyWithheldBlock, r, "Expected B to become the head")
	})
	t.Run("boosting necessary to sandwich attack", func(t *testing.T) {
		// Boosting necessary to sandwich attack.
		// Objects:
		//	Block A - slot N
		//	Block B (parent A) - slot N+1
		//	Block C (parent A) - slot N+2
		//	Block D (parent B) - slot N+3
		//	Attestation_1 (Block C); size 1 - slot N+2 (honest)
		// Steps:
		//	Block A received at N — A is head
		//	Block C received at N+2 — C is head
		//	Block B received at N+2 — C is head
		//	Attestation_1 received at N+3 — C is head
		//	Block D received at N+3 — D is head
		f := setup(jEpoch, fEpoch)
		f.justifiedBalances = balances
		f.store.committeeWeight = uint64(len(balances)*10) / uint64(params.BeaconConfig().SlotsPerEpoch)
		f.numActiveValidators = uint64(len(balances))
		a := zeroHash

		// The head should always start at the finalized block.
		r, err := f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, zeroHash, r, "Incorrect head with genesis")

		cSlot := primitives.Slot(2)
		driftGenesisTime(f, cSlot, 0)
		c := indexToHash(2)
		f.store.proposerBoostRoot = [32]byte{}
		state, blkRoot, err := prepareForkchoiceState(
			ctx,
			cSlot,
			c,
			a, // parent
			zeroHash,
			jEpoch,
			fEpoch,
		)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))

		// Ensure C is the head.
		r, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, c, r, "Incorrect head for justified epoch at slot 2")

		bSlot := primitives.Slot(1)
		b := indexToHash(1)
		f.store.proposerBoostRoot = [32]byte{}
		state, blkRoot, err = prepareForkchoiceState(
			ctx,
			bSlot,
			b,
			a, // parent
			zeroHash,
			jEpoch,
			fEpoch,
		)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))

		// Ensure C is still the head.
		r, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, c, r, "Incorrect head for justified epoch at slot 2")

		// An attestation for C is received at slot N+3.
		votes := []uint64{1}
		f.ProcessAttestation(ctx, votes, c, fEpoch)

		// A block D, building on B, is received at slot N+3. It should not be able to win without boosting.
		dSlot := primitives.Slot(3)
		d := indexToHash(3)
		f.store.proposerBoostRoot = [32]byte{}
		state, blkRoot, err = prepareForkchoiceState(
			ctx,
			dSlot,
			d,
			b, // parent
			zeroHash,
			jEpoch,
			fEpoch,
		)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))

		// D cannot win without a boost.
		r, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, c, r, "Expected C to remain the head")

		// If the same block arrives with boosting then it becomes head:
		driftGenesisTime(f, dSlot, 0)
		d2 := indexToHash(30)
		f.store.proposerBoostRoot = [32]byte{}
		state, blkRoot, err = prepareForkchoiceState(
			ctx,
			dSlot,
			d2,
			b, // parent
			zeroHash,
			jEpoch,
			fEpoch,
		)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))

		votes = []uint64{2}
		f.ProcessAttestation(ctx, votes, d2, fEpoch)
		// Ensure D becomes the head thanks to boosting.
		r, err = f.Head(ctx)
		require.NoError(t, err)
		assert.Equal(t, d2, r, "Expected D to become the head")
	})
}

func TestForkChoice_BoostProposerRoot(t *testing.T) {
	ctx := context.Background()
	root := [32]byte{'A'}
	var zeroHash [32]byte

	t.Run("does not boost block from different slot", func(t *testing.T) {
		f := setup(0, 0)
		slot := primitives.Slot(0)
		currentSlot := primitives.Slot(1)
		driftGenesisTime(f, currentSlot, 0)
		state, blkRoot, err := prepareForkchoiceState(ctx, slot, root, zeroHash, zeroHash, 0, 0)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))
		require.Equal(t, [32]byte{}, f.store.proposerBoostRoot)
	})
	t.Run("does not boost untimely block from same slot", func(t *testing.T) {
		f := setup(0, 0)
		slot := primitives.Slot(1)
		currentSlot := primitives.Slot(1)
		driftGenesisTime(f, currentSlot, params.BeaconConfig().SecondsPerSlot-1)
		state, blkRoot, err := prepareForkchoiceState(ctx, slot, root, zeroHash, zeroHash, 0, 0)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))
		require.Equal(t, [32]byte{}, f.store.proposerBoostRoot)
	})
	t.Run("boosts perfectly timely block from same slot", func(t *testing.T) {
		f := setup(0, 0)
		slot := primitives.Slot(1)
		currentSlot := primitives.Slot(1)
		driftGenesisTime(f, currentSlot, 0)
		state, blkRoot, err := prepareForkchoiceState(ctx, slot, root, zeroHash, zeroHash, 0, 0)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))
		require.Equal(t, root, f.store.proposerBoostRoot)
	})
	t.Run("boosts timely block from same slot", func(t *testing.T) {
		f := setup(0, 0)
		slot := primitives.Slot(1)
		currentSlot := primitives.Slot(1)
		driftGenesisTime(f, currentSlot, 1)
		state, blkRoot, err := prepareForkchoiceState(ctx, slot, root, zeroHash, zeroHash, 0, 0)
		require.NoError(t, err)
		require.NoError(t, f.InsertNode(ctx, state, blkRoot))
		require.Equal(t, root, f.store.proposerBoostRoot)
	})
}

// Regression test (11053)
func TestForkChoice_missingProposerBoostRoots(t *testing.T) {
	ctx := context.Background()
	f := setup(1, 1)
	balances := make([]uint64, 64) // 64 active validators.
	for i := 0; i < len(balances); i++ {
		balances[i] = 10
	}
	f.justifiedBalances = balances
	driftGenesisTime(f, 1, 0)
	st, root, err := prepareForkchoiceState(ctx, 1, [32]byte{'r'}, [32]byte{}, [32]byte{}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, root))

	f.store.previousProposerBoostRoot = [32]byte{'p'}
	headRoot, err := f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, root, headRoot)
	require.Equal(t, [32]byte{'r'}, f.store.proposerBoostRoot)

	f.store.proposerBoostRoot = [32]byte{'p'}
	driftGenesisTime(f, 3, 0)
	st, root, err = prepareForkchoiceState(ctx, 2, [32]byte{'a'}, [32]byte{'r'}, [32]byte{}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, root))
	headRoot, err = f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, root, headRoot)
	require.Equal(t, [32]byte{'p'}, f.store.proposerBoostRoot)
}
