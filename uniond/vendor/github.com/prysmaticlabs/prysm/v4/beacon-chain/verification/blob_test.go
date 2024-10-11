package verification

import (
	"bytes"
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func TestBlobIndexInBounds(t *testing.T) {
	ini := &Initializer{}
	_, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 0, 1)
	b := blobs[0]
	// set Index to a value that is out of bounds
	v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
	require.NoError(t, v.BlobIndexInBounds())
	require.Equal(t, true, v.results.executed(RequireBlobIndexInBounds))
	require.NoError(t, v.results.result(RequireBlobIndexInBounds))

	b.Index = fieldparams.MaxBlobsPerBlock
	v = ini.NewBlobVerifier(b, GossipSidecarRequirements)
	require.ErrorIs(t, v.BlobIndexInBounds(), ErrBlobIndexInvalid)
	require.Equal(t, true, v.results.executed(RequireBlobIndexInBounds))
	require.NotNil(t, v.results.result(RequireBlobIndexInBounds))
}

func TestSlotNotTooEarly(t *testing.T) {
	now := time.Now()
	// make genesis 1 slot in the past
	genesis := now.Add(-1 * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second)

	_, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 0, 1)
	b := blobs[0]
	// slot 1 should be 12 seconds after genesis
	b.SignedBlockHeader.Header.Slot = 1

	// This clock will give a current slot of 1 on the nose
	happyClock := startup.NewClock(genesis, [32]byte{}, startup.WithNower(func() time.Time { return now }))
	ini := Initializer{shared: &sharedResources{clock: happyClock}}
	v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
	require.NoError(t, v.NotFromFutureSlot())
	require.Equal(t, true, v.results.executed(RequireNotFromFutureSlot))
	require.NoError(t, v.results.result(RequireNotFromFutureSlot))

	// Since we have an early return for slots that are directly equal, give a time that is less than max disparity
	// but still in the previous slot.
	closeClock := startup.NewClock(genesis, [32]byte{}, startup.WithNower(func() time.Time { return now.Add(-1 * params.BeaconConfig().MaximumGossipClockDisparityDuration() / 2) }))
	ini = Initializer{shared: &sharedResources{clock: closeClock}}
	v = ini.NewBlobVerifier(b, GossipSidecarRequirements)
	require.NoError(t, v.NotFromFutureSlot())

	// This clock will give a current slot of 0, with now coming more than max clock disparity before slot 1
	disparate := now.Add(-2 * params.BeaconConfig().MaximumGossipClockDisparityDuration())
	dispClock := startup.NewClock(genesis, [32]byte{}, startup.WithNower(func() time.Time { return disparate }))
	// Set up initializer to use the clock that will set now to a little to far before slot 1
	ini = Initializer{shared: &sharedResources{clock: dispClock}}
	v = ini.NewBlobVerifier(b, GossipSidecarRequirements)
	require.ErrorIs(t, v.NotFromFutureSlot(), ErrFromFutureSlot)
	require.Equal(t, true, v.results.executed(RequireNotFromFutureSlot))
	require.NotNil(t, v.results.result(RequireNotFromFutureSlot))
}

func TestSlotAboveFinalized(t *testing.T) {
	ini := &Initializer{shared: &sharedResources{}}
	cases := []struct {
		name          string
		slot          primitives.Slot
		finalizedSlot primitives.Slot
		err           error
	}{
		{
			name: "finalized epoch < blob epoch",
			slot: 32,
		},
		{
			name: "finalized slot < blob slot (same epoch)",
			slot: 31,
		},
		{
			name:          "finalized epoch > blob epoch",
			finalizedSlot: 32,
			err:           ErrSlotNotAfterFinalized,
		},
		{
			name:          "finalized slot == blob slot",
			slot:          35,
			finalizedSlot: 35,
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			finalizedCB := func() *forkchoicetypes.Checkpoint {
				return &forkchoicetypes.Checkpoint{
					Epoch: slots.ToEpoch(c.finalizedSlot),
					Root:  [32]byte{},
				}
			}
			ini.shared.fc = &mockForkchoicer{FinalizedCheckpointCB: finalizedCB}
			_, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 0, 1)
			b := blobs[0]
			b.SignedBlockHeader.Header.Slot = c.slot
			v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
			err := v.SlotAboveFinalized()
			require.Equal(t, true, v.results.executed(RequireSlotAboveFinalized))
			if c.err == nil {
				require.NoError(t, err)
				require.NoError(t, v.results.result(RequireSlotAboveFinalized))
			} else {
				require.ErrorIs(t, err, c.err)
				require.NotNil(t, v.results.result(RequireSlotAboveFinalized))
			}
		})
	}
}

