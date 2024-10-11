package operations

import (
	"os"
	"path"
	"strings"
	"testing"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/golang/snappy"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	blocks2 "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"google.golang.org/protobuf/proto"
	"gopkg.in/d4l3k/messagediff.v1"
)

func RunExecutionPayloadTest(t *testing.T, config string) {
	require.NoError(t, utils.SetConfig(t, config))
	testFolders, testsFolderPath := utils.TestFolders(t, config, "bellatrix", "operations/execution_payload/pyspec_tests")
	if len(testFolders) == 0 {
		t.Fatalf("No test folders found for %s/%s/%s", config, "bellatrix", "operations/execution_payload/pyspec_tests")
	}
	for _, folder := range testFolders {
		t.Run(folder.Name(), func(t *testing.T) {
			helpers.ClearCache()

			blockBodyFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), "body.ssz_snappy")
			require.NoError(t, err)
			blockSSZ, err := snappy.Decode(nil /* dst */, blockBodyFile)
			require.NoError(t, err, "Failed to decompress")
			block := &ethpb.BeaconBlockBodyBellatrix{}
			require.NoError(t, block.UnmarshalSSZ(blockSSZ), "Failed to unmarshal")

			preBeaconStateFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), "pre.ssz_snappy")
			require.NoError(t, err)
			preBeaconStateSSZ, err := snappy.Decode(nil /* dst */, preBeaconStateFile)
			require.NoError(t, err, "Failed to decompress")
			preBeaconStateBase := &ethpb.BeaconStateBellatrix{}
			require.NoError(t, preBeaconStateBase.UnmarshalSSZ(preBeaconStateSSZ), "Failed to unmarshal")
			preBeaconState, err := state_native.InitializeFromProtoBellatrix(preBeaconStateBase)
			require.NoError(t, err)

			postSSZFilepath, err := bazel.Runfile(path.Join(testsFolderPath, folder.Name(), "post.ssz_snappy"))
			postSSZExists := true
			if err != nil && strings.Contains(err.Error(), "could not locate file") {
				postSSZExists = false
			} else {
				require.NoError(t, err)
			}

			payload, err := blocks2.WrappedExecutionPayload(block.ExecutionPayload)
			require.NoError(t, err)

			file, err := util.BazelFileBytes(testsFolderPath, folder.Name(), "execution.yaml")
			require.NoError(t, err)
			config := &ExecutionConfig{}
			require.NoError(t, utils.UnmarshalYaml(file, config), "Failed to Unmarshal")

			gotState, err := blocks.ProcessPayload(preBeaconState, payload)
			if postSSZExists {
				require.NoError(t, err)

				postBeaconStateFile, err := os.ReadFile(postSSZFilepath) // #nosec G304
				require.NoError(t, err)
				postBeaconStateSSZ, err := snappy.Decode(nil /* dst */, postBeaconStateFile)
				require.NoError(t, err, "Failed to decompress")

				postBeaconState := &ethpb.BeaconStateBellatrix{}
				require.NoError(t, postBeaconState.UnmarshalSSZ(postBeaconStateSSZ), "Failed to unmarshal")
				pbState, err := state_native.ProtobufBeaconStateBellatrix(gotState.ToProto())
				require.NoError(t, err)
				if !proto.Equal(pbState, postBeaconState) {
					diff, _ := messagediff.PrettyDiff(gotState.ToProto(), postBeaconState)
					t.Log(diff)
					t.Fatal("Post state does not match expected")
				}
			} else if config.Valid {
				// Note: This doesn't test anything worthwhile. It essentially tests
				// that *any* error has occurred, not any specific error.
				if err == nil {
					t.Fatal("Did not fail when expected")
				}
				t.Logf("Expected failure; failure reason = %v", err)
				return
			}
		})
	}
}

type ExecutionConfig struct {
	Valid bool `json:"execution_valid"`
}
