package sanity

import (
	"context"
	"fmt"
	"os"
	"path"
	"strings"
	"testing"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/d4l3k/messagediff"
	"github.com/golang/snappy"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"google.golang.org/protobuf/proto"
)

func init() {
	transition.SkipSlotCache.Disable()
}

// RunBlockProcessingTest executes "sanity/blocks" tests.
func RunBlockProcessingTest(t *testing.T, config, folderPath string) {
	require.NoError(t, utils.SetConfig(t, config))

	testFolders, testsFolderPath := utils.TestFolders(t, config, "altair", folderPath)
	if len(testFolders) == 0 {
		t.Fatalf("No test folders found for %s/%s/%s", config, "altair", folderPath)
	}
	for _, folder := range testFolders {
		t.Run(folder.Name(), func(t *testing.T) {
			helpers.ClearCache()
			preBeaconStateFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), "pre.ssz_snappy")
			require.NoError(t, err)
			preBeaconStateSSZ, err := snappy.Decode(nil /* dst */, preBeaconStateFile)
			require.NoError(t, err, "Failed to decompress")
			beaconStateBase := &ethpb.BeaconStateAltair{}
			require.NoError(t, beaconStateBase.UnmarshalSSZ(preBeaconStateSSZ), "Failed to unmarshal")
			beaconState, err := state_native.InitializeFromProtoAltair(beaconStateBase)
			require.NoError(t, err)

			file, err := util.BazelFileBytes(testsFolderPath, folder.Name(), "meta.yaml")
			require.NoError(t, err)

			metaYaml := &Config{}
			require.NoError(t, utils.UnmarshalYaml(file, metaYaml), "Failed to Unmarshal")

			var transitionError error
			var processedState state.BeaconState
			var ok bool
			for i := 0; i < metaYaml.BlocksCount; i++ {
				filename := fmt.Sprintf("blocks_%d.ssz_snappy", i)
				blockFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), filename)
				require.NoError(t, err)
				blockSSZ, err := snappy.Decode(nil /* dst */, blockFile)
				require.NoError(t, err, "Failed to decompress")
				block := &ethpb.SignedBeaconBlockAltair{}
				require.NoError(t, block.UnmarshalSSZ(blockSSZ), "Failed to unmarshal")
				wsb, err := blocks.NewSignedBeaconBlock(block)
				require.NoError(t, err)
				processedState, transitionError = transition.ExecuteStateTransition(context.Background(), beaconState, wsb)
				if transitionError != nil {
					break
				}
				beaconState, ok = processedState.(*state_native.BeaconState)
				require.Equal(t, true, ok)
			}

			// If the post.ssz is not present, it means the test should fail on our end.
			postSSZFilepath, readError := bazel.Runfile(path.Join(testsFolderPath, folder.Name(), "post.ssz_snappy"))
			postSSZExists := true
			if readError != nil && strings.Contains(readError.Error(), "could not locate file") {
				postSSZExists = false
			} else if readError != nil {
				t.Fatal(readError)
			}

			if postSSZExists {
				if transitionError != nil {
					t.Errorf("Unexpected error: %v", transitionError)
				}

				postBeaconStateFile, err := os.ReadFile(postSSZFilepath) // #nosec G304
				require.NoError(t, err)
				postBeaconStateSSZ, err := snappy.Decode(nil /* dst */, postBeaconStateFile)
				require.NoError(t, err, "Failed to decompress")

				postBeaconState := &ethpb.BeaconStateAltair{}
				require.NoError(t, postBeaconState.UnmarshalSSZ(postBeaconStateSSZ), "Failed to unmarshal")
				pbState, err := state_native.ProtobufBeaconStateAltair(beaconState.ToProtoUnsafe())
				require.NoError(t, err)
				if !proto.Equal(pbState, postBeaconState) {
					diff, _ := messagediff.PrettyDiff(beaconState.ToProtoUnsafe(), postBeaconState)
					t.Log(diff)
					t.Fatal("Post state does not match expected")
				}
			} else {
				// Note: This doesn't test anything worthwhile. It essentially tests
				// that *any* error has occurred, not any specific error.
				if transitionError == nil {
					t.Fatal("Did not fail when expected")
				}
				t.Logf("Expected failure; failure reason = %v", transitionError)
				return
			}
		})
	}
}
