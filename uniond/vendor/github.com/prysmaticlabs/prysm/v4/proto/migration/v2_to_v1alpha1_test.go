package migration

import (
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpbv1 "github.com/prysmaticlabs/prysm/v4/proto/eth/v1"
	ethpbv2 "github.com/prysmaticlabs/prysm/v4/proto/eth/v2"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func Test_AltairToV1Alpha1SignedBlock(t *testing.T) {
	v2Block := util.HydrateV2AltairSignedBeaconBlock(&ethpbv2.SignedBeaconBlockAltair{})
	v2Block.Message.Slot = slot
	v2Block.Message.ProposerIndex = validatorIndex
	v2Block.Message.ParentRoot = parentRoot
	v2Block.Message.StateRoot = stateRoot
	v2Block.Message.Body.RandaoReveal = randaoReveal
	v2Block.Message.Body.Eth1Data = &ethpbv1.Eth1Data{
		DepositRoot:  depositRoot,
		DepositCount: depositCount,
		BlockHash:    blockHash,
	}
	syncCommitteeBits := bitfield.NewBitvector512()
	syncCommitteeBits.SetBitAt(100, true)
	v2Block.Message.Body.SyncAggregate = &ethpbv1.SyncAggregate{
		SyncCommitteeBits:      syncCommitteeBits,
		SyncCommitteeSignature: signature,
	}
	v2Block.Signature = signature

	alphaBlock, err := AltairToV1Alpha1SignedBlock(v2Block)
	require.NoError(t, err)
	alphaRoot, err := alphaBlock.HashTreeRoot()
	require.NoError(t, err)
	v2Root, err := v2Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, v2Root, alphaRoot)
}

func Test_BellatrixToV1Alpha1SignedBlock(t *testing.T) {
	v2Block := util.HydrateV2BellatrixSignedBeaconBlock(&ethpbv2.SignedBeaconBlockBellatrix{})
	v2Block.Message.Slot = slot
	v2Block.Message.ProposerIndex = validatorIndex
	v2Block.Message.ParentRoot = parentRoot
	v2Block.Message.StateRoot = stateRoot
	v2Block.Message.Body.RandaoReveal = randaoReveal
	v2Block.Message.Body.Eth1Data = &ethpbv1.Eth1Data{
		DepositRoot:  depositRoot,
		DepositCount: depositCount,
		BlockHash:    blockHash,
	}
	syncCommitteeBits := bitfield.NewBitvector512()
	syncCommitteeBits.SetBitAt(100, true)
	v2Block.Message.Body.SyncAggregate = &ethpbv1.SyncAggregate{
		SyncCommitteeBits:      syncCommitteeBits,
		SyncCommitteeSignature: signature,
	}
	v2Block.Message.Body.ExecutionPayload = &enginev1.ExecutionPayload{
		ParentHash:    parentHash,
		FeeRecipient:  feeRecipient,
		StateRoot:     stateRoot,
		ReceiptsRoot:  receiptsRoot,
		LogsBloom:     logsBloom,
		PrevRandao:    prevRandao,
		BlockNumber:   blockNumber,
		GasLimit:      gasLimit,
		GasUsed:       gasUsed,
		Timestamp:     timestamp,
		ExtraData:     extraData,
		BaseFeePerGas: baseFeePerGas,
		BlockHash:     blockHash,
		Transactions:  [][]byte{[]byte("transaction1"), []byte("transaction2")},
	}
	v2Block.Signature = signature

	alphaBlock, err := BellatrixToV1Alpha1SignedBlock(v2Block)
	require.NoError(t, err)
	alphaRoot, err := alphaBlock.HashTreeRoot()
	require.NoError(t, err)
	v2Root, err := v2Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, v2Root, alphaRoot)
}

func Test_BlindedBellatrixToV1Alpha1SignedBlock(t *testing.T) {
	v2Block := util.HydrateV2SignedBlindedBeaconBlockBellatrix(&ethpbv2.SignedBlindedBeaconBlockBellatrix{})
	v2Block.Message.Slot = slot
	v2Block.Message.ProposerIndex = validatorIndex
	v2Block.Message.ParentRoot = parentRoot
	v2Block.Message.StateRoot = stateRoot
	v2Block.Message.Body.RandaoReveal = randaoReveal
	v2Block.Message.Body.Eth1Data = &ethpbv1.Eth1Data{
		DepositRoot:  depositRoot,
		DepositCount: depositCount,
		BlockHash:    blockHash,
	}
	syncCommitteeBits := bitfield.NewBitvector512()
	syncCommitteeBits.SetBitAt(100, true)
	v2Block.Message.Body.SyncAggregate = &ethpbv1.SyncAggregate{
		SyncCommitteeBits:      syncCommitteeBits,
		SyncCommitteeSignature: signature,
	}
	v2Block.Message.Body.ExecutionPayloadHeader = &enginev1.ExecutionPayloadHeader{
		ParentHash:       parentHash,
		FeeRecipient:     feeRecipient,
		StateRoot:        stateRoot,
		ReceiptsRoot:     receiptsRoot,
		LogsBloom:        logsBloom,
		PrevRandao:       prevRandao,
		BlockNumber:      blockNumber,
		GasLimit:         gasLimit,
		GasUsed:          gasUsed,
		Timestamp:        timestamp,
		ExtraData:        extraData,
		BaseFeePerGas:    baseFeePerGas,
		BlockHash:        blockHash,
		TransactionsRoot: transactionsRoot,
	}
	v2Block.Signature = signature

	alphaBlock, err := BlindedBellatrixToV1Alpha1SignedBlock(v2Block)
	require.NoError(t, err)
	alphaRoot, err := alphaBlock.HashTreeRoot()
	require.NoError(t, err)
	v2Root, err := v2Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, v2Root, alphaRoot)
}