func TestValidProposerSignature_Cached(t *testing.T) {
	ctx := context.Background()
	_, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 0, 1)
	b := blobs[0]
	expectedSd := blobToSignatureData(b)
	sc := &mockSignatureCache{
		svcb: func(sig SignatureData) (bool, error) {
			if sig != expectedSd {
				t.Error("Did not see expected SignatureData")
			}
			return true, nil
		},
		vscb: func(sig SignatureData, v ValidatorAtIndexer) (err error) {
			t.Error("VerifySignature should not be called if the result is cached")
			return nil
		},
	}
	ini := Initializer{shared: &sharedResources{sc: sc, sr: &mockStateByRooter{sbr: sbrErrorIfCalled(t)}}}
	v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
	require.NoError(t, v.ValidProposerSignature(ctx))
	require.Equal(t, true, v.results.executed(RequireValidProposerSignature))
	require.NoError(t, v.results.result(RequireValidProposerSignature))

	// simulate an error in the cache - indicating the previous verification failed
	sc.svcb = func(sig SignatureData) (bool, error) {
		if sig != expectedSd {
			t.Error("Did not see expected SignatureData")
		}
		return true, errors.New("derp")
	}
	ini = Initializer{shared: &sharedResources{sc: sc, sr: &mockStateByRooter{sbr: sbrErrorIfCalled(t)}}}
	v = ini.NewBlobVerifier(b, GossipSidecarRequirements)
	require.ErrorIs(t, v.ValidProposerSignature(ctx), ErrInvalidProposerSignature)
	require.Equal(t, true, v.results.executed(RequireValidProposerSignature))
	require.NotNil(t, v.results.result(RequireValidProposerSignature))
}

func TestValidProposerSignature_CacheMiss(t *testing.T) {
	ctx := context.Background()
	_, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 0, 1)
	b := blobs[0]
	expectedSd := blobToSignatureData(b)
	sc := &mockSignatureCache{
		svcb: func(sig SignatureData) (bool, error) {
			return false, nil
		},
		vscb: func(sig SignatureData, v ValidatorAtIndexer) (err error) {
			if expectedSd != sig {
				t.Error("unexpected signature data")
			}
			return nil
		},
	}
	ini := Initializer{shared: &sharedResources{sc: sc, sr: sbrForValOverride(b.ProposerIndex(), &ethpb.Validator{})}}
	v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
	require.NoError(t, v.ValidProposerSignature(ctx))
	require.Equal(t, true, v.results.executed(RequireValidProposerSignature))
	require.NoError(t, v.results.result(RequireValidProposerSignature))

	// simulate state not found
	ini = Initializer{shared: &sharedResources{sc: sc, sr: sbrNotFound(t, expectedSd.Parent)}}
	v = ini.NewBlobVerifier(b, GossipSidecarRequirements)
	require.ErrorIs(t, v.ValidProposerSignature(ctx), ErrInvalidProposerSignature)
	require.Equal(t, true, v.results.executed(RequireValidProposerSignature))
	require.NotNil(t, v.results.result(RequireValidProposerSignature))

	// simulate successful state lookup, but sig failure
	sbr := sbrForValOverride(b.ProposerIndex(), &ethpb.Validator{})
	sc = &mockSignatureCache{
		svcb: sc.svcb,
		vscb: func(sig SignatureData, v ValidatorAtIndexer) (err error) {
			if expectedSd != sig {
				t.Error("unexpected signature data")
			}
			return errors.New("signature, not so good!")
		},
	}
	ini = Initializer{shared: &sharedResources{sc: sc, sr: sbr}}
	v = ini.NewBlobVerifier(b, GossipSidecarRequirements)

	// make sure all the histories are clean before calling the method
	// so we don't get polluted by previous usages
	require.Equal(t, false, sbr.calledForRoot[expectedSd.Parent])
	require.Equal(t, false, sc.svCalledForSig[expectedSd])
	require.Equal(t, false, sc.vsCalledForSig[expectedSd])

	// Here we're mainly checking that all the right interfaces get used in the unhappy path
	require.ErrorIs(t, v.ValidProposerSignature(ctx), ErrInvalidProposerSignature)
	require.Equal(t, true, sbr.calledForRoot[expectedSd.Parent])
	require.Equal(t, true, sc.svCalledForSig[expectedSd])
	require.Equal(t, true, sc.vsCalledForSig[expectedSd])
	require.Equal(t, true, v.results.executed(RequireValidProposerSignature))
	require.NotNil(t, v.results.result(RequireValidProposerSignature))
}

