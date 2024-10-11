package blocks

import (
	"bytes"
	"errors"
	"math/big"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func Test_NewSignedBeaconBlock(t *testing.T) {
	t.Run("GenericSignedBeaconBlock_Phase0", func(t *testing.T) {
		pb := &eth.GenericSignedBeaconBlock_Phase0{
			Phase0: &eth.SignedBeaconBlock{
				Block: &eth.BeaconBlock{
					Body: &eth.BeaconBlockBody{}}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Phase0, b.Version())
	})
	t.Run("SignedBeaconBlock", func(t *testing.T) {
		pb := &eth.SignedBeaconBlock{
			Block: &eth.BeaconBlock{
				Body: &eth.BeaconBlockBody{}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Phase0, b.Version())
	})
	t.Run("GenericSignedBeaconBlock_Altair", func(t *testing.T) {
		pb := &eth.GenericSignedBeaconBlock_Altair{
			Altair: &eth.SignedBeaconBlockAltair{
				Block: &eth.BeaconBlockAltair{
					Body: &eth.BeaconBlockBodyAltair{}}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Altair, b.Version())
	})
	t.Run("SignedBeaconBlockAltair", func(t *testing.T) {
		pb := &eth.SignedBeaconBlockAltair{
			Block: &eth.BeaconBlockAltair{
				Body: &eth.BeaconBlockBodyAltair{}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Altair, b.Version())
	})
	t.Run("GenericSignedBeaconBlock_Bellatrix", func(t *testing.T) {
		pb := &eth.GenericSignedBeaconBlock_Bellatrix{
			Bellatrix: &eth.SignedBeaconBlockBellatrix{
				Block: &eth.BeaconBlockBellatrix{
					Body: &eth.BeaconBlockBodyBellatrix{}}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Bellatrix, b.Version())
	})
	t.Run("SignedBeaconBlockBellatrix", func(t *testing.T) {
		pb := &eth.SignedBeaconBlockBellatrix{
			Block: &eth.BeaconBlockBellatrix{
				Body: &eth.BeaconBlockBodyBellatrix{}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Bellatrix, b.Version())
	})
	t.Run("GenericSignedBeaconBlock_BlindedBellatrix", func(t *testing.T) {
		pb := &eth.GenericSignedBeaconBlock_BlindedBellatrix{
			BlindedBellatrix: &eth.SignedBlindedBeaconBlockBellatrix{
				Block: &eth.BlindedBeaconBlockBellatrix{
					Body: &eth.BlindedBeaconBlockBodyBellatrix{}}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Bellatrix, b.Version())
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("SignedBlindedBeaconBlockBellatrix", func(t *testing.T) {
		pb := &eth.SignedBlindedBeaconBlockBellatrix{
			Block: &eth.BlindedBeaconBlockBellatrix{
				Body: &eth.BlindedBeaconBlockBodyBellatrix{}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Bellatrix, b.Version())
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("GenericSignedBeaconBlock_Capella", func(t *testing.T) {
		pb := &eth.GenericSignedBeaconBlock_Capella{
			Capella: &eth.SignedBeaconBlockCapella{
				Block: &eth.BeaconBlockCapella{
					Body: &eth.BeaconBlockBodyCapella{}}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Capella, b.Version())
	})
	t.Run("SignedBeaconBlockCapella", func(t *testing.T) {
		pb := &eth.SignedBeaconBlockCapella{
			Block: &eth.BeaconBlockCapella{
				Body: &eth.BeaconBlockBodyCapella{}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Capella, b.Version())
	})
	t.Run("GenericSignedBeaconBlock_BlindedCapella", func(t *testing.T) {
		pb := &eth.GenericSignedBeaconBlock_BlindedCapella{
			BlindedCapella: &eth.SignedBlindedBeaconBlockCapella{
				Block: &eth.BlindedBeaconBlockCapella{
					Body: &eth.BlindedBeaconBlockBodyCapella{}}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Capella, b.Version())
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("SignedBlindedBeaconBlockCapella", func(t *testing.T) {
		pb := &eth.SignedBlindedBeaconBlockCapella{
			Block: &eth.BlindedBeaconBlockCapella{
				Body: &eth.BlindedBeaconBlockBodyCapella{}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Capella, b.Version())
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("GenericSignedBeaconBlock_Deneb", func(t *testing.T) {
		pb := &eth.GenericSignedBeaconBlock_Deneb{
			Deneb: &eth.SignedBeaconBlockContentsDeneb{
				Block: &eth.SignedBeaconBlockDeneb{Block: &eth.BeaconBlockDeneb{
					Body: &eth.BeaconBlockBodyDeneb{},
				}},
			},
		}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Deneb, b.Version())
	})
	t.Run("SignedBeaconBlockDeneb", func(t *testing.T) {
		pb := &eth.SignedBeaconBlockDeneb{
			Block: &eth.BeaconBlockDeneb{
				Body: &eth.BeaconBlockBodyDeneb{}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Deneb, b.Version())
	})
	t.Run("SignedBlindedBeaconBlockDeneb", func(t *testing.T) {
		pb := &eth.SignedBlindedBeaconBlockDeneb{
			Message: &eth.BlindedBeaconBlockDeneb{
				Body: &eth.BlindedBeaconBlockBodyDeneb{}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Deneb, b.Version())
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("GenericSignedBeaconBlock_BlindedDeneb", func(t *testing.T) {
		pb := &eth.GenericSignedBeaconBlock_BlindedDeneb{
			BlindedDeneb: &eth.SignedBlindedBeaconBlockDeneb{
				Message: &eth.BlindedBeaconBlockDeneb{
					Body: &eth.BlindedBeaconBlockBodyDeneb{},
				}}}
		b, err := NewSignedBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Deneb, b.Version())
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("nil", func(t *testing.T) {
		_, err := NewSignedBeaconBlock(nil)
		assert.ErrorContains(t, "received nil object", err)
	})
	t.Run("unsupported type", func(t *testing.T) {
		_, err := NewSignedBeaconBlock(&bytes.Reader{})
		assert.ErrorContains(t, "unable to create block from type *bytes.Reader", err)
	})
}

func Test_NewBeaconBlock(t *testing.T) {
	t.Run("GenericBeaconBlock_Phase0", func(t *testing.T) {
		pb := &eth.GenericBeaconBlock_Phase0{Phase0: &eth.BeaconBlock{Body: &eth.BeaconBlockBody{}}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Phase0, b.Version())
	})
	t.Run("BeaconBlock", func(t *testing.T) {
		pb := &eth.BeaconBlock{Body: &eth.BeaconBlockBody{}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Phase0, b.Version())
	})
	t.Run("GenericBeaconBlock_Altair", func(t *testing.T) {
		pb := &eth.GenericBeaconBlock_Altair{Altair: &eth.BeaconBlockAltair{Body: &eth.BeaconBlockBodyAltair{}}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Altair, b.Version())
	})
	t.Run("BeaconBlockAltair", func(t *testing.T) {
		pb := &eth.BeaconBlockAltair{Body: &eth.BeaconBlockBodyAltair{}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Altair, b.Version())
	})
	t.Run("GenericBeaconBlock_Bellatrix", func(t *testing.T) {
		pb := &eth.GenericBeaconBlock_Bellatrix{Bellatrix: &eth.BeaconBlockBellatrix{Body: &eth.BeaconBlockBodyBellatrix{}}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Bellatrix, b.Version())
	})
	t.Run("BeaconBlockBellatrix", func(t *testing.T) {
		pb := &eth.BeaconBlockBellatrix{Body: &eth.BeaconBlockBodyBellatrix{}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Bellatrix, b.Version())
	})
	t.Run("GenericBeaconBlock_BlindedBellatrix", func(t *testing.T) {
		pb := &eth.GenericBeaconBlock_BlindedBellatrix{BlindedBellatrix: &eth.BlindedBeaconBlockBellatrix{Body: &eth.BlindedBeaconBlockBodyBellatrix{}}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Bellatrix, b.Version())
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("BlindedBeaconBlockBellatrix", func(t *testing.T) {
		pb := &eth.BlindedBeaconBlockBellatrix{Body: &eth.BlindedBeaconBlockBodyBellatrix{}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Bellatrix, b.Version())
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("GenericBeaconBlock_Capella", func(t *testing.T) {
		pb := &eth.GenericBeaconBlock_Capella{Capella: &eth.BeaconBlockCapella{Body: &eth.BeaconBlockBodyCapella{}}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Capella, b.Version())
	})
	t.Run("BeaconBlockCapella", func(t *testing.T) {
		pb := &eth.BeaconBlockCapella{Body: &eth.BeaconBlockBodyCapella{}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Capella, b.Version())
	})
	t.Run("GenericBeaconBlock_BlindedCapella", func(t *testing.T) {
		pb := &eth.GenericBeaconBlock_BlindedCapella{BlindedCapella: &eth.BlindedBeaconBlockCapella{Body: &eth.BlindedBeaconBlockBodyCapella{}}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Capella, b.Version())
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("BlindedBeaconBlockCapella", func(t *testing.T) {
		pb := &eth.BlindedBeaconBlockCapella{Body: &eth.BlindedBeaconBlockBodyCapella{}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Capella, b.Version())
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("GenericBeaconBlock_Deneb", func(t *testing.T) {
		pb := &eth.GenericBeaconBlock_Deneb{Deneb: &eth.BeaconBlockContentsDeneb{Block: &eth.BeaconBlockDeneb{
			Body: &eth.BeaconBlockBodyDeneb{},
		}}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Deneb, b.Version())
	})
	t.Run("BeaconBlockDeneb", func(t *testing.T) {
		pb := &eth.BeaconBlockDeneb{Body: &eth.BeaconBlockBodyDeneb{}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Deneb, b.Version())
	})
	t.Run("BlindedBeaconBlockDeneb", func(t *testing.T) {
		pb := &eth.BlindedBeaconBlockDeneb{Body: &eth.BlindedBeaconBlockBodyDeneb{}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Deneb, b.Version())
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("GenericBeaconBlock_BlindedDeneb", func(t *testing.T) {
		pb := &eth.GenericBeaconBlock_BlindedDeneb{BlindedDeneb: &eth.BlindedBeaconBlockDeneb{Body: &eth.BlindedBeaconBlockBodyDeneb{}}}
		b, err := NewBeaconBlock(pb)
		require.NoError(t, err)
		assert.Equal(t, version.Deneb, b.Version())
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("nil", func(t *testing.T) {
		_, err := NewBeaconBlock(nil)
		assert.ErrorContains(t, "received nil object", err)
	})
	t.Run("unsupported type", func(t *testing.T) {
		_, err := NewBeaconBlock(&bytes.Reader{})
		assert.ErrorContains(t, "unable to create block from type *bytes.Reader", err)
	})
}

func Test_NewBeaconBlockBody(t *testing.T) {
	t.Run("BeaconBlockBody", func(t *testing.T) {
		pb := &eth.BeaconBlockBody{}
		i, err := NewBeaconBlockBody(pb)
		require.NoError(t, err)
		b, ok := i.(*BeaconBlockBody)
		require.Equal(t, true, ok)
		assert.Equal(t, version.Phase0, b.version)
	})
	t.Run("BeaconBlockBodyAltair", func(t *testing.T) {
		pb := &eth.BeaconBlockBodyAltair{}
		i, err := NewBeaconBlockBody(pb)
		require.NoError(t, err)
		b, ok := i.(*BeaconBlockBody)
		require.Equal(t, true, ok)
		assert.Equal(t, version.Altair, b.version)
	})
	t.Run("BeaconBlockBodyBellatrix", func(t *testing.T) {
		pb := &eth.BeaconBlockBodyBellatrix{}
		i, err := NewBeaconBlockBody(pb)
		require.NoError(t, err)
		b, ok := i.(*BeaconBlockBody)
		require.Equal(t, true, ok)
		assert.Equal(t, version.Bellatrix, b.version)
	})
	t.Run("BlindedBeaconBlockBodyBellatrix", func(t *testing.T) {
		pb := &eth.BlindedBeaconBlockBodyBellatrix{}
		i, err := NewBeaconBlockBody(pb)
		require.NoError(t, err)
		b, ok := i.(*BeaconBlockBody)
		require.Equal(t, true, ok)
		assert.Equal(t, version.Bellatrix, b.version)
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("BeaconBlockBodyCapella", func(t *testing.T) {
		pb := &eth.BeaconBlockBodyCapella{}
		i, err := NewBeaconBlockBody(pb)
		require.NoError(t, err)
		b, ok := i.(*BeaconBlockBody)
		require.Equal(t, true, ok)
		assert.Equal(t, version.Capella, b.version)
	})
	t.Run("BlindedBeaconBlockBodyCapella", func(t *testing.T) {
		pb := &eth.BlindedBeaconBlockBodyCapella{}
		i, err := NewBeaconBlockBody(pb)
		require.NoError(t, err)
		b, ok := i.(*BeaconBlockBody)
		require.Equal(t, true, ok)
		assert.Equal(t, version.Capella, b.version)
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("BeaconBlockBodyDeneb", func(t *testing.T) {
		pb := &eth.BeaconBlockBodyDeneb{}
		i, err := NewBeaconBlockBody(pb)
		require.NoError(t, err)
		b, ok := i.(*BeaconBlockBody)
		require.Equal(t, true, ok)
		assert.Equal(t, version.Deneb, b.version)
	})
	t.Run("BlindedBeaconBlockBodyDeneb", func(t *testing.T) {
		pb := &eth.BlindedBeaconBlockBodyDeneb{}
		i, err := NewBeaconBlockBody(pb)
		require.NoError(t, err)
		b, ok := i.(*BeaconBlockBody)
		require.Equal(t, true, ok)
		assert.Equal(t, version.Deneb, b.version)
		assert.Equal(t, true, b.IsBlinded())
	})
	t.Run("nil", func(t *testing.T) {
		_, err := NewBeaconBlockBody(nil)
		assert.ErrorContains(t, "received nil object", err)
	})
	t.Run("unsupported type", func(t *testing.T) {
		_, err := NewBeaconBlockBody(&bytes.Reader{})
		assert.ErrorContains(t, "unable to create block body from type *bytes.Reader", err)
	})
}

func Test_BuildSignedBeaconBlock(t *testing.T) {
	sig := bytesutil.ToBytes96([]byte("signature"))
	t.Run("Phase0", func(t *testing.T) {
		b := &BeaconBlock{version: version.Phase0, body: &BeaconBlockBody{version: version.Phase0}}
		sb, err := BuildSignedBeaconBlock(b, sig[:])
		require.NoError(t, err)
		assert.DeepEqual(t, sig, sb.Signature())
		assert.Equal(t, version.Phase0, sb.Version())
	})
	t.Run("Altair", func(t *testing.T) {
		b := &BeaconBlock{version: version.Altair, body: &BeaconBlockBody{version: version.Altair}}
		sb, err := BuildSignedBeaconBlock(b, sig[:])
		require.NoError(t, err)
		assert.DeepEqual(t, sig, sb.Signature())
		assert.Equal(t, version.Altair, sb.Version())
	})
	t.Run("Bellatrix", func(t *testing.T) {
		b := &BeaconBlock{version: version.Bellatrix, body: &BeaconBlockBody{version: version.Bellatrix}}
		sb, err := BuildSignedBeaconBlock(b, sig[:])
		require.NoError(t, err)
		assert.DeepEqual(t, sig, sb.Signature())
		assert.Equal(t, version.Bellatrix, sb.Version())
	})
	t.Run("BellatrixBlind", func(t *testing.T) {
		b := &BeaconBlock{version: version.Bellatrix, body: &BeaconBlockBody{version: version.Bellatrix}}
		sb, err := BuildSignedBeaconBlock(b, sig[:])
		require.NoError(t, err)
		assert.DeepEqual(t, sig, sb.Signature())
		assert.Equal(t, version.Bellatrix, sb.Version())
		assert.Equal(t, true, sb.IsBlinded())
	})
	t.Run("Capella", func(t *testing.T) {
		b := &BeaconBlock{version: version.Capella, body: &BeaconBlockBody{version: version.Capella}}
		sb, err := BuildSignedBeaconBlock(b, sig[:])
		require.NoError(t, err)
		assert.DeepEqual(t, sig, sb.Signature())
		assert.Equal(t, version.Capella, sb.Version())
	})
	t.Run("CapellaBlind", func(t *testing.T) {
		b := &BeaconBlock{version: version.Capella, body: &BeaconBlockBody{version: version.Capella}}
		sb, err := BuildSignedBeaconBlock(b, sig[:])
		require.NoError(t, err)
		assert.DeepEqual(t, sig, sb.Signature())
		assert.Equal(t, version.Capella, sb.Version())
		assert.Equal(t, true, sb.IsBlinded())
	})
	t.Run("Deneb", func(t *testing.T) {
		b := &BeaconBlock{version: version.Deneb, body: &BeaconBlockBody{version: version.Deneb}}
		sb, err := BuildSignedBeaconBlock(b, sig[:])
		require.NoError(t, err)
		assert.DeepEqual(t, sig, sb.Signature())
		assert.Equal(t, version.Deneb, sb.Version())
	})
	t.Run("DenebBlind", func(t *testing.T) {
		b := &BeaconBlock{version: version.Deneb, body: &BeaconBlockBody{version: version.Deneb}}
		sb, err := BuildSignedBeaconBlock(b, sig[:])
		require.NoError(t, err)
		assert.DeepEqual(t, sig, sb.Signature())
		assert.Equal(t, version.Deneb, sb.Version())
		assert.Equal(t, true, sb.IsBlinded())
	})
}

func TestBuildSignedBeaconBlockFromExecutionPayload(t *testing.T) {
	t.Run("nil block check", func(t *testing.T) {
		_, err := BuildSignedBeaconBlockFromExecutionPayload(nil, nil)
		require.ErrorIs(t, ErrNilSignedBeaconBlock, err)
	})
	t.Run("not blinded payload", func(t *testing.T) {
		altairBlock := &eth.SignedBeaconBlockAltair{
			Block: &eth.BeaconBlockAltair{
				Body: &eth.BeaconBlockBodyAltair{}}}
		blk, err := NewSignedBeaconBlock(altairBlock)
		require.NoError(t, err)
		_, err = BuildSignedBeaconBlockFromExecutionPayload(blk, nil)
		require.Equal(t, true, errors.Is(err, errNonBlindedSignedBeaconBlock))
	})
	t.Run("payload header root and payload root mismatch", func(t *testing.T) {
		blockHash := bytesutil.Bytes32(1)
		payload := &enginev1.ExecutionPayload{
			ParentHash:    make([]byte, fieldparams.RootLength),
			FeeRecipient:  make([]byte, 20),
			StateRoot:     make([]byte, fieldparams.RootLength),
			ReceiptsRoot:  make([]byte, fieldparams.RootLength),
			LogsBloom:     make([]byte, 256),
			PrevRandao:    make([]byte, fieldparams.RootLength),
			BaseFeePerGas: make([]byte, fieldparams.RootLength),
			BlockHash:     blockHash,
			Transactions:  make([][]byte, 0),
		}
		wrapped, err := WrappedExecutionPayload(payload)
		require.NoError(t, err)
		header, err := PayloadToHeader(wrapped)
		require.NoError(t, err)
		blindedBlock := &eth.SignedBlindedBeaconBlockBellatrix{
			Block: &eth.BlindedBeaconBlockBellatrix{
				Body: &eth.BlindedBeaconBlockBodyBellatrix{}}}

		// Modify the header.
		header.GasUsed += 1
		blindedBlock.Block.Body.ExecutionPayloadHeader = header

		blk, err := NewSignedBeaconBlock(blindedBlock)
		require.NoError(t, err)
		_, err = BuildSignedBeaconBlockFromExecutionPayload(blk, payload)
		require.ErrorContains(t, "roots do not match", err)
	})
	t.Run("ok", func(t *testing.T) {
		payload := &enginev1.ExecutionPayload{
			ParentHash:    make([]byte, fieldparams.RootLength),
			FeeRecipient:  make([]byte, 20),
			StateRoot:     make([]byte, fieldparams.RootLength),
			ReceiptsRoot:  make([]byte, fieldparams.RootLength),
			LogsBloom:     make([]byte, 256),
			PrevRandao:    make([]byte, fieldparams.RootLength),
			BaseFeePerGas: make([]byte, fieldparams.RootLength),
			BlockHash:     make([]byte, fieldparams.RootLength),
			Transactions:  make([][]byte, 0),
		}
		wrapped, err := WrappedExecutionPayload(payload)
		require.NoError(t, err)
		header, err := PayloadToHeader(wrapped)
		require.NoError(t, err)
		blindedBlock := &eth.SignedBlindedBeaconBlockBellatrix{
			Block: &eth.BlindedBeaconBlockBellatrix{
				Body: &eth.BlindedBeaconBlockBodyBellatrix{}}}
		blindedBlock.Block.Body.ExecutionPayloadHeader = header

		blk, err := NewSignedBeaconBlock(blindedBlock)
		require.NoError(t, err)
		builtBlock, err := BuildSignedBeaconBlockFromExecutionPayload(blk, payload)
		require.NoError(t, err)

		got, err := builtBlock.Block().Body().Execution()
		require.NoError(t, err)
		require.DeepEqual(t, payload, got.Proto())
	})
	t.Run("deneb", func(t *testing.T) {
		payload := &enginev1.ExecutionPayloadDeneb{
			ParentHash:    make([]byte, fieldparams.RootLength),
			FeeRecipient:  make([]byte, 20),
			StateRoot:     make([]byte, fieldparams.RootLength),
			ReceiptsRoot:  make([]byte, fieldparams.RootLength),
			LogsBloom:     make([]byte, 256),
			PrevRandao:    make([]byte, fieldparams.RootLength),
			BaseFeePerGas: make([]byte, fieldparams.RootLength),
			BlockHash:     make([]byte, fieldparams.RootLength),
			Transactions:  make([][]byte, 0),
			ExcessBlobGas: 123,
			BlobGasUsed:   321,
		}
		wrapped, err := WrappedExecutionPayloadDeneb(payload, big.NewInt(123))
		require.NoError(t, err)
		header, err := PayloadToHeaderDeneb(wrapped)
		require.NoError(t, err)
		blindedBlock := &eth.SignedBlindedBeaconBlockDeneb{
			Message: &eth.BlindedBeaconBlockDeneb{
				Body: &eth.BlindedBeaconBlockBodyDeneb{}}}
		blindedBlock.Message.Body.ExecutionPayloadHeader = header

		blk, err := NewSignedBeaconBlock(blindedBlock)
		require.NoError(t, err)
		builtBlock, err := BuildSignedBeaconBlockFromExecutionPayload(blk, payload)
		require.NoError(t, err)

		got, err := builtBlock.Block().Body().Execution()
		require.NoError(t, err)
		require.DeepEqual(t, payload, got.Proto())
		require.DeepEqual(t, uint64(123), payload.ExcessBlobGas)
		require.DeepEqual(t, uint64(321), payload.BlobGasUsed)
	})
}
