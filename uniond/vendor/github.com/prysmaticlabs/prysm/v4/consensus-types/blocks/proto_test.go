package blocks

import (
	"math/big"
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

type fields struct {
	root                     [32]byte
	sig                      [96]byte
	deposits                 []*eth.Deposit
	atts                     []*eth.Attestation
	proposerSlashings        []*eth.ProposerSlashing
	attesterSlashings        []*eth.AttesterSlashing
	voluntaryExits           []*eth.SignedVoluntaryExit
	syncAggregate            *eth.SyncAggregate
	execPayload              *enginev1.ExecutionPayload
	execPayloadHeader        *enginev1.ExecutionPayloadHeader
	execPayloadCapella       *enginev1.ExecutionPayloadCapella
	execPayloadHeaderCapella *enginev1.ExecutionPayloadHeaderCapella
	execPayloadDeneb         *enginev1.ExecutionPayloadDeneb
	execPayloadHeaderDeneb   *enginev1.ExecutionPayloadHeaderDeneb
	blsToExecutionChanges    []*eth.SignedBLSToExecutionChange
	kzgCommitments           [][]byte
}

func Test_SignedBeaconBlock_Proto(t *testing.T) {
	f := getFields()

	t.Run("Phase0", func(t *testing.T) {
		expectedBlock := &eth.SignedBeaconBlock{
			Block: &eth.BeaconBlock{
				Slot:          128,
				ProposerIndex: 128,
				ParentRoot:    f.root[:],
				StateRoot:     f.root[:],
				Body:          bodyPbPhase0(),
			},
			Signature: f.sig[:],
		}
		block := &SignedBeaconBlock{
			version: version.Phase0,
			block: &BeaconBlock{
				version:       version.Phase0,
				slot:          128,
				proposerIndex: 128,
				parentRoot:    f.root,
				stateRoot:     f.root,
				body:          bodyPhase0(),
			},
			signature: f.sig,
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.SignedBeaconBlock)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("Altair", func(t *testing.T) {
		expectedBlock := &eth.SignedBeaconBlockAltair{
			Block: &eth.BeaconBlockAltair{
				Slot:          128,
				ProposerIndex: 128,
				ParentRoot:    f.root[:],
				StateRoot:     f.root[:],
				Body:          bodyPbAltair(),
			},
			Signature: f.sig[:],
		}
		block := &SignedBeaconBlock{
			version: version.Altair,
			block: &BeaconBlock{
				version:       version.Altair,
				slot:          128,
				proposerIndex: 128,
				parentRoot:    f.root,
				stateRoot:     f.root,
				body:          bodyAltair(),
			},
			signature: f.sig,
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.SignedBeaconBlockAltair)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("Bellatrix", func(t *testing.T) {
		expectedBlock := &eth.SignedBeaconBlockBellatrix{
			Block: &eth.BeaconBlockBellatrix{
				Slot:          128,
				ProposerIndex: 128,
				ParentRoot:    f.root[:],
				StateRoot:     f.root[:],
				Body:          bodyPbBellatrix(),
			},
			Signature: f.sig[:],
		}
		block := &SignedBeaconBlock{
			version: version.Bellatrix,
			block: &BeaconBlock{
				version:       version.Bellatrix,
				slot:          128,
				proposerIndex: 128,
				parentRoot:    f.root,
				stateRoot:     f.root,
				body:          bodyBellatrix(t),
			},
			signature: f.sig,
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.SignedBeaconBlockBellatrix)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("BellatrixBlind", func(t *testing.T) {
		expectedBlock := &eth.SignedBlindedBeaconBlockBellatrix{
			Block: &eth.BlindedBeaconBlockBellatrix{
				Slot:          128,
				ProposerIndex: 128,
				ParentRoot:    f.root[:],
				StateRoot:     f.root[:],
				Body:          bodyPbBlindedBellatrix(),
			},
			Signature: f.sig[:],
		}
		block := &SignedBeaconBlock{
			version: version.Bellatrix,
			block: &BeaconBlock{
				version:       version.Bellatrix,
				slot:          128,
				proposerIndex: 128,
				parentRoot:    f.root,
				stateRoot:     f.root,
				body:          bodyBlindedBellatrix(t),
			},
			signature: f.sig,
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.SignedBlindedBeaconBlockBellatrix)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("Capella", func(t *testing.T) {
		expectedBlock := &eth.SignedBeaconBlockCapella{
			Block: &eth.BeaconBlockCapella{
				Slot:          128,
				ProposerIndex: 128,
				ParentRoot:    f.root[:],
				StateRoot:     f.root[:],
				Body:          bodyPbCapella(),
			},
			Signature: f.sig[:],
		}
		block := &SignedBeaconBlock{
			version: version.Capella,
			block: &BeaconBlock{
				version:       version.Capella,
				slot:          128,
				proposerIndex: 128,
				parentRoot:    f.root,
				stateRoot:     f.root,
				body:          bodyCapella(t),
			},
			signature: f.sig,
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.SignedBeaconBlockCapella)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("CapellaBlind", func(t *testing.T) {
		expectedBlock := &eth.SignedBlindedBeaconBlockCapella{
			Block: &eth.BlindedBeaconBlockCapella{
				Slot:          128,
				ProposerIndex: 128,
				ParentRoot:    f.root[:],
				StateRoot:     f.root[:],
				Body:          bodyPbBlindedCapella(),
			},
			Signature: f.sig[:],
		}
		block := &SignedBeaconBlock{
			version: version.Capella,
			block: &BeaconBlock{
				version:       version.Capella,
				slot:          128,
				proposerIndex: 128,
				parentRoot:    f.root,
				stateRoot:     f.root,
				body:          bodyBlindedCapella(t),
			},
			signature: f.sig,
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.SignedBlindedBeaconBlockCapella)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("Deneb", func(t *testing.T) {
		expectedBlock := &eth.SignedBeaconBlockDeneb{
			Block: &eth.BeaconBlockDeneb{
				Slot:          128,
				ProposerIndex: 128,
				ParentRoot:    f.root[:],
				StateRoot:     f.root[:],
				Body:          bodyPbDeneb(),
			},
			Signature: f.sig[:],
		}
		block := &SignedBeaconBlock{
			version: version.Deneb,
			block: &BeaconBlock{
				version:       version.Deneb,
				slot:          128,
				proposerIndex: 128,
				parentRoot:    f.root,
				stateRoot:     f.root,
				body:          bodyDeneb(t),
			},
			signature: f.sig,
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.SignedBeaconBlockDeneb)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("DenebBlind", func(t *testing.T) {
		expectedBlock := &eth.SignedBlindedBeaconBlockDeneb{
			Message: &eth.BlindedBeaconBlockDeneb{
				Slot:          128,
				ProposerIndex: 128,
				ParentRoot:    f.root[:],
				StateRoot:     f.root[:],
				Body:          bodyPbBlindedDeneb(),
			},
			Signature: f.sig[:],
		}
		block := &SignedBeaconBlock{
			version: version.Deneb,
			block: &BeaconBlock{
				version:       version.Deneb,
				slot:          128,
				proposerIndex: 128,
				parentRoot:    f.root,
				stateRoot:     f.root,
				body:          bodyBlindedDeneb(t),
			},
			signature: f.sig,
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.SignedBlindedBeaconBlockDeneb)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
}

func Test_BeaconBlock_Proto(t *testing.T) {
	f := getFields()

	t.Run("Phase0", func(t *testing.T) {
		expectedBlock := &eth.BeaconBlock{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbPhase0(),
		}
		block := &BeaconBlock{
			version:       version.Phase0,
			slot:          128,
			proposerIndex: 128,
			parentRoot:    f.root,
			stateRoot:     f.root,
			body:          bodyPhase0(),
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BeaconBlock)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("Altair", func(t *testing.T) {
		expectedBlock := &eth.BeaconBlockAltair{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbAltair(),
		}
		block := &BeaconBlock{
			version:       version.Altair,
			slot:          128,
			proposerIndex: 128,
			parentRoot:    f.root,
			stateRoot:     f.root,
			body:          bodyAltair(),
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BeaconBlockAltair)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("Bellatrix", func(t *testing.T) {
		expectedBlock := &eth.BeaconBlockBellatrix{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbBellatrix(),
		}
		block := &BeaconBlock{
			version:       version.Bellatrix,
			slot:          128,
			proposerIndex: 128,
			parentRoot:    f.root,
			stateRoot:     f.root,
			body:          bodyBellatrix(t),
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BeaconBlockBellatrix)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("BellatrixBlind", func(t *testing.T) {
		expectedBlock := &eth.BlindedBeaconBlockBellatrix{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbBlindedBellatrix(),
		}
		block := &BeaconBlock{
			version:       version.Bellatrix,
			slot:          128,
			proposerIndex: 128,
			parentRoot:    f.root,
			stateRoot:     f.root,
			body:          bodyBlindedBellatrix(t),
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BlindedBeaconBlockBellatrix)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("Capella", func(t *testing.T) {
		expectedBlock := &eth.BeaconBlockCapella{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbCapella(),
		}
		block := &BeaconBlock{
			version:       version.Capella,
			slot:          128,
			proposerIndex: 128,
			parentRoot:    f.root,
			stateRoot:     f.root,
			body:          bodyCapella(t),
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BeaconBlockCapella)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("CapellaBlind", func(t *testing.T) {
		expectedBlock := &eth.BlindedBeaconBlockCapella{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbBlindedCapella(),
		}
		block := &BeaconBlock{
			version:       version.Capella,
			slot:          128,
			proposerIndex: 128,
			parentRoot:    f.root,
			stateRoot:     f.root,
			body:          bodyBlindedCapella(t),
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BlindedBeaconBlockCapella)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("Deneb", func(t *testing.T) {
		expectedBlock := &eth.BeaconBlockDeneb{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbDeneb(),
		}
		block := &BeaconBlock{
			version:       version.Deneb,
			slot:          128,
			proposerIndex: 128,
			parentRoot:    f.root,
			stateRoot:     f.root,
			body:          bodyDeneb(t),
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BeaconBlockDeneb)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("DenebBlind", func(t *testing.T) {
		expectedBlock := &eth.BlindedBeaconBlockDeneb{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbBlindedDeneb(),
		}
		block := &BeaconBlock{
			version:       version.Deneb,
			slot:          128,
			proposerIndex: 128,
			parentRoot:    f.root,
			stateRoot:     f.root,
			body:          bodyBlindedDeneb(t),
		}

		result, err := block.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BlindedBeaconBlockDeneb)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBlock.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
}

func Test_BeaconBlockBody_Proto(t *testing.T) {
	t.Run("Phase0", func(t *testing.T) {
		expectedBody := bodyPbPhase0()
		body := bodyPhase0()

		result, err := body.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BeaconBlockBody)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBody.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("Altair", func(t *testing.T) {
		expectedBody := bodyPbAltair()
		body := bodyAltair()
		result, err := body.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BeaconBlockBodyAltair)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBody.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("Bellatrix", func(t *testing.T) {
		expectedBody := bodyPbBellatrix()
		body := bodyBellatrix(t)
		result, err := body.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BeaconBlockBodyBellatrix)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBody.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("BellatrixBlind", func(t *testing.T) {
		expectedBody := bodyPbBlindedBellatrix()
		body := bodyBlindedBellatrix(t)
		result, err := body.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BlindedBeaconBlockBodyBellatrix)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBody.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("Capella", func(t *testing.T) {
		expectedBody := bodyPbCapella()
		body := bodyCapella(t)
		result, err := body.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BeaconBlockBodyCapella)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBody.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("CapellaBlind", func(t *testing.T) {
		expectedBody := bodyPbBlindedCapella()
		body := bodyBlindedCapella(t)
		result, err := body.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BlindedBeaconBlockBodyCapella)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBody.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("Deneb", func(t *testing.T) {
		expectedBody := bodyPbDeneb()
		body := bodyDeneb(t)
		result, err := body.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BeaconBlockBodyDeneb)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBody.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("DenebBlind", func(t *testing.T) {
		expectedBody := bodyPbBlindedDeneb()
		body := bodyBlindedDeneb(t)
		result, err := body.Proto()
		require.NoError(t, err)
		resultBlock, ok := result.(*eth.BlindedBeaconBlockBodyDeneb)
		require.Equal(t, true, ok)
		resultHTR, err := resultBlock.HashTreeRoot()
		require.NoError(t, err)
		expectedHTR, err := expectedBody.HashTreeRoot()
		require.NoError(t, err)
		assert.DeepEqual(t, expectedHTR, resultHTR)
	})
	t.Run("Bellatrix - wrong payload type", func(t *testing.T) {
		body := bodyBellatrix(t)
		body.executionPayload = &executionPayloadHeader{}
		_, err := body.Proto()
		require.ErrorIs(t, err, errPayloadWrongType)
	})
	t.Run("BellatrixBlind - wrong payload type", func(t *testing.T) {
		body := bodyBlindedBellatrix(t)
		body.executionPayloadHeader = &executionPayload{}
		_, err := body.Proto()
		require.ErrorIs(t, err, errPayloadHeaderWrongType)
	})
	t.Run("Capella - wrong payload type", func(t *testing.T) {
		body := bodyCapella(t)
		body.executionPayload = &executionPayloadHeaderCapella{}
		_, err := body.Proto()
		require.ErrorIs(t, err, errPayloadWrongType)
	})
	t.Run("CapellaBlind - wrong payload type", func(t *testing.T) {
		body := bodyBlindedCapella(t)
		body.executionPayloadHeader = &executionPayloadCapella{}
		_, err := body.Proto()
		require.ErrorIs(t, err, errPayloadHeaderWrongType)
	})
	t.Run("Deneb - wrong payload type", func(t *testing.T) {
		body := bodyDeneb(t)
		body.executionPayload = &executionPayloadHeaderDeneb{}
		_, err := body.Proto()
		require.ErrorIs(t, err, errPayloadWrongType)
	})
	t.Run("DenebBlind - wrong payload type", func(t *testing.T) {
		body := bodyBlindedDeneb(t)
		body.executionPayloadHeader = &executionPayloadDeneb{}
		_, err := body.Proto()
		require.ErrorIs(t, err, errPayloadHeaderWrongType)
	})
}

func Test_initSignedBlockFromProtoPhase0(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.SignedBeaconBlock{
		Block: &eth.BeaconBlock{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbPhase0(),
		},
		Signature: f.sig[:],
	}
	resultBlock, err := initSignedBlockFromProtoPhase0(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.block.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
	assert.DeepEqual(t, expectedBlock.Signature, resultBlock.signature[:])
}

func Test_initSignedBlockFromProtoAltair(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.SignedBeaconBlockAltair{
		Block: &eth.BeaconBlockAltair{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbAltair(),
		},
		Signature: f.sig[:],
	}
	resultBlock, err := initSignedBlockFromProtoAltair(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.block.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
	assert.DeepEqual(t, expectedBlock.Signature, resultBlock.signature[:])
}

func Test_initSignedBlockFromProtoBellatrix(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.SignedBeaconBlockBellatrix{
		Block: &eth.BeaconBlockBellatrix{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbBellatrix(),
		},
		Signature: f.sig[:],
	}
	resultBlock, err := initSignedBlockFromProtoBellatrix(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.block.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
	assert.DeepEqual(t, expectedBlock.Signature, resultBlock.signature[:])
}

func Test_initBlindedSignedBlockFromProtoBellatrix(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.SignedBlindedBeaconBlockBellatrix{
		Block: &eth.BlindedBeaconBlockBellatrix{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbBlindedBellatrix(),
		},
		Signature: f.sig[:],
	}
	resultBlock, err := initBlindedSignedBlockFromProtoBellatrix(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.block.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
	assert.DeepEqual(t, expectedBlock.Signature, resultBlock.signature[:])
}

func Test_initSignedBlockFromProtoCapella(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.SignedBeaconBlockCapella{
		Block: &eth.BeaconBlockCapella{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbCapella(),
		},
		Signature: f.sig[:],
	}
	resultBlock, err := initSignedBlockFromProtoCapella(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.block.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
	assert.DeepEqual(t, expectedBlock.Signature, resultBlock.signature[:])
}

func Test_initBlindedSignedBlockFromProtoCapella(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.SignedBlindedBeaconBlockCapella{
		Block: &eth.BlindedBeaconBlockCapella{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbBlindedCapella(),
		},
		Signature: f.sig[:],
	}
	resultBlock, err := initBlindedSignedBlockFromProtoCapella(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.block.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
	assert.DeepEqual(t, expectedBlock.Signature, resultBlock.signature[:])
}

func Test_initSignedBlockFromProtoDeneb(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.SignedBeaconBlockDeneb{
		Block: &eth.BeaconBlockDeneb{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbDeneb(),
		},
		Signature: f.sig[:],
	}
	resultBlock, err := initSignedBlockFromProtoDeneb(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.block.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
	assert.DeepEqual(t, expectedBlock.Signature, resultBlock.signature[:])
}

func Test_initBlindedSignedBlockFromProtoDeneb(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.SignedBlindedBeaconBlockDeneb{
		Message: &eth.BlindedBeaconBlockDeneb{
			Slot:          128,
			ProposerIndex: 128,
			ParentRoot:    f.root[:],
			StateRoot:     f.root[:],
			Body:          bodyPbBlindedDeneb(),
		},
		Signature: f.sig[:],
	}
	resultBlock, err := initBlindedSignedBlockFromProtoDeneb(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.block.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.Message.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
	assert.DeepEqual(t, expectedBlock.Signature, resultBlock.signature[:])
}

func Test_initBlockFromProtoPhase0(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.BeaconBlock{
		Slot:          128,
		ProposerIndex: 128,
		ParentRoot:    f.root[:],
		StateRoot:     f.root[:],
		Body:          bodyPbPhase0(),
	}
	resultBlock, err := initBlockFromProtoPhase0(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockFromProtoAltair(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.BeaconBlockAltair{
		Slot:          128,
		ProposerIndex: 128,
		ParentRoot:    f.root[:],
		StateRoot:     f.root[:],
		Body:          bodyPbAltair(),
	}
	resultBlock, err := initBlockFromProtoAltair(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockFromProtoBellatrix(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.BeaconBlockBellatrix{
		Slot:          128,
		ProposerIndex: 128,
		ParentRoot:    f.root[:],
		StateRoot:     f.root[:],
		Body:          bodyPbBellatrix(),
	}
	resultBlock, err := initBlockFromProtoBellatrix(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockFromProtoBlindedBellatrix(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.BlindedBeaconBlockBellatrix{
		Slot:          128,
		ProposerIndex: 128,
		ParentRoot:    f.root[:],
		StateRoot:     f.root[:],
		Body:          bodyPbBlindedBellatrix(),
	}
	resultBlock, err := initBlindedBlockFromProtoBellatrix(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockFromProtoCapella(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.BeaconBlockCapella{
		Slot:          128,
		ProposerIndex: 128,
		ParentRoot:    f.root[:],
		StateRoot:     f.root[:],
		Body:          bodyPbCapella(),
	}
	resultBlock, err := initBlockFromProtoCapella(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockFromProtoBlindedCapella(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.BlindedBeaconBlockCapella{
		Slot:          128,
		ProposerIndex: 128,
		ParentRoot:    f.root[:],
		StateRoot:     f.root[:],
		Body:          bodyPbBlindedCapella(),
	}
	resultBlock, err := initBlindedBlockFromProtoCapella(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockFromProtoDeneb(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.BeaconBlockDeneb{
		Slot:          128,
		ProposerIndex: 128,
		ParentRoot:    f.root[:],
		StateRoot:     f.root[:],
		Body:          bodyPbDeneb(),
	}
	resultBlock, err := initBlockFromProtoDeneb(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockFromProtoBlindedDeneb(t *testing.T) {
	f := getFields()
	expectedBlock := &eth.BlindedBeaconBlockDeneb{
		Slot:          128,
		ProposerIndex: 128,
		ParentRoot:    f.root[:],
		StateRoot:     f.root[:],
		Body:          bodyPbBlindedDeneb(),
	}
	resultBlock, err := initBlindedBlockFromProtoDeneb(expectedBlock)
	require.NoError(t, err)
	resultHTR, err := resultBlock.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBlock.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockBodyFromProtoPhase0(t *testing.T) {
	expectedBody := bodyPbPhase0()
	resultBody, err := initBlockBodyFromProtoPhase0(expectedBody)
	require.NoError(t, err)
	resultHTR, err := resultBody.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBody.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockBodyFromProtoAltair(t *testing.T) {
	expectedBody := bodyPbAltair()
	resultBody, err := initBlockBodyFromProtoAltair(expectedBody)
	require.NoError(t, err)
	resultHTR, err := resultBody.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBody.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockBodyFromProtoBellatrix(t *testing.T) {
	expectedBody := bodyPbBellatrix()
	resultBody, err := initBlockBodyFromProtoBellatrix(expectedBody)
	require.NoError(t, err)
	resultHTR, err := resultBody.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBody.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockBodyFromProtoBlindedBellatrix(t *testing.T) {
	expectedBody := bodyPbBlindedBellatrix()
	resultBody, err := initBlindedBlockBodyFromProtoBellatrix(expectedBody)
	require.NoError(t, err)
	resultHTR, err := resultBody.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBody.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockBodyFromProtoCapella(t *testing.T) {
	expectedBody := bodyPbCapella()
	resultBody, err := initBlockBodyFromProtoCapella(expectedBody)
	require.NoError(t, err)
	resultHTR, err := resultBody.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBody.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockBodyFromProtoBlindedCapella(t *testing.T) {
	expectedBody := bodyPbBlindedCapella()
	resultBody, err := initBlindedBlockBodyFromProtoCapella(expectedBody)
	require.NoError(t, err)
	resultHTR, err := resultBody.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBody.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockBodyFromProtoDeneb(t *testing.T) {
	expectedBody := bodyPbDeneb()
	resultBody, err := initBlockBodyFromProtoDeneb(expectedBody)
	require.NoError(t, err)
	resultHTR, err := resultBody.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBody.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func Test_initBlockBodyFromProtoBlindedDeneb(t *testing.T) {
	expectedBody := bodyPbBlindedDeneb()
	resultBody, err := initBlindedBlockBodyFromProtoDeneb(expectedBody)
	require.NoError(t, err)
	resultHTR, err := resultBody.HashTreeRoot()
	require.NoError(t, err)
	expectedHTR, err := expectedBody.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, resultHTR)
}

func bodyPbPhase0() *eth.BeaconBlockBody {
	f := getFields()
	return &eth.BeaconBlockBody{
		RandaoReveal: f.sig[:],
		Eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		Graffiti:          f.root[:],
		ProposerSlashings: f.proposerSlashings,
		AttesterSlashings: f.attesterSlashings,
		Attestations:      f.atts,
		Deposits:          f.deposits,
		VoluntaryExits:    f.voluntaryExits,
	}
}

func bodyPbAltair() *eth.BeaconBlockBodyAltair {
	f := getFields()
	return &eth.BeaconBlockBodyAltair{
		RandaoReveal: f.sig[:],
		Eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		Graffiti:          f.root[:],
		ProposerSlashings: f.proposerSlashings,
		AttesterSlashings: f.attesterSlashings,
		Attestations:      f.atts,
		Deposits:          f.deposits,
		VoluntaryExits:    f.voluntaryExits,
		SyncAggregate:     f.syncAggregate,
	}
}

func bodyPbBellatrix() *eth.BeaconBlockBodyBellatrix {
	f := getFields()
	return &eth.BeaconBlockBodyBellatrix{
		RandaoReveal: f.sig[:],
		Eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		Graffiti:          f.root[:],
		ProposerSlashings: f.proposerSlashings,
		AttesterSlashings: f.attesterSlashings,
		Attestations:      f.atts,
		Deposits:          f.deposits,
		VoluntaryExits:    f.voluntaryExits,
		SyncAggregate:     f.syncAggregate,
		ExecutionPayload:  f.execPayload,
	}
}

func bodyPbBlindedBellatrix() *eth.BlindedBeaconBlockBodyBellatrix {
	f := getFields()
	return &eth.BlindedBeaconBlockBodyBellatrix{
		RandaoReveal: f.sig[:],
		Eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		Graffiti:               f.root[:],
		ProposerSlashings:      f.proposerSlashings,
		AttesterSlashings:      f.attesterSlashings,
		Attestations:           f.atts,
		Deposits:               f.deposits,
		VoluntaryExits:         f.voluntaryExits,
		SyncAggregate:          f.syncAggregate,
		ExecutionPayloadHeader: f.execPayloadHeader,
	}
}

func bodyPbCapella() *eth.BeaconBlockBodyCapella {
	f := getFields()
	return &eth.BeaconBlockBodyCapella{
		RandaoReveal: f.sig[:],
		Eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		Graffiti:              f.root[:],
		ProposerSlashings:     f.proposerSlashings,
		AttesterSlashings:     f.attesterSlashings,
		Attestations:          f.atts,
		Deposits:              f.deposits,
		VoluntaryExits:        f.voluntaryExits,
		SyncAggregate:         f.syncAggregate,
		ExecutionPayload:      f.execPayloadCapella,
		BlsToExecutionChanges: f.blsToExecutionChanges,
	}
}

func bodyPbBlindedCapella() *eth.BlindedBeaconBlockBodyCapella {
	f := getFields()
	return &eth.BlindedBeaconBlockBodyCapella{
		RandaoReveal: f.sig[:],
		Eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		Graffiti:               f.root[:],
		ProposerSlashings:      f.proposerSlashings,
		AttesterSlashings:      f.attesterSlashings,
		Attestations:           f.atts,
		Deposits:               f.deposits,
		VoluntaryExits:         f.voluntaryExits,
		SyncAggregate:          f.syncAggregate,
		ExecutionPayloadHeader: f.execPayloadHeaderCapella,
		BlsToExecutionChanges:  f.blsToExecutionChanges,
	}
}

func bodyPbDeneb() *eth.BeaconBlockBodyDeneb {
	f := getFields()
	return &eth.BeaconBlockBodyDeneb{
		RandaoReveal: f.sig[:],
		Eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		Graffiti:              f.root[:],
		ProposerSlashings:     f.proposerSlashings,
		AttesterSlashings:     f.attesterSlashings,
		Attestations:          f.atts,
		Deposits:              f.deposits,
		VoluntaryExits:        f.voluntaryExits,
		SyncAggregate:         f.syncAggregate,
		ExecutionPayload:      f.execPayloadDeneb,
		BlsToExecutionChanges: f.blsToExecutionChanges,
		BlobKzgCommitments:    f.kzgCommitments,
	}
}

func bodyPbBlindedDeneb() *eth.BlindedBeaconBlockBodyDeneb {
	f := getFields()
	return &eth.BlindedBeaconBlockBodyDeneb{
		RandaoReveal: f.sig[:],
		Eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		Graffiti:               f.root[:],
		ProposerSlashings:      f.proposerSlashings,
		AttesterSlashings:      f.attesterSlashings,
		Attestations:           f.atts,
		Deposits:               f.deposits,
		VoluntaryExits:         f.voluntaryExits,
		SyncAggregate:          f.syncAggregate,
		ExecutionPayloadHeader: f.execPayloadHeaderDeneb,
		BlsToExecutionChanges:  f.blsToExecutionChanges,
		BlobKzgCommitments:     f.kzgCommitments,
	}
}

func bodyPhase0() *BeaconBlockBody {
	f := getFields()
	return &BeaconBlockBody{
		version:      version.Phase0,
		randaoReveal: f.sig,
		eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		graffiti:          f.root,
		proposerSlashings: f.proposerSlashings,
		attesterSlashings: f.attesterSlashings,
		attestations:      f.atts,
		deposits:          f.deposits,
		voluntaryExits:    f.voluntaryExits,
	}
}

func bodyAltair() *BeaconBlockBody {
	f := getFields()
	return &BeaconBlockBody{
		version:      version.Altair,
		randaoReveal: f.sig,
		eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		graffiti:          f.root,
		proposerSlashings: f.proposerSlashings,
		attesterSlashings: f.attesterSlashings,
		attestations:      f.atts,
		deposits:          f.deposits,
		voluntaryExits:    f.voluntaryExits,
		syncAggregate:     f.syncAggregate,
	}
}

func bodyBellatrix(t *testing.T) *BeaconBlockBody {
	f := getFields()
	p, err := WrappedExecutionPayload(f.execPayload)
	require.NoError(t, err)
	return &BeaconBlockBody{
		version:      version.Bellatrix,
		randaoReveal: f.sig,
		eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		graffiti:          f.root,
		proposerSlashings: f.proposerSlashings,
		attesterSlashings: f.attesterSlashings,
		attestations:      f.atts,
		deposits:          f.deposits,
		voluntaryExits:    f.voluntaryExits,
		syncAggregate:     f.syncAggregate,
		executionPayload:  p,
	}
}

func bodyBlindedBellatrix(t *testing.T) *BeaconBlockBody {
	f := getFields()
	ph, err := WrappedExecutionPayloadHeader(f.execPayloadHeader)
	require.NoError(t, err)
	return &BeaconBlockBody{
		version:      version.Bellatrix,
		randaoReveal: f.sig,
		eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		graffiti:               f.root,
		proposerSlashings:      f.proposerSlashings,
		attesterSlashings:      f.attesterSlashings,
		attestations:           f.atts,
		deposits:               f.deposits,
		voluntaryExits:         f.voluntaryExits,
		syncAggregate:          f.syncAggregate,
		executionPayloadHeader: ph,
	}
}

func bodyCapella(t *testing.T) *BeaconBlockBody {
	f := getFields()
	p, err := WrappedExecutionPayloadCapella(f.execPayloadCapella, big.NewInt(0))
	require.NoError(t, err)
	return &BeaconBlockBody{
		version:      version.Capella,
		randaoReveal: f.sig,
		eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		graffiti:              f.root,
		proposerSlashings:     f.proposerSlashings,
		attesterSlashings:     f.attesterSlashings,
		attestations:          f.atts,
		deposits:              f.deposits,
		voluntaryExits:        f.voluntaryExits,
		syncAggregate:         f.syncAggregate,
		executionPayload:      p,
		blsToExecutionChanges: f.blsToExecutionChanges,
	}
}

func bodyBlindedCapella(t *testing.T) *BeaconBlockBody {
	f := getFields()
	ph, err := WrappedExecutionPayloadHeaderCapella(f.execPayloadHeaderCapella, big.NewInt(0))
	require.NoError(t, err)
	return &BeaconBlockBody{
		version:      version.Capella,
		randaoReveal: f.sig,
		eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		graffiti:               f.root,
		proposerSlashings:      f.proposerSlashings,
		attesterSlashings:      f.attesterSlashings,
		attestations:           f.atts,
		deposits:               f.deposits,
		voluntaryExits:         f.voluntaryExits,
		syncAggregate:          f.syncAggregate,
		executionPayloadHeader: ph,
		blsToExecutionChanges:  f.blsToExecutionChanges,
	}
}

func bodyDeneb(t *testing.T) *BeaconBlockBody {
	f := getFields()
	p, err := WrappedExecutionPayloadDeneb(f.execPayloadDeneb, big.NewInt(0))
	require.NoError(t, err)
	return &BeaconBlockBody{
		version:      version.Deneb,
		randaoReveal: f.sig,
		eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		graffiti:              f.root,
		proposerSlashings:     f.proposerSlashings,
		attesterSlashings:     f.attesterSlashings,
		attestations:          f.atts,
		deposits:              f.deposits,
		voluntaryExits:        f.voluntaryExits,
		syncAggregate:         f.syncAggregate,
		executionPayload:      p,
		blsToExecutionChanges: f.blsToExecutionChanges,
		blobKzgCommitments:    f.kzgCommitments,
	}
}

func bodyBlindedDeneb(t *testing.T) *BeaconBlockBody {
	f := getFields()
	ph, err := WrappedExecutionPayloadHeaderDeneb(f.execPayloadHeaderDeneb, big.NewInt(0))
	require.NoError(t, err)
	return &BeaconBlockBody{
		version:      version.Deneb,
		randaoReveal: f.sig,
		eth1Data: &eth.Eth1Data{
			DepositRoot:  f.root[:],
			DepositCount: 128,
			BlockHash:    f.root[:],
		},
		graffiti:               f.root,
		proposerSlashings:      f.proposerSlashings,
		attesterSlashings:      f.attesterSlashings,
		attestations:           f.atts,
		deposits:               f.deposits,
		voluntaryExits:         f.voluntaryExits,
		syncAggregate:          f.syncAggregate,
		executionPayloadHeader: ph,
		blsToExecutionChanges:  f.blsToExecutionChanges,
		blobKzgCommitments:     f.kzgCommitments,
	}
}

func getFields() fields {
	b20 := make([]byte, 20)
	b48 := make([]byte, 48)
	b256 := make([]byte, 256)
	var root [32]byte
	var sig [96]byte
	b20[0], b20[5], b20[10] = 'q', 'u', 'x'
	b48[0], b48[5], b48[10] = 'b', 'a', 'r'
	b256[0], b256[5], b256[10] = 'x', 'y', 'z'
	root[0], root[5], root[10] = 'a', 'b', 'c'
	sig[0], sig[5], sig[10] = 'd', 'e', 'f'
	deposits := make([]*eth.Deposit, 16)
	for i := range deposits {
		deposits[i] = &eth.Deposit{}
		deposits[i].Proof = make([][]byte, 33)
		for j := range deposits[i].Proof {
			deposits[i].Proof[j] = root[:]
		}
		deposits[i].Data = &eth.Deposit_Data{
			PublicKey:             b48,
			WithdrawalCredentials: root[:],
			Amount:                128,
			Signature:             sig[:],
		}
	}
	atts := make([]*eth.Attestation, 128)
	for i := range atts {
		atts[i] = &eth.Attestation{}
		atts[i].Signature = sig[:]
		atts[i].AggregationBits = bitfield.NewBitlist(1)
		atts[i].Data = &eth.AttestationData{
			Slot:            128,
			CommitteeIndex:  128,
			BeaconBlockRoot: root[:],
			Source: &eth.Checkpoint{
				Epoch: 128,
				Root:  root[:],
			},
			Target: &eth.Checkpoint{
				Epoch: 128,
				Root:  root[:],
			},
		}
	}
	proposerSlashing := &eth.ProposerSlashing{
		Header_1: &eth.SignedBeaconBlockHeader{
			Header: &eth.BeaconBlockHeader{
				Slot:          128,
				ProposerIndex: 128,
				ParentRoot:    root[:],
				StateRoot:     root[:],
				BodyRoot:      root[:],
			},
			Signature: sig[:],
		},
		Header_2: &eth.SignedBeaconBlockHeader{
			Header: &eth.BeaconBlockHeader{
				Slot:          128,
				ProposerIndex: 128,
				ParentRoot:    root[:],
				StateRoot:     root[:],
				BodyRoot:      root[:],
			},
			Signature: sig[:],
		},
	}
	attesterSlashing := &eth.AttesterSlashing{
		Attestation_1: &eth.IndexedAttestation{
			AttestingIndices: []uint64{1, 2, 8},
			Data: &eth.AttestationData{
				Slot:            128,
				CommitteeIndex:  128,
				BeaconBlockRoot: root[:],
				Source: &eth.Checkpoint{
					Epoch: 128,
					Root:  root[:],
				},
				Target: &eth.Checkpoint{
					Epoch: 128,
					Root:  root[:],
				},
			},
			Signature: sig[:],
		},
		Attestation_2: &eth.IndexedAttestation{
			AttestingIndices: []uint64{1, 2, 8},
			Data: &eth.AttestationData{
				Slot:            128,
				CommitteeIndex:  128,
				BeaconBlockRoot: root[:],
				Source: &eth.Checkpoint{
					Epoch: 128,
					Root:  root[:],
				},
				Target: &eth.Checkpoint{
					Epoch: 128,
					Root:  root[:],
				},
			},
			Signature: sig[:],
		},
	}
	voluntaryExit := &eth.SignedVoluntaryExit{
		Exit: &eth.VoluntaryExit{
			Epoch:          128,
			ValidatorIndex: 128,
		},
		Signature: sig[:],
	}
	syncCommitteeBits := bitfield.NewBitvector512()
	syncCommitteeBits.SetBitAt(1, true)
	syncCommitteeBits.SetBitAt(2, true)
	syncCommitteeBits.SetBitAt(8, true)
	syncAggregate := &eth.SyncAggregate{
		SyncCommitteeBits:      syncCommitteeBits,
		SyncCommitteeSignature: sig[:],
	}
	execPayload := &enginev1.ExecutionPayload{
		ParentHash:    root[:],
		FeeRecipient:  b20,
		StateRoot:     root[:],
		ReceiptsRoot:  root[:],
		LogsBloom:     b256,
		PrevRandao:    root[:],
		BlockNumber:   128,
		GasLimit:      128,
		GasUsed:       128,
		Timestamp:     128,
		ExtraData:     root[:],
		BaseFeePerGas: root[:],
		BlockHash:     root[:],
		Transactions: [][]byte{
			[]byte("transaction1"),
			[]byte("transaction2"),
			[]byte("transaction8"),
		},
	}
	execPayloadHeader := &enginev1.ExecutionPayloadHeader{
		ParentHash:       root[:],
		FeeRecipient:     b20,
		StateRoot:        root[:],
		ReceiptsRoot:     root[:],
		LogsBloom:        b256,
		PrevRandao:       root[:],
		BlockNumber:      128,
		GasLimit:         128,
		GasUsed:          128,
		Timestamp:        128,
		ExtraData:        root[:],
		BaseFeePerGas:    root[:],
		BlockHash:        root[:],
		TransactionsRoot: root[:],
	}
	execPayloadCapella := &enginev1.ExecutionPayloadCapella{
		ParentHash:    root[:],
		FeeRecipient:  b20,
		StateRoot:     root[:],
		ReceiptsRoot:  root[:],
		LogsBloom:     b256,
		PrevRandao:    root[:],
		BlockNumber:   128,
		GasLimit:      128,
		GasUsed:       128,
		Timestamp:     128,
		ExtraData:     root[:],
		BaseFeePerGas: root[:],
		BlockHash:     root[:],
		Transactions: [][]byte{
			[]byte("transaction1"),
			[]byte("transaction2"),
			[]byte("transaction8"),
		},
		Withdrawals: []*enginev1.Withdrawal{
			{
				Index:   128,
				Address: b20,
				Amount:  128,
			},
		},
	}
	execPayloadHeaderCapella := &enginev1.ExecutionPayloadHeaderCapella{
		ParentHash:       root[:],
		FeeRecipient:     b20,
		StateRoot:        root[:],
		ReceiptsRoot:     root[:],
		LogsBloom:        b256,
		PrevRandao:       root[:],
		BlockNumber:      128,
		GasLimit:         128,
		GasUsed:          128,
		Timestamp:        128,
		ExtraData:        root[:],
		BaseFeePerGas:    root[:],
		BlockHash:        root[:],
		TransactionsRoot: root[:],
		WithdrawalsRoot:  root[:],
	}
	blsToExecutionChanges := []*eth.SignedBLSToExecutionChange{{
		Message: &eth.BLSToExecutionChange{
			ValidatorIndex:     128,
			FromBlsPubkey:      b48,
			ToExecutionAddress: b20,
		},
		Signature: sig[:],
	}}

	execPayloadDeneb := &enginev1.ExecutionPayloadDeneb{
		ParentHash:    root[:],
		FeeRecipient:  b20,
		StateRoot:     root[:],
		ReceiptsRoot:  root[:],
		LogsBloom:     b256,
		PrevRandao:    root[:],
		BlockNumber:   128,
		GasLimit:      128,
		GasUsed:       128,
		Timestamp:     128,
		ExtraData:     root[:],
		BaseFeePerGas: root[:],
		BlockHash:     root[:],
		Transactions: [][]byte{
			[]byte("transaction1"),
			[]byte("transaction2"),
			[]byte("transaction8"),
		},
		Withdrawals: []*enginev1.Withdrawal{
			{
				Index:   128,
				Address: b20,
				Amount:  128,
			},
		},
		BlobGasUsed:   128,
		ExcessBlobGas: 128,
	}
	execPayloadHeaderDeneb := &enginev1.ExecutionPayloadHeaderDeneb{
		ParentHash:       root[:],
		FeeRecipient:     b20,
		StateRoot:        root[:],
		ReceiptsRoot:     root[:],
		LogsBloom:        b256,
		PrevRandao:       root[:],
		BlockNumber:      128,
		GasLimit:         128,
		GasUsed:          128,
		Timestamp:        128,
		ExtraData:        root[:],
		BaseFeePerGas:    root[:],
		BlockHash:        root[:],
		TransactionsRoot: root[:],
		WithdrawalsRoot:  root[:],
		BlobGasUsed:      128,
		ExcessBlobGas:    128,
	}

	kzgCommitments := [][]byte{
		bytesutil.PadTo([]byte{123}, 48),
		bytesutil.PadTo([]byte{223}, 48),
		bytesutil.PadTo([]byte{183}, 48),
		bytesutil.PadTo([]byte{143}, 48),
	}

	return fields{
		root:                     root,
		sig:                      sig,
		deposits:                 deposits,
		atts:                     atts,
		proposerSlashings:        []*eth.ProposerSlashing{proposerSlashing},
		attesterSlashings:        []*eth.AttesterSlashing{attesterSlashing},
		voluntaryExits:           []*eth.SignedVoluntaryExit{voluntaryExit},
		syncAggregate:            syncAggregate,
		execPayload:              execPayload,
		execPayloadHeader:        execPayloadHeader,
		execPayloadCapella:       execPayloadCapella,
		execPayloadHeaderCapella: execPayloadHeaderCapella,
		execPayloadDeneb:         execPayloadDeneb,
		execPayloadHeaderDeneb:   execPayloadHeaderDeneb,
		blsToExecutionChanges:    blsToExecutionChanges,
		kzgCommitments:           kzgCommitments,
	}
}