func badParentCb(t *testing.T, expected [32]byte, e bool) func([32]byte) bool {
	return func(r [32]byte) bool {
		if expected != r {
			t.Error("badParent callback did not receive expected root")
		}
		return e
	}
}

func TestSidecarParentSeen(t *testing.T) {
	_, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 0, 1)
	b := blobs[0]

	fcHas := &mockForkchoicer{
		HasNodeCB: func(parent [32]byte) bool {
			if parent != b.ParentRoot() {
				t.Error("forkchoice.HasNode called with unexpected parent root")
			}
			return true
		},
	}
	fcLacks := &mockForkchoicer{
		HasNodeCB: func(parent [32]byte) bool {
			if parent != b.ParentRoot() {
				t.Error("forkchoice.HasNode called with unexpected parent root")
			}
			return false
		},
	}

	t.Run("happy path", func(t *testing.T) {
		ini := Initializer{shared: &sharedResources{fc: fcHas}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.NoError(t, v.SidecarParentSeen(nil))
		require.Equal(t, true, v.results.executed(RequireSidecarParentSeen))
		require.NoError(t, v.results.result(RequireSidecarParentSeen))
	})
	t.Run("HasNode false, no badParent cb, expected error", func(t *testing.T) {
		ini := Initializer{shared: &sharedResources{fc: fcLacks}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.ErrorIs(t, v.SidecarParentSeen(nil), ErrSidecarParentNotSeen)
		require.Equal(t, true, v.results.executed(RequireSidecarParentSeen))
		require.NotNil(t, v.results.result(RequireSidecarParentSeen))
	})

	t.Run("HasNode false, badParent true", func(t *testing.T) {
		ini := Initializer{shared: &sharedResources{fc: fcLacks}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.NoError(t, v.SidecarParentSeen(badParentCb(t, b.ParentRoot(), true)))
		require.Equal(t, true, v.results.executed(RequireSidecarParentSeen))
		require.NoError(t, v.results.result(RequireSidecarParentSeen))
	})
	t.Run("HasNode false, badParent false", func(t *testing.T) {
		ini := Initializer{shared: &sharedResources{fc: fcLacks}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.ErrorIs(t, v.SidecarParentSeen(badParentCb(t, b.ParentRoot(), false)), ErrSidecarParentNotSeen)
		require.Equal(t, true, v.results.executed(RequireSidecarParentSeen))
		require.NotNil(t, v.results.result(RequireSidecarParentSeen))
	})
}

func TestSidecarParentValid(t *testing.T) {
	_, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 0, 1)
	b := blobs[0]
	t.Run("parent valid", func(t *testing.T) {
		ini := Initializer{shared: &sharedResources{}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.NoError(t, v.SidecarParentValid(badParentCb(t, b.ParentRoot(), false)))
		require.Equal(t, true, v.results.executed(RequireSidecarParentValid))
		require.NoError(t, v.results.result(RequireSidecarParentValid))
	})
	t.Run("parent not valid", func(t *testing.T) {
		ini := Initializer{shared: &sharedResources{}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.ErrorIs(t, v.SidecarParentValid(badParentCb(t, b.ParentRoot(), true)), ErrSidecarParentInvalid)
		require.Equal(t, true, v.results.executed(RequireSidecarParentValid))
		require.NotNil(t, v.results.result(RequireSidecarParentValid))
	})
}

func TestSidecarParentSlotLower(t *testing.T) {
	_, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 1, 1)
	b := blobs[0]
	cases := []struct {
		name   string
		fcSlot primitives.Slot
		fcErr  error
		err    error
	}{
		{
			name:  "not in fc",
			fcErr: errors.New("not in forkchoice"),
			err:   ErrSlotNotAfterParent,
		},
		{
			name:   "in fc, slot lower",
			fcSlot: b.Slot() - 1,
		},
		{
			name:   "in fc, slot equal",
			fcSlot: b.Slot(),
			err:    ErrSlotNotAfterParent,
		},
		{
			name:   "in fc, slot higher",
			fcSlot: b.Slot() + 1,
			err:    ErrSlotNotAfterParent,
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			ini := Initializer{shared: &sharedResources{fc: &mockForkchoicer{SlotCB: func(r [32]byte) (primitives.Slot, error) {
				if b.ParentRoot() != r {
					t.Error("forkchoice.Slot called with unexpected parent root")
				}
				return c.fcSlot, c.fcErr
			}}}}
			v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
			err := v.SidecarParentSlotLower()
			require.Equal(t, true, v.results.executed(RequireSidecarParentSlotLower))
			if c.err == nil {
				require.NoError(t, err)
				require.NoError(t, v.results.result(RequireSidecarParentSlotLower))
			} else {
				require.ErrorIs(t, err, c.err)
				require.NotNil(t, v.results.result(RequireSidecarParentSlotLower))
			}
		})
	}
}

func TestSidecarDescendsFromFinalized(t *testing.T) {
	_, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 1, 1)
	b := blobs[0]
	t.Run("not canonical", func(t *testing.T) {
		ini := Initializer{shared: &sharedResources{fc: &mockForkchoicer{HasNodeCB: func(r [32]byte) bool {
			if b.ParentRoot() != r {
				t.Error("forkchoice.Slot called with unexpected parent root")
			}
			return false
		}}}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.ErrorIs(t, v.SidecarDescendsFromFinalized(), ErrSidecarNotFinalizedDescendent)
		require.Equal(t, true, v.results.executed(RequireSidecarDescendsFromFinalized))
		require.NotNil(t, v.results.result(RequireSidecarDescendsFromFinalized))
	})
	t.Run("canonical", func(t *testing.T) {
		ini := Initializer{shared: &sharedResources{fc: &mockForkchoicer{HasNodeCB: func(r [32]byte) bool {
			if b.ParentRoot() != r {
				t.Error("forkchoice.Slot called with unexpected parent root")
			}
			return true
		}}}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.NoError(t, v.SidecarDescendsFromFinalized())
		require.Equal(t, true, v.results.executed(RequireSidecarDescendsFromFinalized))
		require.NoError(t, v.results.result(RequireSidecarDescendsFromFinalized))
	})
}

func TestSidecarInclusionProven(t *testing.T) {
	// GenerateTestDenebBlockWithSidecar is supposed to generate valid inclusion proofs
	_, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 1, 1)
	b := blobs[0]

	ini := Initializer{}
	v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
	require.NoError(t, v.SidecarInclusionProven())
	require.Equal(t, true, v.results.executed(RequireSidecarInclusionProven))
	require.NoError(t, v.results.result(RequireSidecarInclusionProven))

	// Invert bits of the first byte of the body root to mess up the proof
	byte0 := b.SignedBlockHeader.Header.BodyRoot[0]
	b.SignedBlockHeader.Header.BodyRoot[0] = byte0 ^ 255
	v = ini.NewBlobVerifier(b, GossipSidecarRequirements)
	require.ErrorIs(t, v.SidecarInclusionProven(), ErrSidecarInclusionProofInvalid)
	require.Equal(t, true, v.results.executed(RequireSidecarInclusionProven))
	require.NotNil(t, v.results.result(RequireSidecarInclusionProven))
}

func TestSidecarKzgProofVerified(t *testing.T) {
	// GenerateTestDenebBlockWithSidecar is supposed to generate valid commitments
	_, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 1, 1)
	b := blobs[0]
	passes := func(vb ...blocks.ROBlob) error {
		require.Equal(t, true, bytes.Equal(b.KzgCommitment, vb[0].KzgCommitment))
		return nil
	}
	v := &ROBlobVerifier{verifyBlobCommitment: passes, results: newResults(), blob: b}
	require.NoError(t, v.SidecarKzgProofVerified())
	require.Equal(t, true, v.results.executed(RequireSidecarKzgProofVerified))
	require.NoError(t, v.results.result(RequireSidecarKzgProofVerified))

	fails := func(vb ...blocks.ROBlob) error {
		require.Equal(t, true, bytes.Equal(b.KzgCommitment, vb[0].KzgCommitment))
		return errors.New("bad blob")
	}
	v = &ROBlobVerifier{results: newResults(), blob: b, verifyBlobCommitment: fails}
	require.ErrorIs(t, v.SidecarKzgProofVerified(), ErrSidecarKzgProofInvalid)
	require.Equal(t, true, v.results.executed(RequireSidecarKzgProofVerified))
	require.NotNil(t, v.results.result(RequireSidecarKzgProofVerified))
}

func TestSidecarProposerExpected(t *testing.T) {
	ctx := context.Background()
	_, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 1, 1)
	b := blobs[0]
	t.Run("cached, matches", func(t *testing.T) {
		ini := Initializer{shared: &sharedResources{pc: &mockProposerCache{ProposerCB: pcReturnsIdx(b.ProposerIndex())}, fc: &mockForkchoicer{TargetRootForEpochCB: fcReturnsTargetRoot([32]byte{})}}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.NoError(t, v.SidecarProposerExpected(ctx))
		require.Equal(t, true, v.results.executed(RequireSidecarProposerExpected))
		require.NoError(t, v.results.result(RequireSidecarProposerExpected))
	})
	t.Run("cached, does not match", func(t *testing.T) {
		ini := Initializer{shared: &sharedResources{pc: &mockProposerCache{ProposerCB: pcReturnsIdx(b.ProposerIndex() + 1)}, fc: &mockForkchoicer{TargetRootForEpochCB: fcReturnsTargetRoot([32]byte{})}}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.ErrorIs(t, v.SidecarProposerExpected(ctx), ErrSidecarUnexpectedProposer)
		require.Equal(t, true, v.results.executed(RequireSidecarProposerExpected))
		require.NotNil(t, v.results.result(RequireSidecarProposerExpected))
	})
	t.Run("not cached, state lookup failure", func(t *testing.T) {
		ini := Initializer{shared: &sharedResources{sr: sbrNotFound(t, b.ParentRoot()), pc: &mockProposerCache{ProposerCB: pcReturnsNotFound()}, fc: &mockForkchoicer{TargetRootForEpochCB: fcReturnsTargetRoot([32]byte{})}}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.ErrorIs(t, v.SidecarProposerExpected(ctx), ErrSidecarUnexpectedProposer)
		require.Equal(t, true, v.results.executed(RequireSidecarProposerExpected))
		require.NotNil(t, v.results.result(RequireSidecarProposerExpected))
	})

	t.Run("not cached, proposer matches", func(t *testing.T) {
		pc := &mockProposerCache{
			ProposerCB: pcReturnsNotFound(),
			ComputeProposerCB: func(ctx context.Context, root [32]byte, slot primitives.Slot, pst state.BeaconState) (primitives.ValidatorIndex, error) {
				require.Equal(t, b.ParentRoot(), root)
				require.Equal(t, b.Slot(), slot)
				return b.ProposerIndex(), nil
			},
		}
		ini := Initializer{shared: &sharedResources{sr: sbrForValOverride(b.ProposerIndex(), &ethpb.Validator{}), pc: pc, fc: &mockForkchoicer{TargetRootForEpochCB: fcReturnsTargetRoot([32]byte{})}}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.NoError(t, v.SidecarProposerExpected(ctx))
		require.Equal(t, true, v.results.executed(RequireSidecarProposerExpected))
		require.NoError(t, v.results.result(RequireSidecarProposerExpected))
	})
	t.Run("not cached, proposer does not match", func(t *testing.T) {
		pc := &mockProposerCache{
			ProposerCB: pcReturnsNotFound(),
			ComputeProposerCB: func(ctx context.Context, root [32]byte, slot primitives.Slot, pst state.BeaconState) (primitives.ValidatorIndex, error) {
				require.Equal(t, b.ParentRoot(), root)
				require.Equal(t, b.Slot(), slot)
				return b.ProposerIndex() + 1, nil
			},
		}
		ini := Initializer{shared: &sharedResources{sr: sbrForValOverride(b.ProposerIndex(), &ethpb.Validator{}), pc: pc, fc: &mockForkchoicer{TargetRootForEpochCB: fcReturnsTargetRoot([32]byte{})}}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.ErrorIs(t, v.SidecarProposerExpected(ctx), ErrSidecarUnexpectedProposer)
		require.Equal(t, true, v.results.executed(RequireSidecarProposerExpected))
		require.NotNil(t, v.results.result(RequireSidecarProposerExpected))
	})
	t.Run("not cached, ComputeProposer fails", func(t *testing.T) {
		pc := &mockProposerCache{
			ProposerCB: pcReturnsNotFound(),
			ComputeProposerCB: func(ctx context.Context, root [32]byte, slot primitives.Slot, pst state.BeaconState) (primitives.ValidatorIndex, error) {
				require.Equal(t, b.ParentRoot(), root)
				require.Equal(t, b.Slot(), slot)
				return 0, errors.New("ComputeProposer failed")
			},
		}
		ini := Initializer{shared: &sharedResources{sr: sbrForValOverride(b.ProposerIndex(), &ethpb.Validator{}), pc: pc, fc: &mockForkchoicer{TargetRootForEpochCB: fcReturnsTargetRoot([32]byte{})}}}
		v := ini.NewBlobVerifier(b, GossipSidecarRequirements)
		require.ErrorIs(t, v.SidecarProposerExpected(ctx), ErrSidecarUnexpectedProposer)
		require.Equal(t, true, v.results.executed(RequireSidecarProposerExpected))
		require.NotNil(t, v.results.result(RequireSidecarProposerExpected))
	})
}

func TestRequirementSatisfaction(t *testing.T) {
	_, blobs := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, 1, 1)
	b := blobs[0]
	ini := Initializer{}
	v := ini.NewBlobVerifier(b, GossipSidecarRequirements)

	_, err := v.VerifiedROBlob()
	require.ErrorIs(t, err, ErrBlobInvalid)
	me, ok := err.(VerificationMultiError)
	require.Equal(t, true, ok)
	fails := me.Failures()
	// we haven't performed any verification, so all the results should be this type
	for _, v := range fails {
		require.ErrorIs(t, v, ErrMissingVerification)
	}

	// satisfy everything through the backdoor and ensure we get the verified ro blob at the end
	for _, r := range GossipSidecarRequirements {
		v.results.record(r, nil)
	}
	require.Equal(t, true, v.results.allSatisfied())
	_, err = v.VerifiedROBlob()
	require.NoError(t, err)
}

type mockForkchoicer struct {
	FinalizedCheckpointCB func() *forkchoicetypes.Checkpoint
	HasNodeCB             func([32]byte) bool
	IsCanonicalCB         func(root [32]byte) bool
	SlotCB                func([32]byte) (primitives.Slot, error)
	TargetRootForEpochCB  func([32]byte, primitives.Epoch) ([32]byte, error)
}

var _ Forkchoicer = &mockForkchoicer{}

func (m *mockForkchoicer) FinalizedCheckpoint() *forkchoicetypes.Checkpoint {
	return m.FinalizedCheckpointCB()
}

func (m *mockForkchoicer) HasNode(root [32]byte) bool {
	return m.HasNodeCB(root)
}

func (m *mockForkchoicer) IsCanonical(root [32]byte) bool {
	return m.IsCanonicalCB(root)
}

func (m *mockForkchoicer) Slot(root [32]byte) (primitives.Slot, error) {
	return m.SlotCB(root)
}

func (m *mockForkchoicer) TargetRootForEpoch(root [32]byte, epoch primitives.Epoch) ([32]byte, error) {
	return m.TargetRootForEpochCB(root, epoch)
}

func fcReturnsTargetRoot(root [32]byte) func([32]byte, primitives.Epoch) ([32]byte, error) {
	return func([32]byte, primitives.Epoch) ([32]byte, error) {
		return root, nil
	}
}

type mockSignatureCache struct {
	svCalledForSig map[SignatureData]bool
	svcb           func(sig SignatureData) (bool, error)
	vsCalledForSig map[SignatureData]bool
	vscb           func(sig SignatureData, v ValidatorAtIndexer) (err error)
}

// SignatureVerified implements SignatureCache.
func (m *mockSignatureCache) SignatureVerified(sig SignatureData) (bool, error) {
	if m.svCalledForSig == nil {
		m.svCalledForSig = make(map[SignatureData]bool)
	}
	m.svCalledForSig[sig] = true
	return m.svcb(sig)
}

// VerifySignature implements SignatureCache.
func (m *mockSignatureCache) VerifySignature(sig SignatureData, v ValidatorAtIndexer) (err error) {
	if m.vsCalledForSig == nil {
		m.vsCalledForSig = make(map[SignatureData]bool)
	}
	m.vsCalledForSig[sig] = true
	return m.vscb(sig, v)
}

var _ SignatureCache = &mockSignatureCache{}

type sbrfunc func(context.Context, [32]byte) (state.BeaconState, error)

type mockStateByRooter struct {
	sbr           sbrfunc
	calledForRoot map[[32]byte]bool
}

func (sbr *mockStateByRooter) StateByRoot(ctx context.Context, root [32]byte) (state.BeaconState, error) {
	if sbr.calledForRoot == nil {
		sbr.calledForRoot = make(map[[32]byte]bool)
	}
	sbr.calledForRoot[root] = true
	return sbr.sbr(ctx, root)
}

var _ StateByRooter = &mockStateByRooter{}

func sbrErrorIfCalled(t *testing.T) sbrfunc {
	return func(_ context.Context, _ [32]byte) (state.BeaconState, error) {
		t.Error("StateByRoot should not have been called")
		return nil, nil
	}
}

func sbrNotFound(t *testing.T, expectedRoot [32]byte) *mockStateByRooter {
	return &mockStateByRooter{sbr: func(_ context.Context, parent [32]byte) (state.BeaconState, error) {
		if parent != expectedRoot {
			t.Errorf("did not receive expected root in StateByRootCall, want %#x got %#x", expectedRoot, parent)
		}
		return nil, db.ErrNotFound
	}}
}

func sbrForValOverride(idx primitives.ValidatorIndex, val *ethpb.Validator) *mockStateByRooter {
	return &mockStateByRooter{sbr: func(_ context.Context, root [32]byte) (state.BeaconState, error) {
		return &validxStateOverride{vals: map[primitives.ValidatorIndex]*ethpb.Validator{
			idx: val,
		}}, nil
	}}
}

type validxStateOverride struct {
	state.BeaconState
	vals map[primitives.ValidatorIndex]*ethpb.Validator
}

var _ state.BeaconState = &validxStateOverride{}

func (v *validxStateOverride) ValidatorAtIndex(idx primitives.ValidatorIndex) (*ethpb.Validator, error) {
	val, ok := v.vals[idx]
	if !ok {
		return nil, fmt.Errorf("validxStateOverride does not know index %d", idx)
	}
	return val, nil
}

type mockProposerCache struct {
	ComputeProposerCB func(ctx context.Context, root [32]byte, slot primitives.Slot, pst state.BeaconState) (primitives.ValidatorIndex, error)
	ProposerCB        func(c *forkchoicetypes.Checkpoint, slot primitives.Slot) (primitives.ValidatorIndex, bool)
}

func (p *mockProposerCache) ComputeProposer(ctx context.Context, root [32]byte, slot primitives.Slot, pst state.BeaconState) (primitives.ValidatorIndex, error) {
	return p.ComputeProposerCB(ctx, root, slot, pst)
}

func (p *mockProposerCache) Proposer(c *forkchoicetypes.Checkpoint, slot primitives.Slot) (primitives.ValidatorIndex, bool) {
	return p.ProposerCB(c, slot)
}

var _ ProposerCache = &mockProposerCache{}

func pcReturnsIdx(idx primitives.ValidatorIndex) func(c *forkchoicetypes.Checkpoint, slot primitives.Slot) (primitives.ValidatorIndex, bool) {
	return func(c *forkchoicetypes.Checkpoint, slot primitives.Slot) (primitives.ValidatorIndex, bool) {
		return idx, true
	}
}

func pcReturnsNotFound() func(c *forkchoicetypes.Checkpoint, slot primitives.Slot) (primitives.ValidatorIndex, bool) {
	return func(c *forkchoicetypes.Checkpoint, slot primitives.Slot) (primitives.ValidatorIndex, bool) {
		return 0, false
	}
}
