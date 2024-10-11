package slashings

import (
	"context"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func proposerSlashingForValIdx(valIdx primitives.ValidatorIndex) *ethpb.ProposerSlashing {
	return &ethpb.ProposerSlashing{
		Header_1: &ethpb.SignedBeaconBlockHeader{
			Header: &ethpb.BeaconBlockHeader{ProposerIndex: valIdx},
		},
		Header_2: &ethpb.SignedBeaconBlockHeader{
			Header: &ethpb.BeaconBlockHeader{ProposerIndex: valIdx},
		},
	}
}

func TestPool_InsertProposerSlashing(t *testing.T) {
	type fields struct {
		wantedErr string
		pending   []*ethpb.ProposerSlashing
		included  map[primitives.ValidatorIndex]bool
	}
	type args struct {
		slashings []*ethpb.ProposerSlashing
	}

	beaconState, privKeys := util.DeterministicGenesisState(t, 64)
	slashings := make([]*ethpb.ProposerSlashing, 20)
	for i := 0; i < len(slashings); i++ {
		sl, err := util.GenerateProposerSlashingForValidator(beaconState, privKeys[i], primitives.ValidatorIndex(i))
		require.NoError(t, err)
		slashings[i] = sl
	}

	require.NoError(t, beaconState.SetSlot(params.BeaconConfig().SlotsPerEpoch))

	// We mark the following validators with some preconditions.
	exitedVal, err := beaconState.ValidatorAtIndex(primitives.ValidatorIndex(2))
	require.NoError(t, err)
	exitedVal.WithdrawableEpoch = 0
	futureExitedVal, err := beaconState.ValidatorAtIndex(primitives.ValidatorIndex(4))
	require.NoError(t, err)
	futureExitedVal.WithdrawableEpoch = 17
	slashedVal, err := beaconState.ValidatorAtIndex(primitives.ValidatorIndex(5))
	require.NoError(t, err)
	slashedVal.Slashed = true
	require.NoError(t, beaconState.UpdateValidatorAtIndex(primitives.ValidatorIndex(2), exitedVal))
	require.NoError(t, beaconState.UpdateValidatorAtIndex(primitives.ValidatorIndex(4), futureExitedVal))
	require.NoError(t, beaconState.UpdateValidatorAtIndex(primitives.ValidatorIndex(5), slashedVal))

	tests := []struct {
		name   string
		fields fields
		args   args
		want   []*ethpb.ProposerSlashing
	}{
		{
			name: "Empty list",
			fields: fields{
				pending:  make([]*ethpb.ProposerSlashing, 0),
				included: make(map[primitives.ValidatorIndex]bool),
			},
			args: args{
				slashings: slashings[0:1],
			},
			want: slashings[0:1],
		},
		{
			name: "Duplicate identical slashing",
			fields: fields{
				pending:   slashings[0:1],
				included:  make(map[primitives.ValidatorIndex]bool),
				wantedErr: "slashing object already exists in pending proposer slashings",
			},
			args: args{
				slashings: slashings[0:1],
			},
			want: slashings[0:1],
		},
		{
			name: "Slashing for exited validator",
			fields: fields{
				pending:   []*ethpb.ProposerSlashing{},
				included:  make(map[primitives.ValidatorIndex]bool),
				wantedErr: "is not slashable",
			},
			args: args{
				slashings: slashings[2:3],
			},
			want: []*ethpb.ProposerSlashing{},
		},
		{
			name: "Slashing for exiting validator",
			fields: fields{
				pending:  []*ethpb.ProposerSlashing{},
				included: make(map[primitives.ValidatorIndex]bool),
			},
			args: args{
				slashings: slashings[4:5],
			},
			want: slashings[4:5],
		},
		{
			name: "Slashing for slashed validator",
			fields: fields{
				pending:   []*ethpb.ProposerSlashing{},
				included:  make(map[primitives.ValidatorIndex]bool),
				wantedErr: "not slashable",
			},
			args: args{
				slashings: slashings[5:6],
			},
			want: []*ethpb.ProposerSlashing{},
		},
		{
			name: "Already included",
			fields: fields{
				pending: []*ethpb.ProposerSlashing{},
				included: map[primitives.ValidatorIndex]bool{
					1: true,
				},
				wantedErr: "cannot be slashed",
			},
			args: args{
				slashings: slashings[1:2],
			},
			want: []*ethpb.ProposerSlashing{},
		},
		{
			name: "Maintains sorted order",
			fields: fields{
				pending: []*ethpb.ProposerSlashing{
					slashings[0],
					slashings[2],
				},
				included: make(map[primitives.ValidatorIndex]bool),
			},
			args: args{
				slashings: slashings[1:2],
			},
			want: []*ethpb.ProposerSlashing{
				slashings[0],
				slashings[1],
				slashings[2],
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p := &Pool{
				pendingProposerSlashing: tt.fields.pending,
				included:                tt.fields.included,
			}
			var err error
			for i := 0; i < len(tt.args.slashings); i++ {
				err = p.InsertProposerSlashing(context.Background(), beaconState, tt.args.slashings[i])
			}
			if tt.fields.wantedErr != "" {
				require.ErrorContains(t, tt.fields.wantedErr, err)
			} else {
				require.NoError(t, err)
			}
			assert.Equal(t, len(tt.want), len(p.pendingProposerSlashing))
			for i := range p.pendingAttesterSlashing {
				assert.Equal(t, p.pendingProposerSlashing[i].Header_1.Header.ProposerIndex, tt.want[i].Header_1.Header.ProposerIndex)
				assert.DeepEqual(t, tt.want[i], p.pendingProposerSlashing[i], "Proposer slashing at index %d does not match expected", i)
			}
		})
	}
}

func TestPool_InsertProposerSlashing_SigFailsVerify_ClearPool(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	conf := params.BeaconConfig()
	conf.MaxAttesterSlashings = 2
	params.OverrideBeaconConfig(conf)
	beaconState, privKeys := util.DeterministicGenesisState(t, 64)
	slashings := make([]*ethpb.ProposerSlashing, 2)
	for i := 0; i < 2; i++ {
		sl, err := util.GenerateProposerSlashingForValidator(beaconState, privKeys[i], primitives.ValidatorIndex(i))
		require.NoError(t, err)
		slashings[i] = sl
	}
	// We mess up the signature of the second slashing.
	badSig := make([]byte, fieldparams.BLSSignatureLength)
	copy(badSig, "muahaha")
	slashings[1].Header_1.Signature = badSig
	p := &Pool{
		pendingProposerSlashing: make([]*ethpb.ProposerSlashing, 0),
	}
	// We only want a single slashing to remain.
	require.NoError(t, p.InsertProposerSlashing(context.Background(), beaconState, slashings[0]))
	err := p.InsertProposerSlashing(context.Background(), beaconState, slashings[1])
	require.ErrorContains(t, "could not verify proposer slashing", err, "Expected slashing with bad signature to fail")
	assert.Equal(t, 1, len(p.pendingProposerSlashing))
}

func TestPool_MarkIncludedProposerSlashing(t *testing.T) {
	type fields struct {
		pending  []*ethpb.ProposerSlashing
		included map[primitives.ValidatorIndex]bool
	}
	type args struct {
		slashing *ethpb.ProposerSlashing
	}
	tests := []struct {
		name   string
		fields fields
		args   args
		want   fields
	}{
		{
			name: "Included, does not exist in pending",
			fields: fields{
				pending: []*ethpb.ProposerSlashing{
					proposerSlashingForValIdx(1),
				},
				included: make(map[primitives.ValidatorIndex]bool),
			},
			args: args{
				slashing: proposerSlashingForValIdx(3),
			},
			want: fields{
				pending: []*ethpb.ProposerSlashing{
					proposerSlashingForValIdx(1),
				},
				included: map[primitives.ValidatorIndex]bool{
					3: true,
				},
			},
		},
		{
			name: "Removes from pending list",
			fields: fields{
				pending: []*ethpb.ProposerSlashing{
					proposerSlashingForValIdx(1),
					proposerSlashingForValIdx(2),
					proposerSlashingForValIdx(3),
				},
				included: map[primitives.ValidatorIndex]bool{
					0: true,
				},
			},
			args: args{
				slashing: proposerSlashingForValIdx(2),
			},
			want: fields{
				pending: []*ethpb.ProposerSlashing{
					proposerSlashingForValIdx(1),
					proposerSlashingForValIdx(3),
				},
				included: map[primitives.ValidatorIndex]bool{
					0: true,
					2: true,
				},
			},
		},
		{
			name: "Removes from pending long list",
			fields: fields{
				pending: []*ethpb.ProposerSlashing{
					proposerSlashingForValIdx(1),
					proposerSlashingForValIdx(2),
					proposerSlashingForValIdx(3),
					proposerSlashingForValIdx(4),
					proposerSlashingForValIdx(5),
					proposerSlashingForValIdx(6),
					proposerSlashingForValIdx(7),
					proposerSlashingForValIdx(8),
					proposerSlashingForValIdx(9),
					proposerSlashingForValIdx(10),
				},
				included: map[primitives.ValidatorIndex]bool{
					0: true,
				},
			},
			args: args{
				slashing: proposerSlashingForValIdx(7),
			},
			want: fields{
				pending: []*ethpb.ProposerSlashing{
					proposerSlashingForValIdx(1),
					proposerSlashingForValIdx(2),
					proposerSlashingForValIdx(3),
					proposerSlashingForValIdx(4),
					proposerSlashingForValIdx(5),
					proposerSlashingForValIdx(6),
					proposerSlashingForValIdx(8),
					proposerSlashingForValIdx(9),
					proposerSlashingForValIdx(10),
				},
				included: map[primitives.ValidatorIndex]bool{
					0: true,
					7: true,
				},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p := &Pool{
				pendingProposerSlashing: tt.fields.pending,
				included:                tt.fields.included,
			}
			p.MarkIncludedProposerSlashing(tt.args.slashing)
			assert.Equal(t, len(tt.want.pending), len(p.pendingProposerSlashing))
			for i := range p.pendingProposerSlashing {
				assert.DeepSSZEqual(t, tt.want.pending[i], p.pendingProposerSlashing[i], "Unexpected pending proposer slashing at index %d", i)
			}
			assert.DeepEqual(t, tt.want.included, p.included)
		})
	}
}

func TestPool_PendingProposerSlashings(t *testing.T) {
	type fields struct {
		pending []*ethpb.ProposerSlashing
		noLimit bool
	}
	beaconState, privKeys := util.DeterministicGenesisState(t, 64)
	slashings := make([]*ethpb.ProposerSlashing, 20)
	for i := 0; i < len(slashings); i++ {
		sl, err := util.GenerateProposerSlashingForValidator(beaconState, privKeys[i], primitives.ValidatorIndex(i))
		require.NoError(t, err)
		slashings[i] = sl
	}
	tests := []struct {
		name   string
		fields fields
		want   []*ethpb.ProposerSlashing
	}{
		{
			name: "Empty list",
			fields: fields{
				pending: []*ethpb.ProposerSlashing{},
			},
			want: []*ethpb.ProposerSlashing{},
		},
		{
			name: "All",
			fields: fields{
				pending: slashings,
				noLimit: true,
			},
			want: slashings,
		},
		{
			name: "All block eligible",
			fields: fields{
				pending: slashings[:params.BeaconConfig().MaxProposerSlashings],
			},
			want: slashings[:params.BeaconConfig().MaxProposerSlashings],
		},
		{
			name: "Multiple indices",
			fields: fields{
				pending: slashings[3:6],
			},
			want: slashings[3:6],
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p := &Pool{
				pendingProposerSlashing: tt.fields.pending,
			}
			assert.DeepEqual(t, tt.want, p.PendingProposerSlashings(context.Background(), beaconState, tt.fields.noLimit))
		})
	}
}

