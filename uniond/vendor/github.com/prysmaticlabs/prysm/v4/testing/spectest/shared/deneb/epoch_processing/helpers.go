package epoch_processing

import (
	"os"
	"path"
	"strings"
	"testing"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/golang/snappy"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"google.golang.org/protobuf/proto"
)

type epochOperation func(*testing.T, state.BeaconState) (state.BeaconState, error)

// RunEpochOperationTest takes in the prestate and processes it through the
// passed in epoch operation function and checks the post state with the expected post state.
func RunEpochOperationTest(
	t *testing.T,
	testFolderPath string,
	operationFn epochOperation,
) {
	preBeaconStateFile, err := util.BazelFileBytes(path.Join(testFolderPath, "pre.ssz_snappy"))
	require.NoError(t, err)
	preBeaconStateSSZ, err := snappy.Decode(nil /* dst */, preBeaconStateFile)
	require.NoError(t, err, "Failed to decompress")
	preBeaconStateBase := &ethpb.BeaconStateDeneb{}
	if err := preBeaconStateBase.UnmarshalSSZ(preBeaconStateSSZ); err != nil {
		t.Fatalf("Failed to unmarshal: %v", err)
	}
	preBeaconState, err := state_native.InitializeFromProtoDeneb(preBeaconStateBase)
	require.NoError(t, err)

	// If the post.ssz is not present, it means the test should fail on our end.
	postSSZFilepath, err := bazel.Runfile(path.Join(testFolderPath, "post.ssz_snappy"))
	postSSZExists := true
	if err != nil && strings.Contains(err.Error(), "could not locate file") {
		postSSZExists = false
	} else if err != nil {
		t.Fatal(err)
	}

	beaconState, err := operationFn(t, preBeaconState)
	if postSSZExists {
		require.NoError(t, err)

		postBeaconStateFile, err := os.ReadFile(postSSZFilepath) // #nosec G304
		require.NoError(t, err)
		postBeaconStateSSZ, err := snappy.Decode(nil /* dst */, postBeaconStateFile)
		require.NoError(t, err, "Failed to decompress")
		postBeaconState := &ethpb.BeaconStateDeneb{}
		if err := postBeaconState.UnmarshalSSZ(postBeaconStateSSZ); err != nil {
			t.Fatalf("Failed to unmarshal: %v", err)
		}

		pbState, err := state_native.ProtobufBeaconStateDeneb(beaconState.ToProtoUnsafe())
		require.NoError(t, err)
		if !proto.Equal(pbState, postBeaconState) {
			t.Fatal("Post state does not match expected")
		}
	} else {
		// Note: This doesn't test anything worthwhile. It essentially tests
		// that *any* error has occurred, not any specific error.
		if err == nil {
			t.Fatal("Did not fail when expected")
		}
		t.Logf("Expected failure; failure reason = %v", err)
		return
	}
}
