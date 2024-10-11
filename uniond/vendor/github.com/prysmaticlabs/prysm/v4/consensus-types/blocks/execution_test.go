package blocks_test

import (
	"math/big"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	consensus_types "github.com/prysmaticlabs/prysm/v4/consensus-types"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestWrapExecutionPayload(t *testing.T) {
	data := &enginev1.ExecutionPayload{GasUsed: 54}
	wsb, err := blocks.WrappedExecutionPayload(data)
	require.NoError(t, err)

	assert.DeepEqual(t, data, wsb.Proto())
}

func TestWrapExecutionPayloadHeader(t *testing.T) {
	data := &enginev1.ExecutionPayloadHeader{GasUsed: 54}
	wsb, err := blocks.WrappedExecutionPayloadHeader(data)
	require.NoError(t, err)

	assert.DeepEqual(t, data, wsb.Proto())
}

func TestWrapExecutionPayload_IsNil(t *testing.T) {
	_, err := blocks.WrappedExecutionPayload(nil)
	require.Equal(t, consensus_types.ErrNilObjectWrapped, err)

	data := &enginev1.ExecutionPayload{GasUsed: 54}
	wsb, err := blocks.WrappedExecutionPayload(data)
	require.NoError(t, err)

	assert.Equal(t, false, wsb.IsNil())
}

func TestWrapExecutionPayloadHeader_IsNil(t *testing.T) {
	_, err := blocks.WrappedExecutionPayloadHeader(nil)
	require.Equal(t, consensus_types.ErrNilObjectWrapped, err)

	data := &enginev1.ExecutionPayloadHeader{GasUsed: 54}
	wsb, err := blocks.WrappedExecutionPayloadHeader(data)
	require.NoError(t, err)

	assert.Equal(t, false, wsb.IsNil())
}

func TestWrapExecutionPayload_SSZ(t *testing.T) {
	wsb := createWrappedPayload(t)
	rt, err := wsb.HashTreeRoot()
	assert.NoError(t, err)
	assert.NotEmpty(t, rt)

	var b []byte
	b, err = wsb.MarshalSSZTo(b)
	assert.NoError(t, err)
	assert.NotEqual(t, 0, len(b))
	encoded, err := wsb.MarshalSSZ()
	require.NoError(t, err)
	assert.NotEqual(t, 0, wsb.SizeSSZ())
	assert.NoError(t, wsb.UnmarshalSSZ(encoded))
}

func TestWrapExecutionPayloadHeader_SSZ(t *testing.T) {
	wsb := createWrappedPayloadHeader(t)
	rt, err := wsb.HashTreeRoot()
	assert.NoError(t, err)
	assert.NotEmpty(t, rt)

	var b []byte
	b, err = wsb.MarshalSSZTo(b)
	assert.NoError(t, err)
	assert.NotEqual(t, 0, len(b))
	encoded, err := wsb.MarshalSSZ()
	require.NoError(t, err)
	assert.NotEqual(t, 0, wsb.SizeSSZ())
	assert.NoError(t, wsb.UnmarshalSSZ(encoded))
}

func TestWrapExecutionPayloadCapella(t *testing.T) {
	data := &enginev1.ExecutionPayloadCapella{
		ParentHash:    []byte("parenthash"),
		FeeRecipient:  []byte("feerecipient"),
		StateRoot:     []byte("stateroot"),
		ReceiptsRoot:  []byte("receiptsroot"),
		LogsBloom:     []byte("logsbloom"),
		PrevRandao:    []byte("prevrandao"),
		BlockNumber:   11,
		GasLimit:      22,
		GasUsed:       33,
		Timestamp:     44,
		ExtraData:     []byte("extradata"),
		BaseFeePerGas: []byte("basefeepergas"),
		BlockHash:     []byte("blockhash"),
		Transactions:  [][]byte{[]byte("transaction")},
		Withdrawals: []*enginev1.Withdrawal{{
			Index:          55,
			ValidatorIndex: 66,
			Address:        []byte("executionaddress"),
			Amount:         77,
		}},
	}
	payload, err := blocks.WrappedExecutionPayloadCapella(data, big.NewInt(10*1e9))
	require.NoError(t, err)
	wei, err := payload.ValueInWei()
	require.NoError(t, err)
	assert.Equal(t, 0, big.NewInt(10*1e9).Cmp(wei))
	gwei, err := payload.ValueInGwei()
	require.NoError(t, err)
	assert.Equal(t, uint64(10), gwei)

	assert.DeepEqual(t, data, payload.Proto())
}

func TestWrapExecutionPayloadHeaderCapella(t *testing.T) {
	data := &enginev1.ExecutionPayloadHeaderCapella{
		ParentHash:       []byte("parenthash"),
		FeeRecipient:     []byte("feerecipient"),
		StateRoot:        []byte("stateroot"),
		ReceiptsRoot:     []byte("receiptsroot"),
		LogsBloom:        []byte("logsbloom"),
		PrevRandao:       []byte("prevrandao"),
		BlockNumber:      11,
		GasLimit:         22,
		GasUsed:          33,
		Timestamp:        44,
		ExtraData:        []byte("extradata"),
		BaseFeePerGas:    []byte("basefeepergas"),
		BlockHash:        []byte("blockhash"),
		TransactionsRoot: []byte("transactionsroot"),
		WithdrawalsRoot:  []byte("withdrawalsroot"),
	}
	payload, err := blocks.WrappedExecutionPayloadHeaderCapella(data, big.NewInt(10*1e9))
	require.NoError(t, err)

	wei, err := payload.ValueInWei()
	require.NoError(t, err)
	assert.Equal(t, 0, big.NewInt(10*1e9).Cmp(wei))
	gwei, err := payload.ValueInGwei()
	require.NoError(t, err)
	assert.Equal(t, uint64(10), gwei)

	assert.DeepEqual(t, data, payload.Proto())

	txRoot, err := payload.TransactionsRoot()
	require.NoError(t, err)
	require.DeepEqual(t, txRoot, data.TransactionsRoot)

	wrRoot, err := payload.WithdrawalsRoot()
	require.NoError(t, err)
	require.DeepEqual(t, wrRoot, data.WithdrawalsRoot)
}

func TestWrapExecutionPayloadCapella_IsNil(t *testing.T) {
	_, err := blocks.WrappedExecutionPayloadCapella(nil, big.NewInt(0))
	require.Equal(t, consensus_types.ErrNilObjectWrapped, err)

	data := &enginev1.ExecutionPayloadCapella{GasUsed: 54}
	payload, err := blocks.WrappedExecutionPayloadCapella(data, big.NewInt(0))
	require.NoError(t, err)

	assert.Equal(t, false, payload.IsNil())
}

func TestWrapExecutionPayloadHeaderCapella_IsNil(t *testing.T) {
	_, err := blocks.WrappedExecutionPayloadHeaderCapella(nil, big.NewInt(0))
	require.Equal(t, consensus_types.ErrNilObjectWrapped, err)

	data := &enginev1.ExecutionPayloadHeaderCapella{GasUsed: 54}
	payload, err := blocks.WrappedExecutionPayloadHeaderCapella(data, big.NewInt(0))
	require.NoError(t, err)

	assert.Equal(t, false, payload.IsNil())
}

func TestWrapExecutionPayloadCapella_SSZ(t *testing.T) {
	payload := createWrappedPayloadCapella(t)
	rt, err := payload.HashTreeRoot()
	assert.NoError(t, err)
	assert.NotEmpty(t, rt)

	var b []byte
	b, err = payload.MarshalSSZTo(b)
	assert.NoError(t, err)
	assert.NotEqual(t, 0, len(b))
	encoded, err := payload.MarshalSSZ()
	require.NoError(t, err)
	assert.NotEqual(t, 0, payload.SizeSSZ())
	assert.NoError(t, payload.UnmarshalSSZ(encoded))
}

func TestWrapExecutionPayloadHeaderCapella_SSZ(t *testing.T) {
	payload := createWrappedPayloadHeaderCapella(t)
	rt, err := payload.HashTreeRoot()
	assert.NoError(t, err)
	assert.NotEmpty(t, rt)

	var b []byte
	b, err = payload.MarshalSSZTo(b)
	assert.NoError(t, err)
	assert.NotEqual(t, 0, len(b))
	encoded, err := payload.MarshalSSZ()
	require.NoError(t, err)
	assert.NotEqual(t, 0, payload.SizeSSZ())
	assert.NoError(t, payload.UnmarshalSSZ(encoded))
}

func Test_executionPayload_Pb(t *testing.T) {
	payload := createWrappedPayload(t)
	pb, err := payload.PbBellatrix()
	require.NoError(t, err)
	assert.DeepEqual(t, payload.Proto(), pb)

	_, err = payload.PbCapella()
	require.ErrorIs(t, err, consensus_types.ErrUnsupportedField)
}

func Test_executionPayloadHeader_Pb(t *testing.T) {
	payload := createWrappedPayloadHeader(t)
	_, err := payload.PbBellatrix()
	require.ErrorIs(t, err, consensus_types.ErrUnsupportedField)

	_, err = payload.PbCapella()
	require.ErrorIs(t, err, consensus_types.ErrUnsupportedField)
}

func Test_executionPayloadCapella_Pb(t *testing.T) {
	payload := createWrappedPayloadCapella(t)
	pb, err := payload.PbCapella()
	require.NoError(t, err)
	assert.DeepEqual(t, payload.Proto(), pb)

	_, err = payload.PbBellatrix()
	require.ErrorIs(t, err, consensus_types.ErrUnsupportedField)
}

func Test_executionPayloadHeaderCapella_Pb(t *testing.T) {
	payload := createWrappedPayloadHeaderCapella(t)
	_, err := payload.PbBellatrix()
	require.ErrorIs(t, err, consensus_types.ErrUnsupportedField)

	_, err = payload.PbCapella()
	require.ErrorIs(t, err, consensus_types.ErrUnsupportedField)
}

func TestWrapExecutionPayloadDeneb(t *testing.T) {
	data := &enginev1.ExecutionPayloadDeneb{
		ParentHash:    []byte("parenthash"),
		FeeRecipient:  []byte("feerecipient"),
		StateRoot:     []byte("stateroot"),
		ReceiptsRoot:  []byte("receiptsroot"),
		LogsBloom:     []byte("logsbloom"),
		PrevRandao:    []byte("prevrandao"),
		BlockNumber:   11,
		GasLimit:      22,
		GasUsed:       33,
		Timestamp:     44,
		ExtraData:     []byte("extradata"),
		BaseFeePerGas: []byte("basefeepergas"),
		BlockHash:     []byte("blockhash"),
		Transactions:  [][]byte{[]byte("transaction")},
		Withdrawals: []*enginev1.Withdrawal{{
			Index:          55,
			ValidatorIndex: 66,
			Address:        []byte("executionaddress"),
			Amount:         77,
		}},
		BlobGasUsed:   88,
		ExcessBlobGas: 99,
	}
	payload, err := blocks.WrappedExecutionPayloadDeneb(data, big.NewInt(420*1e9))
	require.NoError(t, err)
	wei, err := payload.ValueInWei()
	require.NoError(t, err)
	assert.Equal(t, 0, big.NewInt(420*1e9).Cmp(wei))
	gwei, err := payload.ValueInGwei()
	require.NoError(t, err)
	assert.Equal(t, uint64(420), gwei)

	g, err := payload.BlobGasUsed()
	require.NoError(t, err)
	require.DeepEqual(t, uint64(88), g)

	g, err = payload.ExcessBlobGas()
	require.NoError(t, err)
	require.DeepEqual(t, uint64(99), g)
}

func TestWrapExecutionPayloadHeaderDeneb(t *testing.T) {
	data := &enginev1.ExecutionPayloadHeaderDeneb{
		ParentHash:       []byte("parenthash"),
		FeeRecipient:     []byte("feerecipient"),
		StateRoot:        []byte("stateroot"),
		ReceiptsRoot:     []byte("receiptsroot"),
		LogsBloom:        []byte("logsbloom"),
		PrevRandao:       []byte("prevrandao"),
		BlockNumber:      11,
		GasLimit:         22,
		GasUsed:          33,
		Timestamp:        44,
		ExtraData:        []byte("extradata"),
		BaseFeePerGas:    []byte("basefeepergas"),
		BlockHash:        []byte("blockhash"),
		TransactionsRoot: []byte("transactionsroot"),
		WithdrawalsRoot:  []byte("withdrawalsroot"),
		BlobGasUsed:      88,
		ExcessBlobGas:    99,
	}
	payload, err := blocks.WrappedExecutionPayloadHeaderDeneb(data, big.NewInt(10*1e9))
	require.NoError(t, err)

	wei, err := payload.ValueInWei()
	require.NoError(t, err)
	assert.Equal(t, 0, big.NewInt(10*1e9).Cmp(wei))
	gwei, err := payload.ValueInGwei()
	require.NoError(t, err)
	assert.Equal(t, uint64(10), gwei)

	g, err := payload.BlobGasUsed()
	require.NoError(t, err)
	require.DeepEqual(t, uint64(88), g)

	g, err = payload.ExcessBlobGas()
	require.NoError(t, err)
	require.DeepEqual(t, uint64(99), g)
}

func TestWrapExecutionPayloadDeneb_SSZ(t *testing.T) {
	payload := createWrappedPayloadDeneb(t)
	rt, err := payload.HashTreeRoot()
	assert.NoError(t, err)
	assert.NotEmpty(t, rt)

	var b []byte
	b, err = payload.MarshalSSZTo(b)
	assert.NoError(t, err)
	assert.NotEqual(t, 0, len(b))
	encoded, err := payload.MarshalSSZ()
	require.NoError(t, err)
	assert.NotEqual(t, 0, payload.SizeSSZ())
	assert.NoError(t, payload.UnmarshalSSZ(encoded))
}

func TestWrapExecutionPayloadHeaderDeneb_SSZ(t *testing.T) {
	payload := createWrappedPayloadHeaderDeneb(t)
	rt, err := payload.HashTreeRoot()
	assert.NoError(t, err)
	assert.NotEmpty(t, rt)

	var b []byte
	b, err = payload.MarshalSSZTo(b)
	assert.NoError(t, err)
	assert.NotEqual(t, 0, len(b))
	encoded, err := payload.MarshalSSZ()
	require.NoError(t, err)
	assert.NotEqual(t, 0, payload.SizeSSZ())
	assert.NoError(t, payload.UnmarshalSSZ(encoded))
}

func createWrappedPayload(t testing.TB) interfaces.ExecutionData {
	wsb, err := blocks.WrappedExecutionPayload(&enginev1.ExecutionPayload{
		ParentHash:    make([]byte, fieldparams.RootLength),
		FeeRecipient:  make([]byte, fieldparams.FeeRecipientLength),
		StateRoot:     make([]byte, fieldparams.RootLength),
		ReceiptsRoot:  make([]byte, fieldparams.RootLength),
		LogsBloom:     make([]byte, fieldparams.LogsBloomLength),
		PrevRandao:    make([]byte, fieldparams.RootLength),
		BlockNumber:   0,
		GasLimit:      0,
		GasUsed:       0,
		Timestamp:     0,
		ExtraData:     make([]byte, 0),
		BaseFeePerGas: make([]byte, fieldparams.RootLength),
		BlockHash:     make([]byte, fieldparams.RootLength),
		Transactions:  make([][]byte, 0),
	})
	require.NoError(t, err)
	return wsb
}

func createWrappedPayloadHeader(t testing.TB) interfaces.ExecutionData {
	wsb, err := blocks.WrappedExecutionPayloadHeader(&enginev1.ExecutionPayloadHeader{
		ParentHash:       make([]byte, fieldparams.RootLength),
		FeeRecipient:     make([]byte, fieldparams.FeeRecipientLength),
		StateRoot:        make([]byte, fieldparams.RootLength),
		ReceiptsRoot:     make([]byte, fieldparams.RootLength),
		LogsBloom:        make([]byte, fieldparams.LogsBloomLength),
		PrevRandao:       make([]byte, fieldparams.RootLength),
		BlockNumber:      0,
		GasLimit:         0,
		GasUsed:          0,
		Timestamp:        0,
		ExtraData:        make([]byte, 0),
		BaseFeePerGas:    make([]byte, fieldparams.RootLength),
		BlockHash:        make([]byte, fieldparams.RootLength),
		TransactionsRoot: make([]byte, fieldparams.RootLength),
	})
	require.NoError(t, err)
	return wsb
}

func createWrappedPayloadCapella(t testing.TB) interfaces.ExecutionData {
	payload, err := blocks.WrappedExecutionPayloadCapella(&enginev1.ExecutionPayloadCapella{
		ParentHash:    make([]byte, fieldparams.RootLength),
		FeeRecipient:  make([]byte, fieldparams.FeeRecipientLength),
		StateRoot:     make([]byte, fieldparams.RootLength),
		ReceiptsRoot:  make([]byte, fieldparams.RootLength),
		LogsBloom:     make([]byte, fieldparams.LogsBloomLength),
		PrevRandao:    make([]byte, fieldparams.RootLength),
		BlockNumber:   0,
		GasLimit:      0,
		GasUsed:       0,
		Timestamp:     0,
		ExtraData:     make([]byte, 0),
		BaseFeePerGas: make([]byte, fieldparams.RootLength),
		BlockHash:     make([]byte, fieldparams.RootLength),
		Transactions:  make([][]byte, 0),
		Withdrawals:   make([]*enginev1.Withdrawal, 0),
	}, big.NewInt(0))
	require.NoError(t, err)
	return payload
}

func createWrappedPayloadHeaderCapella(t testing.TB) interfaces.ExecutionData {
	payload, err := blocks.WrappedExecutionPayloadHeaderCapella(&enginev1.ExecutionPayloadHeaderCapella{
		ParentHash:       make([]byte, fieldparams.RootLength),
		FeeRecipient:     make([]byte, fieldparams.FeeRecipientLength),
		StateRoot:        make([]byte, fieldparams.RootLength),
		ReceiptsRoot:     make([]byte, fieldparams.RootLength),
		LogsBloom:        make([]byte, fieldparams.LogsBloomLength),
		PrevRandao:       make([]byte, fieldparams.RootLength),
		BlockNumber:      0,
		GasLimit:         0,
		GasUsed:          0,
		Timestamp:        0,
		ExtraData:        make([]byte, 0),
		BaseFeePerGas:    make([]byte, fieldparams.RootLength),
		BlockHash:        make([]byte, fieldparams.RootLength),
		TransactionsRoot: make([]byte, fieldparams.RootLength),
		WithdrawalsRoot:  make([]byte, fieldparams.RootLength),
	}, big.NewInt(0))
	require.NoError(t, err)
	return payload
}

func createWrappedPayloadDeneb(t testing.TB) interfaces.ExecutionData {
	payload, err := blocks.WrappedExecutionPayloadDeneb(&enginev1.ExecutionPayloadDeneb{
		ParentHash:    make([]byte, fieldparams.RootLength),
		FeeRecipient:  make([]byte, fieldparams.FeeRecipientLength),
		StateRoot:     make([]byte, fieldparams.RootLength),
		ReceiptsRoot:  make([]byte, fieldparams.RootLength),
		LogsBloom:     make([]byte, fieldparams.LogsBloomLength),
		PrevRandao:    make([]byte, fieldparams.RootLength),
		BlockNumber:   0,
		GasLimit:      0,
		GasUsed:       0,
		Timestamp:     0,
		ExtraData:     make([]byte, 0),
		BaseFeePerGas: make([]byte, fieldparams.RootLength),
		BlockHash:     make([]byte, fieldparams.RootLength),
		Transactions:  make([][]byte, 0),
		Withdrawals:   make([]*enginev1.Withdrawal, 0),
		BlobGasUsed:   0,
		ExcessBlobGas: 0,
	}, big.NewInt(0))
	require.NoError(t, err)
	return payload
}

func createWrappedPayloadHeaderDeneb(t testing.TB) interfaces.ExecutionData {
	payload, err := blocks.WrappedExecutionPayloadHeaderDeneb(&enginev1.ExecutionPayloadHeaderDeneb{
		ParentHash:       make([]byte, fieldparams.RootLength),
		FeeRecipient:     make([]byte, fieldparams.FeeRecipientLength),
		StateRoot:        make([]byte, fieldparams.RootLength),
		ReceiptsRoot:     make([]byte, fieldparams.RootLength),
		LogsBloom:        make([]byte, fieldparams.LogsBloomLength),
		PrevRandao:       make([]byte, fieldparams.RootLength),
		BlockNumber:      0,
		GasLimit:         0,
		GasUsed:          0,
		Timestamp:        0,
		ExtraData:        make([]byte, 0),
		BaseFeePerGas:    make([]byte, fieldparams.RootLength),
		BlockHash:        make([]byte, fieldparams.RootLength),
		TransactionsRoot: make([]byte, fieldparams.RootLength),
		WithdrawalsRoot:  make([]byte, fieldparams.RootLength),
		BlobGasUsed:      0,
		ExcessBlobGas:    0,
	}, big.NewInt(0))
	require.NoError(t, err)
	return payload
}
