package operations

import (
	"context"
	"os"
	"path"
	"strings"
	"testing"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/golang/snappy"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"google.golang.org/protobuf/proto"
	"gopkg.in/d4l3k/messagediff.v1"
)

type blockOperation func(context.Context, state.BeaconState, interfaces.ReadOnlySignedBeaconBlock) (state.BeaconState, error)

// RunBlockOperationTest takes in the prestate and the beacon block body, processes it through the
// passed in block operation function and checks the post state with the expected post state.
func RunBlockOperationTest(
	t *testing.T,
	folderPath string,
	body *ethpb.BeaconBlockBodyBellatrix,
	operationFn blockOperation,
) {
	preBeaconStateFile, err := util.BazelFileBytes(path.Join(folderPath, "pre.ssz_snappy"))
	require.NoError(t, err)
	preBeaconStateSSZ, err := snappy.Decode(nil /* dst */, preBeaconStateFile)
	require.NoError(t, err, "Failed to decompress")
	preStateBase := &ethpb.BeaconStateBellatrix{}
	if err := preStateBase.UnmarshalSSZ(preBeaconStateSSZ); err != nil {
		t.Fatalf("Failed to unmarshal: %v", err)
	}
	preState, err := state_native.InitializeFromProtoBellatrix(preStateBase)
	require.NoError(t, err)

	// If the post.ssz is not present, it means the test should fail on our end.
	postSSZFilepath, err := bazel.Runfile(path.Join(folderPath, "post.ssz_snappy"))
	postSSZExists := true
	if err != nil && strings.Contains(err.Error(), "could not locate file") {
		postSSZExists = false
	} else if err != nil {
		t.Fatal(err)
	}

	helpers.ClearCache()
	b := util.NewBeaconBlockBellatrix()
	b.Block.Body = body
	wsb, err := blocks.NewSignedBeaconBlock(b)
	require.NoError(t, err)
	beaconState, err := operationFn(context.Background(), preState, wsb)
	if postSSZExists {
		require.NoError(t, err)

		postBeaconStateFile, err := os.ReadFile(postSSZFilepath) // #nosec G304
		require.NoError(t, err)
		postBeaconStateSSZ, err := snappy.Decode(nil /* dst */, postBeaconStateFile)
		require.NoError(t, err, "Failed to decompress")

		postBeaconState := &ethpb.BeaconStateBellatrix{}
		if err := postBeaconState.UnmarshalSSZ(postBeaconStateSSZ); err != nil {
			t.Fatalf("Failed to unmarshal: %v", err)
		}
		pbState, err := state_native.ProtobufBeaconStateBellatrix(beaconState.ToProtoUnsafe())
		require.NoError(t, err)
		if !proto.Equal(pbState, postBeaconState) {
			diff, _ := messagediff.PrettyDiff(beaconState.ToProtoUnsafe(), postBeaconState)
			t.Log(diff)
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