func TestPool_PendingProposerSlashings_Slashed(t *testing.T) {
	type fields struct {
		all     bool
		pending []*ethpb.ProposerSlashing
	}
	beaconState, privKeys := util.DeterministicGenesisState(t, 64)
	val, err := beaconState.ValidatorAtIndex(0)
	require.NoError(t, err)
	val.Slashed = true
	require.NoError(t, beaconState.UpdateValidatorAtIndex(0, val))
	val, err = beaconState.ValidatorAtIndex(5)
	require.NoError(t, err)
	val.Slashed = true
	require.NoError(t, beaconState.UpdateValidatorAtIndex(5, val))
	slashings := make([]*ethpb.ProposerSlashing, 32)
	slashings2 := make([]*ethpb.ProposerSlashing, 32)
	result := make([]*ethpb.ProposerSlashing, 32)
	for i := 0; i < len(slashings); i++ {
		sl, err := util.GenerateProposerSlashingForValidator(beaconState, privKeys[i], primitives.ValidatorIndex(i))
		require.NoError(t, err)
		slashings[i] = sl
		slashings2[i] = sl
		result[i] = sl
	}
	result = append(result[1:5], result[6:]...)
	tests := []struct {
		name   string
		fields fields
		want   []*ethpb.ProposerSlashing
	}{
		{
			name: "removes slashed",
			fields: fields{
				pending: slashings,
			},
			want: result[:16],
		},
		{
			name: "gets noLimit and no slashed",
			fields: fields{
				all:     true,
				pending: slashings2,
			},
			want: result,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p := &Pool{
				pendingProposerSlashing: tt.fields.pending,
			}
			result := p.PendingProposerSlashings(context.Background(), beaconState, tt.fields.all /*noLimit*/)
			t.Log(tt.want[0].Header_1.Header.ProposerIndex)
			t.Log(result[0].Header_1.Header.ProposerIndex)
			assert.DeepEqual(t, tt.want, result)
		})
	}
}
