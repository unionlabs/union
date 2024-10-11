package rewards

import (
	"context"
	"encoding/binary"
	"fmt"
	"path"
	"reflect"
	"testing"

	"github.com/golang/snappy"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/epoch/precompute"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

// Delta contains list of rewards and penalties.
type Delta struct {
	Rewards   []uint64 `json:"rewards"`
	Penalties []uint64 `json:"penalties"`
}

// unmarshalSSZ deserializes specs data into a simple aggregating container.
func (d *Delta) unmarshalSSZ(buf []byte) error {
	offset1 := binary.LittleEndian.Uint32(buf[:4])
	offset2 := binary.LittleEndian.Uint32(buf[4:8])

	for i := uint32(0); i < offset2-offset1; i += 8 {
		d.Rewards = append(d.Rewards, binary.LittleEndian.Uint64(buf[offset1+i:offset1+i+8]))
		d.Penalties = append(d.Penalties, binary.LittleEndian.Uint64(buf[offset2+i:offset2+i+8]))
	}
	return nil
}

var deltaFiles = []string{
	"source_deltas.ssz_snappy",
	"target_deltas.ssz_snappy",
	"head_deltas.ssz_snappy",
	"inactivity_penalty_deltas.ssz_snappy",
	"inclusion_delay_deltas.ssz_snappy",
}

// RunPrecomputeRewardsAndPenaltiesTests executes "rewards/{basic, leak, random}" tests.
func RunPrecomputeRewardsAndPenaltiesTests(t *testing.T, config string) {
	require.NoError(t, utils.SetConfig(t, config))

	_, testsFolderPath := utils.TestFolders(t, config, "phase0", "rewards")
	testTypes, err := util.BazelListDirectories(testsFolderPath)
	require.NoError(t, err)
	if len(testTypes) == 0 {
		t.Fatalf("No test types found for %s/%s/%s", config, "phase0", "rewards")
	}

	for _, testType := range testTypes {
		testPath := fmt.Sprintf("rewards/%s/pyspec_tests", testType)
		testFolders, testsFolderPath := utils.TestFolders(t, config, "phase0", testPath)
		if len(testFolders) == 0 {
			t.Fatalf("No test folders found for %s/%s/%s", config, "phase0", testPath)
		}
		for _, folder := range testFolders {
			helpers.ClearCache()
			t.Run(fmt.Sprintf("%v/%v", testType, folder.Name()), func(t *testing.T) {
				folderPath := path.Join(testsFolderPath, folder.Name())
				runPrecomputeRewardsAndPenaltiesTest(t, folderPath)
			})
		}
	}
}

func runPrecomputeRewardsAndPenaltiesTest(t *testing.T, testFolderPath string) {
	ctx := context.Background()
	preBeaconStateFile, err := util.BazelFileBytes(path.Join(testFolderPath, "pre.ssz_snappy"))
	require.NoError(t, err)
	preBeaconStateSSZ, err := snappy.Decode(nil /* dst */, preBeaconStateFile)
	require.NoError(t, err, "Failed to decompress")
	preBeaconStateBase := &ethpb.BeaconState{}
	require.NoError(t, preBeaconStateBase.UnmarshalSSZ(preBeaconStateSSZ), "Failed to unmarshal")
	preBeaconState, err := state_native.InitializeFromProtoPhase0(preBeaconStateBase)
	require.NoError(t, err)

	vp, bp, err := precompute.New(ctx, preBeaconState)
	require.NoError(t, err)
	vp, bp, err = precompute.ProcessAttestations(ctx, preBeaconState, vp, bp)
	require.NoError(t, err)

	rewards, penalties, err := precompute.AttestationsDelta(preBeaconState, bp, vp)
	require.NoError(t, err)
	pRewards, err := precompute.ProposersDelta(preBeaconState, bp, vp)
	require.NoError(t, err)
	if len(rewards) != len(penalties) && len(rewards) != len(pRewards) {
		t.Fatal("Incorrect lengths")
	}
	for i, reward := range rewards {
		rewards[i] = reward + pRewards[i]
	}

	totalSpecTestRewards := make([]uint64, len(rewards))
	totalSpecTestPenalties := make([]uint64, len(penalties))

	for _, dFile := range deltaFiles {
		sourceFile, err := util.BazelFileBytes(path.Join(testFolderPath, dFile))
		require.NoError(t, err)
		sourceSSZ, err := snappy.Decode(nil /* dst */, sourceFile)
		require.NoError(t, err, "Failed to decompress")
		d := &Delta{}
		require.NoError(t, d.unmarshalSSZ(sourceSSZ), "Failed to unmarshal")
		for i, reward := range d.Rewards {
			totalSpecTestRewards[i] += reward
		}
		for i, penalty := range d.Penalties {
			totalSpecTestPenalties[i] += penalty
		}
	}

	if !reflect.DeepEqual(rewards, totalSpecTestRewards) {
		t.Error("Rewards don't match")
		t.Log(rewards)
		t.Log(totalSpecTestRewards)
	}
	if !reflect.DeepEqual(penalties, totalSpecTestPenalties) {
		t.Error("Penalties don't match")
		t.Log(penalties)
		t.Log(totalSpecTestPenalties)
	}
}
