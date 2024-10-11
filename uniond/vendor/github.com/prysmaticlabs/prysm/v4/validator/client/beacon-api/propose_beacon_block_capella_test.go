package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
	test_helpers "github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/test-helpers"
)

func TestProposeBeaconBlock_Capella(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	capellaBlock := generateSignedCapellaBlock()

	genericSignedBlock := &ethpb.GenericSignedBeaconBlock{}
	genericSignedBlock.Block = capellaBlock

	jsonCapellaBlock := &shared.SignedBeaconBlockCapella{
		Signature: hexutil.Encode(capellaBlock.Capella.Signature),
		Message: &shared.BeaconBlockCapella{
			ParentRoot:    hexutil.Encode(capellaBlock.Capella.Block.ParentRoot),
			ProposerIndex: uint64ToString(capellaBlock.Capella.Block.ProposerIndex),
			Slot:          uint64ToString(capellaBlock.Capella.Block.Slot),
			StateRoot:     hexutil.Encode(capellaBlock.Capella.Block.StateRoot),
			Body: &shared.BeaconBlockBodyCapella{
				Attestations:      jsonifyAttestations(capellaBlock.Capella.Block.Body.Attestations),
				AttesterSlashings: jsonifyAttesterSlashings(capellaBlock.Capella.Block.Body.AttesterSlashings),
				Deposits:          jsonifyDeposits(capellaBlock.Capella.Block.Body.Deposits),
				Eth1Data:          jsonifyEth1Data(capellaBlock.Capella.Block.Body.Eth1Data),
				Graffiti:          hexutil.Encode(capellaBlock.Capella.Block.Body.Graffiti),
				ProposerSlashings: jsonifyProposerSlashings(capellaBlock.Capella.Block.Body.ProposerSlashings),
				RandaoReveal:      hexutil.Encode(capellaBlock.Capella.Block.Body.RandaoReveal),
				VoluntaryExits:    JsonifySignedVoluntaryExits(capellaBlock.Capella.Block.Body.VoluntaryExits),
				SyncAggregate: &shared.SyncAggregate{
					SyncCommitteeBits:      hexutil.Encode(capellaBlock.Capella.Block.Body.SyncAggregate.SyncCommitteeBits),
					SyncCommitteeSignature: hexutil.Encode(capellaBlock.Capella.Block.Body.SyncAggregate.SyncCommitteeSignature),
				},
				ExecutionPayload: &shared.ExecutionPayloadCapella{
					BaseFeePerGas: bytesutil.LittleEndianBytesToBigInt(capellaBlock.Capella.Block.Body.ExecutionPayload.BaseFeePerGas).String(),
					BlockHash:     hexutil.Encode(capellaBlock.Capella.Block.Body.ExecutionPayload.BlockHash),
					BlockNumber:   uint64ToString(capellaBlock.Capella.Block.Body.ExecutionPayload.BlockNumber),
					ExtraData:     hexutil.Encode(capellaBlock.Capella.Block.Body.ExecutionPayload.ExtraData),
					FeeRecipient:  hexutil.Encode(capellaBlock.Capella.Block.Body.ExecutionPayload.FeeRecipient),
					GasLimit:      uint64ToString(capellaBlock.Capella.Block.Body.ExecutionPayload.GasLimit),
					GasUsed:       uint64ToString(capellaBlock.Capella.Block.Body.ExecutionPayload.GasUsed),
					LogsBloom:     hexutil.Encode(capellaBlock.Capella.Block.Body.ExecutionPayload.LogsBloom),
					ParentHash:    hexutil.Encode(capellaBlock.Capella.Block.Body.ExecutionPayload.ParentHash),
					PrevRandao:    hexutil.Encode(capellaBlock.Capella.Block.Body.ExecutionPayload.PrevRandao),
					ReceiptsRoot:  hexutil.Encode(capellaBlock.Capella.Block.Body.ExecutionPayload.ReceiptsRoot),
					StateRoot:     hexutil.Encode(capellaBlock.Capella.Block.Body.ExecutionPayload.StateRoot),
					Timestamp:     uint64ToString(capellaBlock.Capella.Block.Body.ExecutionPayload.Timestamp),
					Transactions:  jsonifyTransactions(capellaBlock.Capella.Block.Body.ExecutionPayload.Transactions),
					Withdrawals:   jsonifyWithdrawals(capellaBlock.Capella.Block.Body.ExecutionPayload.Withdrawals),
				},
				BLSToExecutionChanges: jsonifyBlsToExecutionChanges(capellaBlock.Capella.Block.Body.BlsToExecutionChanges),
			},
		},
	}

	marshalledBlock, err := json.Marshal(jsonCapellaBlock)
	require.NoError(t, err)

	// Make sure that what we send in the POST body is the marshalled version of the protobuf block
	headers := map[string]string{"Eth-Consensus-Version": "capella"}
	jsonRestHandler.EXPECT().Post(
		context.Background(),
		"/eth/v1/beacon/blocks",
		headers,
		bytes.NewBuffer(marshalledBlock),
		nil,
	)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	proposeResponse, err := validatorClient.proposeBeaconBlock(context.Background(), genericSignedBlock)
	assert.NoError(t, err)
	require.NotNil(t, proposeResponse)

	expectedBlockRoot, err := capellaBlock.Capella.Block.HashTreeRoot()
	require.NoError(t, err)

	// Make sure that the block root is set
	assert.DeepEqual(t, expectedBlockRoot[:], proposeResponse.BlockRoot)
}

func generateSignedCapellaBlock() *ethpb.GenericSignedBeaconBlock_Capella {
	return &ethpb.GenericSignedBeaconBlock_Capella{
		Capella: &ethpb.SignedBeaconBlockCapella{
			Block:     test_helpers.GenerateProtoCapellaBeaconBlock(),
			Signature: test_helpers.FillByteSlice(96, 127),
		},
	}
}
