package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
	test_helpers "github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/test-helpers"
)

func TestProposeBeaconBlock_Altair(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	altairBlock := generateSignedAltairBlock()

	genericSignedBlock := &ethpb.GenericSignedBeaconBlock{}
	genericSignedBlock.Block = altairBlock

	jsonAltairBlock := &shared.SignedBeaconBlockAltair{
		Signature: hexutil.Encode(altairBlock.Altair.Signature),
		Message: &shared.BeaconBlockAltair{
			ParentRoot:    hexutil.Encode(altairBlock.Altair.Block.ParentRoot),
			ProposerIndex: uint64ToString(altairBlock.Altair.Block.ProposerIndex),
			Slot:          uint64ToString(altairBlock.Altair.Block.Slot),
			StateRoot:     hexutil.Encode(altairBlock.Altair.Block.StateRoot),
			Body: &shared.BeaconBlockBodyAltair{
				Attestations:      jsonifyAttestations(altairBlock.Altair.Block.Body.Attestations),
				AttesterSlashings: jsonifyAttesterSlashings(altairBlock.Altair.Block.Body.AttesterSlashings),
				Deposits:          jsonifyDeposits(altairBlock.Altair.Block.Body.Deposits),
				Eth1Data:          jsonifyEth1Data(altairBlock.Altair.Block.Body.Eth1Data),
				Graffiti:          hexutil.Encode(altairBlock.Altair.Block.Body.Graffiti),
				ProposerSlashings: jsonifyProposerSlashings(altairBlock.Altair.Block.Body.ProposerSlashings),
				RandaoReveal:      hexutil.Encode(altairBlock.Altair.Block.Body.RandaoReveal),
				VoluntaryExits:    JsonifySignedVoluntaryExits(altairBlock.Altair.Block.Body.VoluntaryExits),
				SyncAggregate: &shared.SyncAggregate{
					SyncCommitteeBits:      hexutil.Encode(altairBlock.Altair.Block.Body.SyncAggregate.SyncCommitteeBits),
					SyncCommitteeSignature: hexutil.Encode(altairBlock.Altair.Block.Body.SyncAggregate.SyncCommitteeSignature),
				},
			},
		},
	}

	marshalledBlock, err := json.Marshal(jsonAltairBlock)
	require.NoError(t, err)

	ctx := context.Background()

	// Make sure that what we send in the POST body is the marshalled version of the protobuf block
	headers := map[string]string{"Eth-Consensus-Version": "altair"}
	jsonRestHandler.EXPECT().Post(
		ctx,
		"/eth/v1/beacon/blocks",
		headers,
		bytes.NewBuffer(marshalledBlock),
		nil,
	)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	proposeResponse, err := validatorClient.proposeBeaconBlock(ctx, genericSignedBlock)
	assert.NoError(t, err)
	require.NotNil(t, proposeResponse)

	expectedBlockRoot, err := altairBlock.Altair.Block.HashTreeRoot()
	require.NoError(t, err)

	// Make sure that the block root is set
	assert.DeepEqual(t, expectedBlockRoot[:], proposeResponse.BlockRoot)
}

func generateSignedAltairBlock() *ethpb.GenericSignedBeaconBlock_Altair {
	return &ethpb.GenericSignedBeaconBlock_Altair{
		Altair: &ethpb.SignedBeaconBlockAltair{
			Block:     test_helpers.GenerateProtoAltairBeaconBlock(),
			Signature: test_helpers.FillByteSlice(96, 112),
		},
	}
}
