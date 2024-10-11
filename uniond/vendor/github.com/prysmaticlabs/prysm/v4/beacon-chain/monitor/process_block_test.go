package monitor

import (
	"context"
	"fmt"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestProcessSlashings(t *testing.T) {
	tests := []struct {
		name      string
		block     *ethpb.BeaconBlock
		wantedErr string
	}{
		{
			name: "Proposer slashing a tracked index",
			block: &ethpb.BeaconBlock{
				Body: &ethpb.BeaconBlockBody{
					ProposerSlashings: []*ethpb.ProposerSlashing{
						{
							Header_1: &ethpb.SignedBeaconBlockHeader{
								Header: &ethpb.BeaconBlockHeader{
									ProposerIndex: 2,
									Slot:          params.BeaconConfig().SlotsPerEpoch + 1,
								},
							},
							Header_2: &ethpb.SignedBeaconBlockHeader{
								Header: &ethpb.BeaconBlockHeader{
									ProposerIndex: 2,
									Slot:          0,
								},
							},
						},
					},
				},
			},
			wantedErr: "\"Proposer slashing was included\" BodyRoot1= BodyRoot2= ProposerIndex=2",
		},
		{
			name: "Proposer slashing an untracked index",
			block: &ethpb.BeaconBlock{
				Body: &ethpb.BeaconBlockBody{
					ProposerSlashings: []*ethpb.ProposerSlashing{
						{
							Header_1: &ethpb.SignedBeaconBlockHeader{
								Header: &ethpb.BeaconBlockHeader{
									ProposerIndex: 3,
									Slot:          params.BeaconConfig().SlotsPerEpoch + 4,
								},
							},
							Header_2: &ethpb.SignedBeaconBlockHeader{
								Header: &ethpb.BeaconBlockHeader{
									ProposerIndex: 3,
									Slot:          0,
								},
							},
						},
					},
				},
			},
			wantedErr: "",
		},
		{
			name: "Attester slashing a tracked index",
			block: &ethpb.BeaconBlock{
				Body: &ethpb.BeaconBlockBody{
					AttesterSlashings: []*ethpb.AttesterSlashing{
						{
							Attestation_1: util.HydrateIndexedAttestation(&ethpb.IndexedAttestation{
								Data: &ethpb.AttestationData{
									Source: &ethpb.Checkpoint{Epoch: 1},
								},
								AttestingIndices: []uint64{1, 3, 4},
							}),
							Attestation_2: util.HydrateIndexedAttestation(&ethpb.IndexedAttestation{
								AttestingIndices: []uint64{1, 5, 6},
							}),
						},
					},
				},
			},
			wantedErr: "\"Attester slashing was included\" AttestationSlot1=0 AttestationSlot2=0 AttesterIndex=1 " +
				"BeaconBlockRoot1=0x000000000000 BeaconBlockRoot2=0x000000000000 BlockInclusionSlot=0 SourceEpoch1=1 SourceEpoch2=0 TargetEpoch1=0 TargetEpoch2=0",
		},
		{
			name: "Attester slashing untracked index",
			block: &ethpb.BeaconBlock{
				Body: &ethpb.BeaconBlockBody{
					AttesterSlashings: []*ethpb.AttesterSlashing{
						{
							Attestation_1: util.HydrateIndexedAttestation(&ethpb.IndexedAttestation{
								Data: &ethpb.AttestationData{
									Source: &ethpb.Checkpoint{Epoch: 1},
								},
								AttestingIndices: []uint64{1, 3, 4},
							}),
							Attestation_2: util.HydrateIndexedAttestation(&ethpb.IndexedAttestation{
								AttestingIndices: []uint64{3, 5, 6},
							}),
						},
					},
				},
			},
			wantedErr: "",
		}}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			hook := logTest.NewGlobal()
			s := &Service{
				TrackedValidators: map[primitives.ValidatorIndex]bool{
					1: true,
					2: true,
				},
			}
			wb, err := blocks.NewBeaconBlock(tt.block)
			require.NoError(t, err)
			s.processSlashings(wb)
			if tt.wantedErr != "" {
				require.LogsContain(t, hook, tt.wantedErr)
			} else {
				require.LogsDoNotContain(t, hook, "slashing")
			}
		})
	}
}

func TestProcessProposedBlock(t *testing.T) {
	tests := []struct {
		name      string
		block     *ethpb.BeaconBlock
		wantedErr string
	}{
		{
			name: "Block proposed by tracked validator",
			block: &ethpb.BeaconBlock{
				Slot:          6,
				ProposerIndex: 12,
				ParentRoot:    bytesutil.PadTo([]byte("hello-world"), 32),
				StateRoot:     bytesutil.PadTo([]byte("state-world"), 32),
				Body:          &ethpb.BeaconBlockBody{},
			},
			wantedErr: "\"Proposed beacon block was included\" BalanceChange=100000000 BlockRoot=0x68656c6c6f2d NewBalance=32000000000 ParentRoot=0x68656c6c6f2d ProposerIndex=12 Slot=6 Version=0 prefix=monitor",
		},
		{
			name: "Block proposed by untracked validator",
			block: &ethpb.BeaconBlock{
				Slot:          6,
				ProposerIndex: 13,
				ParentRoot:    bytesutil.PadTo([]byte("hello-world"), 32),
				StateRoot:     bytesutil.PadTo([]byte("state-world"), 32),
				Body:          &ethpb.BeaconBlockBody{},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			hook := logTest.NewGlobal()
			s := setupService(t)
			beaconState, _ := util.DeterministicGenesisState(t, 256)
			var root [32]byte
			copy(root[:], "hello-world")
			wb, err := blocks.NewBeaconBlock(tt.block)
			require.NoError(t, err)
			s.processProposedBlock(beaconState, root, wb)
			if tt.wantedErr != "" {
				require.LogsContain(t, hook, tt.wantedErr)
			} else {
				require.LogsDoNotContain(t, hook, "included")
			}
		})
	}

}

