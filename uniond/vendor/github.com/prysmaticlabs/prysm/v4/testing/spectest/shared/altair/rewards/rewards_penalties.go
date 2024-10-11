package rewards

import (
	"context"
	"encoding/binary"
	"fmt"
	"path"
	"reflect"
	"strings"
	"testing"

	"github.com/golang/snappy"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
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

// RunPrecomputeRewardsAndPenaltiesTests executes "rewards/{basic, leak, random}" tests.
func RunPrecomputeRewardsAndPenaltiesTests(t *testing.T, config string) {
	require.NoError(t, utils.SetConfig(t, config))

	_, testsFolderPath := utils.TestFolders(t, config, "altair", "rewards")
	testTypes, err := util.BazelListDirectories(testsFolderPath)
	require.NoError(t, err)

	if len(testTypes) == 0 {
		t.Fatalf("No test types found for %s", testsFolderPath)
	}

	for _, testType := range testTypes {
		testFolders, testsFolderPath := utils.TestFolders(t, config, "altair", fmt.Sprintf("rewards/%s/pyspec_tests", testType))
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
	preBeaconStateBase := &ethpb.BeaconStateAltair{}
	require.NoError(t, preBeaconStateBase.UnmarshalSSZ(preBeaconStateSSZ), "Failed to unmarshal")
	preBeaconState, err := state_native.InitializeFromProtoAltair(preBeaconStateBase)
	require.NoError(t, err)

	vp, bp, err := altair.InitializePrecomputeValidators(ctx, preBeaconState)
	require.NoError(t, err)
	vp, bp, err = altair.ProcessEpochParticipation(ctx, preBeaconState, bp, vp)
	require.NoError(t, err)
	deltas, err := altair.AttestationsDelta(preBeaconState, bp, vp)
	require.NoError(t, err)

	rewards := make([]uint64, len(deltas))
	penalties := make([]uint64, len(deltas))
	for i, d := range deltas {
		rewards[i] = d.HeadReward + d.SourceReward + d.TargetReward
		penalties[i] = d.SourcePenalty + d.TargetPenalty + d.InactivityPenalty
	}

	totalSpecTestRewards := make([]uint64, len(rewards))
	totalSpecTestPenalties := make([]uint64, len(penalties))

	// Fetch delta files. i.e. source_deltas.ssz_snappy, etc.
	testfiles, err := util.BazelListFiles(path.Join(testFolderPath))
	require.NoError(t, err)
	deltaFiles := make([]string, 0, len(testfiles))
	for _, tf := range testfiles {
		if strings.Contains(tf, "deltas") {
			deltaFiles = append(deltaFiles, tf)
		}
	}
	if len(deltaFiles) == 0 {
		t.Fatal("No delta files")
	}

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
