package operations

import (
	"context"
	"math/big"
	"path"
	"testing"

	"github.com/golang/snappy"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func RunWithdrawalsTest(t *testing.T, config string) {
	require.NoError(t, utils.SetConfig(t, config))
	testFolders, testsFolderPath := utils.TestFolders(t, config, "capella", "operations/withdrawals/pyspec_tests")
	if len(testFolders) == 0 {
		t.Fatalf("No test folders found for %s/%s/%s", config, "capella", "operations/withdrawals/pyspec_tests")
	}
	for _, folder := range testFolders {
		t.Run(folder.Name(), func(t *testing.T) {
			folderPath := path.Join(testsFolderPath, folder.Name())
			payloadFile, err := util.BazelFileBytes(folderPath, "execution_payload.ssz_snappy")
			require.NoError(t, err)
			payloadSSZ, err := snappy.Decode(nil /* dst */, payloadFile)
			require.NoError(t, err, "Failed to decompress")
			payload := &enginev1.ExecutionPayloadCapella{}
			require.NoError(t, payload.UnmarshalSSZ(payloadSSZ), "Failed to unmarshal")

			body := &ethpb.BeaconBlockBodyCapella{ExecutionPayload: payload}
			RunBlockOperationTest(t, folderPath, body, func(_ context.Context, s state.BeaconState, b interfaces.ReadOnlySignedBeaconBlock) (state.BeaconState, error) {
				payload, err := b.Block().Body().Execution()
				if err != nil {
					return nil, err
				}
				withdrawals, err := payload.Withdrawals()
				if err != nil {
					return nil, err
				}
				p, err := consensusblocks.WrappedExecutionPayloadCapella(&enginev1.ExecutionPayloadCapella{Withdrawals: withdrawals}, big.NewInt(0))
				require.NoError(t, err)
				return blocks.ProcessWithdrawals(s, p)
			})
		})
	}
}
