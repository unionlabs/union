package operations

import (
	"math/big"
	"os"
	"path"
	"strings"
	"testing"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/golang/snappy"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	blocks2 "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"google.golang.org/protobuf/proto"
)

func RunExecutionPayloadTest(t *testing.T, config string) {
	require.NoError(t, utils.SetConfig(t, config))
	testFolders, testsFolderPath := utils.TestFolders(t, config, "deneb", "operations/execution_payload/pyspec_tests")
	if len(testFolders) == 0 {
		t.Fatalf("No test folders found for %s/%s/%s", config, "deneb", "operations/execution_payload/pyspec_tests")
	}
	for _, folder := range testFolders {
		t.Run(folder.Name(), func(t *testing.T) {
			helpers.ClearCache()

			blockBodyFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), "body.ssz_snappy")
			require.NoError(t, err)
			blockSSZ, err := snappy.Decode(nil /* dst */, blockBodyFile)
			require.NoError(t, err, "Failed to decompress")
			body := &ethpb.BeaconBlockBodyDeneb{}
			require.NoError(t, body.UnmarshalSSZ(blockSSZ), "Failed to unmarshal")
			b, err := blocks2.NewBeaconBlock(&ethpb.BeaconBlockDeneb{Body: body})
			require.NoError(t, err)

			preBeaconStateFile, err := util.BazelFileBytes(testsFolderPath, folder.Name(), "pre.ssz_snappy")
			require.NoError(t, err)
			preBeaconStateSSZ, err := snappy.Decode(nil /* dst */, preBeaconStateFile)
			require.NoError(t, err, "Failed to decompress")
			preBeaconStateBase := &ethpb.BeaconStateDeneb{}
			require.NoError(t, preBeaconStateBase.UnmarshalSSZ(preBeaconStateSSZ), "Failed to unmarshal")
			preBeaconState, err := state_native.InitializeFromProtoDeneb(preBeaconStateBase)
			require.NoError(t, err)

			postSSZFilepath, err := bazel.Runfile(path.Join(testsFolderPath, folder.Name(), "post.ssz_snappy"))
			postSSZExists := true
			if err != nil && strings.Contains(err.Error(), "could not locate file") {
				postSSZExists = false
			} else {
				require.NoError(t, err)
			}

			payload, err := blocks2.WrappedExecutionPayloadDeneb(body.ExecutionPayload, big.NewInt(0))
			require.NoError(t, err)

			file, err := util.BazelFileBytes(testsFolderPath, folder.Name(), "execution.yaml")
			require.NoError(t, err)
			config := &ExecutionConfig{}
			require.NoError(t, utils.UnmarshalYaml(file, config), "Failed to Unmarshal")

			if postSSZExists {
				require.NoError(t, blocks.ValidatePayloadWhenMergeCompletes(preBeaconState, payload))
				require.NoError(t, blocks.ValidatePayload(preBeaconState, payload))
				require.NoError(t, transition.VerifyBlobCommitmentCount(b))
				require.NoError(t, preBeaconState.SetLatestExecutionPayloadHeader(payload))
				postBeaconStateFile, err := os.ReadFile(postSSZFilepath) // #nosec G304
				require.NoError(t, err)
				postBeaconStateSSZ, err := snappy.Decode(nil /* dst */, postBeaconStateFile)
				require.NoError(t, err, "Failed to decompress")

				postBeaconState := &ethpb.BeaconStateDeneb{}
				require.NoError(t, postBeaconState.UnmarshalSSZ(postBeaconStateSSZ), "Failed to unmarshal")
				pbState, err := state_native.ProtobufBeaconStateDeneb(preBeaconState.ToProto())
				require.NoError(t, err)
				t.Log(pbState)
				t.Log(postBeaconState)
				if !proto.Equal(pbState, postBeaconState) {
					t.Fatal("Post state does not match expected")
				}
			} else if config.Valid {
				err1 := blocks.ValidatePayloadWhenMergeCompletes(preBeaconState, payload)
				err2 := blocks.ValidatePayload(preBeaconState, payload)
				err3 := transition.VerifyBlobCommitmentCount(b)
				// Note: This doesn't test anything worthwhile. It essentially tests
				// that *any* error has occurred, not any specific error.
				if err1 == nil && err2 == nil && err3 == nil {
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
