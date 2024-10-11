package beacon

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"google.golang.org/protobuf/proto"

	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/slashings"
	mockp2p "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestServer_SubmitProposerSlashing(t *testing.T) {
	ctx := context.Background()

	st, privs := util.DeterministicGenesisState(t, 64)
	slashedVal, err := st.ValidatorAtIndex(5)
	require.NoError(t, err)
	// We mark the validator at index 5 as already slashed.
	slashedVal.Slashed = true
	require.NoError(t, st.UpdateValidatorAtIndex(5, slashedVal))

	mb := &mockp2p.MockBroadcaster{}
	bs := &Server{
		HeadFetcher: &mock.ChainService{
			State: st,
		},
		SlashingsPool: slashings.NewPool(),
		Broadcaster:   mb,
	}

	// We want a proposer slashing for validator with index 2 to
	// be included in the pool.
	slashing, err := util.GenerateProposerSlashingForValidator(st, privs[2], primitives.ValidatorIndex(2))
	require.NoError(t, err)

	_, err = bs.SubmitProposerSlashing(ctx, slashing)
	require.NoError(t, err)
	assert.Equal(t, true, mb.BroadcastCalled.Load(), "Expected broadcast to be called")
}

func TestServer_SubmitAttesterSlashing(t *testing.T) {
	ctx := context.Background()
	// We mark the validators at index 5, 6 as already slashed.
	st, privs := util.DeterministicGenesisState(t, 64)
	slashedVal, err := st.ValidatorAtIndex(5)
	require.NoError(t, err)

	// We mark the validator at index 5 as already slashed.
	slashedVal.Slashed = true
	require.NoError(t, st.UpdateValidatorAtIndex(5, slashedVal))

	mb := &mockp2p.MockBroadcaster{}
	bs := &Server{
		HeadFetcher: &mock.ChainService{
			State: st,
		},
		SlashingsPool: slashings.NewPool(),
		Broadcaster:   mb,
	}

	slashing, err := util.GenerateAttesterSlashingForValidator(st, privs[2], primitives.ValidatorIndex(2))
	require.NoError(t, err)

	// We want the intersection of the slashing attesting indices
	// to be slashed, so we expect validators 2 and 3 to be in the response
	// slashed indices.
	_, err = bs.SubmitAttesterSlashing(ctx, slashing)
	require.NoError(t, err)
	assert.Equal(t, true, mb.BroadcastCalled.Load(), "Expected broadcast to be called when flag is set")
}

func TestServer_SubmitProposerSlashing_DontBroadcast(t *testing.T) {
	resetCfg := features.InitWithReset(&features.Flags{DisableBroadcastSlashings: true})
	defer resetCfg()
	ctx := context.Background()
	st, privs := util.DeterministicGenesisState(t, 64)
	slashedVal, err := st.ValidatorAtIndex(5)
	require.NoError(t, err)
	// We mark the validator at index 5 as already slashed.
	slashedVal.Slashed = true
	require.NoError(t, st.UpdateValidatorAtIndex(5, slashedVal))

	mb := &mockp2p.MockBroadcaster{}
	bs := &Server{
		HeadFetcher: &mock.ChainService{
			State: st,
		},
		SlashingsPool: slashings.NewPool(),
		Broadcaster:   mb,
	}

	// We want a proposer slashing for validator with index 2 to
	// be included in the pool.
	wanted := &ethpb.SubmitSlashingResponse{
		SlashedIndices: []primitives.ValidatorIndex{2},
	}
	slashing, err := util.GenerateProposerSlashingForValidator(st, privs[2], primitives.ValidatorIndex(2))
	require.NoError(t, err)

	res, err := bs.SubmitProposerSlashing(ctx, slashing)
	require.NoError(t, err)
	if !proto.Equal(wanted, res) {
		t.Errorf("Wanted %v, received %v", wanted, res)
	}

	assert.Equal(t, false, mb.BroadcastCalled.Load(), "Expected broadcast not to be called by default")

	slashing, err = util.GenerateProposerSlashingForValidator(st, privs[5], primitives.ValidatorIndex(5))
	require.NoError(t, err)

	// We do not want a proposer slashing for an already slashed validator
	// (the validator at index 5) to be included in the pool.
	_, err = bs.SubmitProposerSlashing(ctx, slashing)
	require.ErrorContains(t, "Could not insert proposer slashing into pool", err)
}

func TestServer_SubmitAttesterSlashing_DontBroadcast(t *testing.T) {
	resetCfg := features.InitWithReset(&features.Flags{DisableBroadcastSlashings: true})
	defer resetCfg()
	ctx := context.Background()
	// We mark the validators at index 5, 6 as already slashed.
	st, privs := util.DeterministicGenesisState(t, 64)
	slashedVal, err := st.ValidatorAtIndex(5)
	require.NoError(t, err)

	// We mark the validator at index 5 as already slashed.
	slashedVal.Slashed = true
	require.NoError(t, st.UpdateValidatorAtIndex(5, slashedVal))

	mb := &mockp2p.MockBroadcaster{}
	bs := &Server{
		HeadFetcher: &mock.ChainService{
			State: st,
		},
		SlashingsPool: slashings.NewPool(),
		Broadcaster:   mb,
	}

	slashing, err := util.GenerateAttesterSlashingForValidator(st, privs[2], primitives.ValidatorIndex(2))
	require.NoError(t, err)

	// We want the intersection of the slashing attesting indices
	// to be slashed, so we expect validators 2 and 3 to be in the response
	// slashed indices.
	wanted := &ethpb.SubmitSlashingResponse{
		SlashedIndices: []primitives.ValidatorIndex{2},
	}
	res, err := bs.SubmitAttesterSlashing(ctx, slashing)
	require.NoError(t, err)
	if !proto.Equal(wanted, res) {
		t.Errorf("Wanted %v, received %v", wanted, res)
	}
	assert.Equal(t, false, mb.BroadcastCalled.Load(), "Expected broadcast not to be called by default")

	slashing, err = util.GenerateAttesterSlashingForValidator(st, privs[5], primitives.ValidatorIndex(5))
	require.NoError(t, err)
	// If any of the attesting indices in the slashing object have already
	// been slashed, we should fail to insert properly into the attester slashing pool.
	_, err = bs.SubmitAttesterSlashing(ctx, slashing)
	assert.NotNil(t, err, "Expected including a attester slashing for an already slashed validator to fail")
}