func TestProcessBlock_AllEventsTrackedVals(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()

	genesis, keys := util.DeterministicGenesisStateAltair(t, 64)
	c, err := altair.NextSyncCommittee(ctx, genesis)
	require.NoError(t, err)
	require.NoError(t, genesis.SetCurrentSyncCommittee(c))

	genConfig := util.DefaultBlockGenConfig()
	genConfig.NumProposerSlashings = 1
	genConfig.FullSyncAggregate = true
	b, err := util.GenerateFullBlockAltair(genesis, keys, genConfig, 1)
	require.NoError(t, err)
	s := setupService(t)

	pubKeys := make([][]byte, 3)
	pubKeys[0] = genesis.Validators()[0].PublicKey
	pubKeys[1] = genesis.Validators()[1].PublicKey
	pubKeys[2] = genesis.Validators()[2].PublicKey

	currentSyncCommittee := util.ConvertToCommittee([][]byte{
		pubKeys[0], pubKeys[1], pubKeys[2], pubKeys[1], pubKeys[1],
	})
	require.NoError(t, genesis.SetCurrentSyncCommittee(currentSyncCommittee))

	idx := b.Block.Body.ProposerSlashings[0].Header_1.Header.ProposerIndex
	s.RLock()
	if !s.trackedIndex(idx) {
		s.TrackedValidators[idx] = true
		s.latestPerformance[idx] = ValidatorLatestPerformance{
			balance: 31900000000,
		}
		s.aggregatedPerformance[idx] = ValidatorAggregatedPerformance{}
	}
	s.RUnlock()
	s.updateSyncCommitteeTrackedVals(genesis)

	root, err := b.GetBlock().HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, s.config.StateGen.SaveState(ctx, root, genesis))
	wanted1 := fmt.Sprintf("\"Proposed beacon block was included\" BalanceChange=100000000 BlockRoot=%#x NewBalance=32000000000 ParentRoot=0xf732eaeb7fae ProposerIndex=15 Slot=1 Version=1 prefix=monitor", bytesutil.Trunc(root[:]))
	wanted2 := fmt.Sprintf("\"Proposer slashing was included\" BodyRoot1=0x000100000000 BodyRoot2=0x000200000000 ProposerIndex=%d SlashingSlot=0 Slot=1 prefix=monitor", idx)
	wanted3 := "\"Sync committee contribution included\" BalanceChange=0 ContribCount=3 ExpectedContribCount=3 NewBalance=32000000000 ValidatorIndex=1 prefix=monitor"
	wanted4 := "\"Sync committee contribution included\" BalanceChange=0 ContribCount=1 ExpectedContribCount=1 NewBalance=32000000000 ValidatorIndex=2 prefix=monitor"
	wrapped, err := blocks.NewSignedBeaconBlock(b)
	require.NoError(t, err)
	s.processBlock(ctx, wrapped)
	require.LogsContain(t, hook, wanted1)
	require.LogsContain(t, hook, wanted2)
	require.LogsContain(t, hook, wanted3)
	require.LogsContain(t, hook, wanted4)
}

func TestLogAggregatedPerformance(t *testing.T) {
	hook := logTest.NewGlobal()
	latestPerformance := map[primitives.ValidatorIndex]ValidatorLatestPerformance{
		1: {
			balance: 32000000000,
		},
		2: {
			balance: 32000000000,
		},
		12: {
			balance: 31900000000,
		},
		15: {
			balance: 31900000000,
		},
	}
	aggregatedPerformance := map[primitives.ValidatorIndex]ValidatorAggregatedPerformance{
		1: {
			startEpoch:                      0,
			startBalance:                    31700000000,
			totalAttestedCount:              12,
			totalRequestedCount:             15,
			totalDistance:                   14,
			totalCorrectHead:                8,
			totalCorrectSource:              11,
			totalCorrectTarget:              12,
			totalProposedCount:              1,
			totalSyncCommitteeContributions: 0,
			totalSyncCommitteeAggregations:  0,
		},
		2:  {},
		12: {},
		15: {},
	}
	s := &Service{
		latestPerformance:     latestPerformance,
		aggregatedPerformance: aggregatedPerformance,
	}

	s.logAggregatedPerformance()
	wanted := "\"Aggregated performance since launch\" AttestationInclusion=\"80.00%\"" +
		" AverageInclusionDistance=1.2 BalanceChangePct=\"0.95%\" CorrectlyVotedHeadPct=\"66.67%\" " +
		"CorrectlyVotedSourcePct=\"91.67%\" CorrectlyVotedTargetPct=\"100.00%\" StartBalance=31700000000 " +
		"StartEpoch=0 TotalAggregations=0 TotalProposedBlocks=1 TotalRequested=15 TotalSyncContributions=0 " +
		"ValidatorIndex=1 prefix=monitor"
	require.LogsContain(t, hook, wanted)
}
