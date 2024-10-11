package enginev1_test

import (
	"encoding/json"
	"fmt"
	"math/big"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	gethtypes "github.com/ethereum/go-ethereum/core/types"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

type withdrawalJSON struct {
	Index     *hexutil.Uint64 `json:"index"`
	Validator *hexutil.Uint64 `json:"validatorIndex"`
	Address   *common.Address `json:"address"`
	Amount    *hexutil.Uint64 `json:"amount"`
}

func TestJsonMarshalUnmarshal(t *testing.T) {
	t.Run("payload attributes", func(t *testing.T) {
		random := bytesutil.PadTo([]byte("random"), fieldparams.RootLength)
		feeRecipient := bytesutil.PadTo([]byte("feeRecipient"), fieldparams.FeeRecipientLength)
		jsonPayload := &enginev1.PayloadAttributes{
			Timestamp:             1,
			PrevRandao:            random,
			SuggestedFeeRecipient: feeRecipient,
		}
		enc, err := json.Marshal(jsonPayload)
		require.NoError(t, err)
		payloadPb := &enginev1.PayloadAttributes{}
		require.NoError(t, json.Unmarshal(enc, payloadPb))
		require.DeepEqual(t, uint64(1), payloadPb.Timestamp)
		require.DeepEqual(t, random, payloadPb.PrevRandao)
		require.DeepEqual(t, feeRecipient, payloadPb.SuggestedFeeRecipient)
	})
	t.Run("payload status", func(t *testing.T) {
		hash := bytesutil.PadTo([]byte("hash"), fieldparams.RootLength)
		jsonPayload := &enginev1.PayloadStatus{
			Status:          enginev1.PayloadStatus_INVALID,
			LatestValidHash: hash,
			ValidationError: "failed validation",
		}
		enc, err := json.Marshal(jsonPayload)
		require.NoError(t, err)
		payloadPb := &enginev1.PayloadStatus{}
		require.NoError(t, json.Unmarshal(enc, payloadPb))
		require.DeepEqual(t, "INVALID", payloadPb.Status.String())
		require.DeepEqual(t, hash, payloadPb.LatestValidHash)
		require.DeepEqual(t, "failed validation", payloadPb.ValidationError)
	})
	t.Run("forkchoice state", func(t *testing.T) {
		head := bytesutil.PadTo([]byte("head"), fieldparams.RootLength)
		safe := bytesutil.PadTo([]byte("safe"), fieldparams.RootLength)
		finalized := bytesutil.PadTo([]byte("finalized"), fieldparams.RootLength)
		jsonPayload := &enginev1.ForkchoiceState{
			HeadBlockHash:      head,
			SafeBlockHash:      safe,
			FinalizedBlockHash: finalized,
		}
		enc, err := json.Marshal(jsonPayload)
		require.NoError(t, err)
		payloadPb := &enginev1.ForkchoiceState{}
		require.NoError(t, json.Unmarshal(enc, payloadPb))
		require.DeepEqual(t, head, payloadPb.HeadBlockHash)
		require.DeepEqual(t, safe, payloadPb.SafeBlockHash)
		require.DeepEqual(t, finalized, payloadPb.FinalizedBlockHash)
	})
	t.Run("execution payload", func(t *testing.T) {
		baseFeePerGas := big.NewInt(1770307273)
		parentHash := bytesutil.PadTo([]byte("parent"), fieldparams.RootLength)
		feeRecipient := bytesutil.PadTo([]byte("feeRecipient"), fieldparams.FeeRecipientLength)
		stateRoot := bytesutil.PadTo([]byte("stateRoot"), fieldparams.RootLength)
		receiptsRoot := bytesutil.PadTo([]byte("receiptsRoot"), fieldparams.RootLength)
		logsBloom := bytesutil.PadTo([]byte("logs"), fieldparams.LogsBloomLength)
		random := bytesutil.PadTo([]byte("random"), fieldparams.RootLength)
		extra := bytesutil.PadTo([]byte("extraData"), fieldparams.RootLength)
		hash := bytesutil.PadTo([]byte("hash"), fieldparams.RootLength)
		jsonPayload := &enginev1.ExecutionPayload{
			ParentHash:    parentHash,
			FeeRecipient:  feeRecipient,
			StateRoot:     stateRoot,
			ReceiptsRoot:  receiptsRoot,
			LogsBloom:     logsBloom,
			PrevRandao:    random,
			BlockNumber:   1,
			GasLimit:      2,
			GasUsed:       3,
			Timestamp:     4,
			ExtraData:     extra,
			BaseFeePerGas: baseFeePerGas.Bytes(),
			BlockHash:     hash,
			Transactions:  [][]byte{[]byte("hi")},
		}
		enc, err := json.Marshal(jsonPayload)
		require.NoError(t, err)
		payloadPb := &enginev1.ExecutionPayload{}
		require.NoError(t, json.Unmarshal(enc, payloadPb))
		require.DeepEqual(t, parentHash, payloadPb.ParentHash)
		require.DeepEqual(t, feeRecipient, payloadPb.FeeRecipient)
		require.DeepEqual(t, stateRoot, payloadPb.StateRoot)
		require.DeepEqual(t, receiptsRoot, payloadPb.ReceiptsRoot)
		require.DeepEqual(t, logsBloom, payloadPb.LogsBloom)
		require.DeepEqual(t, random, payloadPb.PrevRandao)
		require.DeepEqual(t, uint64(1), payloadPb.BlockNumber)
		require.DeepEqual(t, uint64(2), payloadPb.GasLimit)
		require.DeepEqual(t, uint64(3), payloadPb.GasUsed)
		require.DeepEqual(t, uint64(4), payloadPb.Timestamp)
		require.DeepEqual(t, extra, payloadPb.ExtraData)
		require.DeepEqual(t, bytesutil.PadTo(baseFeePerGas.Bytes(), fieldparams.RootLength), payloadPb.BaseFeePerGas)
		require.DeepEqual(t, hash, payloadPb.BlockHash)
		require.DeepEqual(t, [][]byte{[]byte("hi")}, payloadPb.Transactions)
	})
	t.Run("execution payload Capella", func(t *testing.T) {
		parentHash := common.BytesToHash([]byte("parent"))
		feeRecipient := common.BytesToAddress([]byte("feeRecipient"))
		stateRoot := common.BytesToHash([]byte("stateRoot"))
		receiptsRoot := common.BytesToHash([]byte("receiptsRoot"))
		logsBloom := hexutil.Bytes(bytesutil.PadTo([]byte("logs"), fieldparams.LogsBloomLength))
		random := common.BytesToHash([]byte("random"))
		extra := common.BytesToHash([]byte("extra"))
		hash := common.BytesToHash([]byte("hash"))
		bn := hexutil.Uint64(1)
		gl := hexutil.Uint64(2)
		gu := hexutil.Uint64(3)
		ts := hexutil.Uint64(4)

		resp := &enginev1.GetPayloadV2ResponseJson{
			BlockValue: fmt.Sprint("0x123"),
			ExecutionPayload: &enginev1.ExecutionPayloadCapellaJSON{
				ParentHash:    &parentHash,
				FeeRecipient:  &feeRecipient,
				StateRoot:     &stateRoot,
				ReceiptsRoot:  &receiptsRoot,
				LogsBloom:     &logsBloom,
				PrevRandao:    &random,
				BlockNumber:   &bn,
				GasLimit:      &gl,
				GasUsed:       &gu,
				Timestamp:     &ts,
				ExtraData:     hexutil.Bytes(extra[:]),
				BaseFeePerGas: "0x123",
				BlockHash:     &hash,
				Transactions:  []hexutil.Bytes{{}},
				Withdrawals: []*enginev1.Withdrawal{{
					Index:          1,
					ValidatorIndex: 1,
					Address:        bytesutil.PadTo([]byte("address"), 20),
					Amount:         1,
				}},
			},
		}
		enc, err := json.Marshal(resp)
		require.NoError(t, err)
		pb := &enginev1.ExecutionPayloadCapellaWithValue{}
		require.NoError(t, json.Unmarshal(enc, pb))
		require.DeepEqual(t, parentHash.Bytes(), pb.Payload.ParentHash)
		require.DeepEqual(t, feeRecipient.Bytes(), pb.Payload.FeeRecipient)
		require.DeepEqual(t, stateRoot.Bytes(), pb.Payload.StateRoot)
		require.DeepEqual(t, receiptsRoot.Bytes(), pb.Payload.ReceiptsRoot)
		require.DeepEqual(t, logsBloom, hexutil.Bytes(pb.Payload.LogsBloom))
		require.DeepEqual(t, random.Bytes(), pb.Payload.PrevRandao)
		require.DeepEqual(t, uint64(1), pb.Payload.BlockNumber)
		require.DeepEqual(t, uint64(2), pb.Payload.GasLimit)
		require.DeepEqual(t, uint64(3), pb.Payload.GasUsed)
		require.DeepEqual(t, uint64(4), pb.Payload.Timestamp)
		require.DeepEqual(t, extra.Bytes(), pb.Payload.ExtraData)
		feePerGas := new(big.Int).SetBytes(pb.Payload.BaseFeePerGas)
		require.Equal(t, "15832716547479101977395928904157292820330083199902421483727713169783165812736", feePerGas.String())
		require.DeepEqual(t, hash.Bytes(), pb.Payload.BlockHash)
		require.DeepEqual(t, [][]byte{{}}, pb.Payload.Transactions)
		require.Equal(t, 1, len(pb.Payload.Withdrawals))
		withdrawal := pb.Payload.Withdrawals[0]
		require.Equal(t, uint64(1), withdrawal.Index)
		require.Equal(t, primitives.ValidatorIndex(1), withdrawal.ValidatorIndex)
		require.DeepEqual(t, bytesutil.PadTo([]byte("address"), 20), withdrawal.Address)
		require.Equal(t, uint64(1), withdrawal.Amount)
	})
	t.Run("execution payload deneb", func(t *testing.T) {
		parentHash := common.BytesToHash([]byte("parent"))
		feeRecipient := common.BytesToAddress([]byte("feeRecipient"))
		stateRoot := common.BytesToHash([]byte("stateRoot"))
		receiptsRoot := common.BytesToHash([]byte("receiptsRoot"))
		logsBloom := hexutil.Bytes(bytesutil.PadTo([]byte("logs"), fieldparams.LogsBloomLength))
		random := common.BytesToHash([]byte("random"))
		extra := common.BytesToHash([]byte("extra"))
		hash := common.BytesToHash([]byte("hash"))
		bn := hexutil.Uint64(1)
		gl := hexutil.Uint64(2)
		gu := hexutil.Uint64(3)
		ts := hexutil.Uint64(4)
		bgu := hexutil.Uint64(5)
		ebg := hexutil.Uint64(6)

		resp := &enginev1.GetPayloadV3ResponseJson{
			BlobsBundle: &enginev1.BlobBundleJSON{
				Commitments: []hexutil.Bytes{{'a'}, {'b'}, {'c'}, {'d'}},
				Proofs:      []hexutil.Bytes{{'e'}, {'f'}, {'g'}, {'h'}},
				Blobs:       []hexutil.Bytes{{'i'}, {'j'}, {'k'}, {'l'}},
			},
			BlockValue: fmt.Sprint("0x123"),
			ExecutionPayload: &enginev1.ExecutionPayloadDenebJSON{
				ParentHash:    &parentHash,
				FeeRecipient:  &feeRecipient,
				StateRoot:     &stateRoot,
				ReceiptsRoot:  &receiptsRoot,
				LogsBloom:     &logsBloom,
				PrevRandao:    &random,
				BlockNumber:   &bn,
				GasLimit:      &gl,
				GasUsed:       &gu,
				Timestamp:     &ts,
				ExtraData:     hexutil.Bytes(extra[:]),
				BaseFeePerGas: "0x123",
				BlockHash:     &hash,
				Transactions:  []hexutil.Bytes{{}},
				Withdrawals: []*enginev1.Withdrawal{{
					Index:          1,
					ValidatorIndex: 1,
					Address:        bytesutil.PadTo([]byte("address"), 20),
					Amount:         1,
				}},
				BlobGasUsed:   &bgu,
				ExcessBlobGas: &ebg,
			},
		}
		enc, err := json.Marshal(resp)
		require.NoError(t, err)
		pb := &enginev1.ExecutionPayloadDenebWithValueAndBlobsBundle{}
		require.NoError(t, json.Unmarshal(enc, pb))
		require.DeepEqual(t, parentHash.Bytes(), pb.Payload.ParentHash)
		require.DeepEqual(t, feeRecipient.Bytes(), pb.Payload.FeeRecipient)
		require.DeepEqual(t, stateRoot.Bytes(), pb.Payload.StateRoot)
		require.DeepEqual(t, receiptsRoot.Bytes(), pb.Payload.ReceiptsRoot)
		require.DeepEqual(t, logsBloom, hexutil.Bytes(pb.Payload.LogsBloom))
		require.DeepEqual(t, random.Bytes(), pb.Payload.PrevRandao)
		require.DeepEqual(t, uint64(1), pb.Payload.BlockNumber)
		require.DeepEqual(t, uint64(2), pb.Payload.GasLimit)
		require.DeepEqual(t, uint64(3), pb.Payload.GasUsed)
		require.DeepEqual(t, uint64(4), pb.Payload.Timestamp)
		require.DeepEqual(t, uint64(5), pb.Payload.BlobGasUsed)
		require.DeepEqual(t, uint64(6), pb.Payload.ExcessBlobGas)
		require.DeepEqual(t, extra.Bytes(), pb.Payload.ExtraData)
		feePerGas := new(big.Int).SetBytes(pb.Payload.BaseFeePerGas)
		require.Equal(t, "15832716547479101977395928904157292820330083199902421483727713169783165812736", feePerGas.String())
		require.DeepEqual(t, hash.Bytes(), pb.Payload.BlockHash)
		require.DeepEqual(t, [][]byte{{}}, pb.Payload.Transactions)
		require.Equal(t, 1, len(pb.Payload.Withdrawals))
		withdrawal := pb.Payload.Withdrawals[0]
		require.Equal(t, uint64(1), withdrawal.Index)
		require.Equal(t, primitives.ValidatorIndex(1), withdrawal.ValidatorIndex)
		require.DeepEqual(t, bytesutil.PadTo([]byte("address"), 20), withdrawal.Address)
		require.Equal(t, uint64(1), withdrawal.Amount)
		require.DeepEqual(t, [][]byte{
			bytesutil.PadTo([]byte{'e'}, 48),
			bytesutil.PadTo([]byte{'f'}, 48),
			bytesutil.PadTo([]byte{'g'}, 48),
			bytesutil.PadTo([]byte{'h'}, 48)}, pb.BlobsBundle.Proofs)
		require.DeepEqual(t, [][]byte{
			bytesutil.PadTo([]byte{'a'}, 48),
			bytesutil.PadTo([]byte{'b'}, 48),
			bytesutil.PadTo([]byte{'c'}, 48),
			bytesutil.PadTo([]byte{'d'}, 48)}, pb.BlobsBundle.KzgCommitments)
		require.DeepEqual(t, [][]byte{
			bytesutil.PadTo([]byte{'i'}, 131072),
			bytesutil.PadTo([]byte{'j'}, 131072),
			bytesutil.PadTo([]byte{'k'}, 131072),
			bytesutil.PadTo([]byte{'l'}, 131072)}, pb.BlobsBundle.Blobs)
	})

	t.Run("execution block", func(t *testing.T) {
		baseFeePerGas := big.NewInt(1770307273)
		want := &gethtypes.Header{
			Number:      big.NewInt(1),
			ParentHash:  common.BytesToHash([]byte("parent")),
			UncleHash:   common.BytesToHash([]byte("uncle")),
			Coinbase:    common.BytesToAddress([]byte("coinbase")),
			Root:        common.BytesToHash([]byte("uncle")),
			TxHash:      common.BytesToHash([]byte("txHash")),
			ReceiptHash: common.BytesToHash([]byte("receiptHash")),
			Bloom:       gethtypes.BytesToBloom([]byte("bloom")),
			Difficulty:  big.NewInt(2),
			GasLimit:    3,
			GasUsed:     4,
			Time:        5,
			BaseFee:     baseFeePerGas,
			Extra:       []byte("extraData"),
			MixDigest:   common.BytesToHash([]byte("mix")),
			Nonce:       gethtypes.EncodeNonce(6),
		}
		enc, err := json.Marshal(want)
		require.NoError(t, err)

		payloadItems := make(map[string]interface{})
		require.NoError(t, json.Unmarshal(enc, &payloadItems))

		blockHash := want.Hash()
		payloadItems["hash"] = blockHash.String()
		payloadItems["totalDifficulty"] = "0x393a2e53de197c"

		encodedPayloadItems, err := json.Marshal(payloadItems)
		require.NoError(t, err)

		payloadPb := &enginev1.ExecutionBlock{}
		require.NoError(t, json.Unmarshal(encodedPayloadItems, payloadPb))

		require.DeepEqual(t, blockHash, payloadPb.Hash)
		require.DeepEqual(t, want.Number, payloadPb.Number)
		require.DeepEqual(t, want.ParentHash, payloadPb.ParentHash)
		require.DeepEqual(t, want.UncleHash, payloadPb.UncleHash)
		require.DeepEqual(t, want.Coinbase, payloadPb.Coinbase)
		require.DeepEqual(t, want.Root, payloadPb.Root)
		require.DeepEqual(t, want.TxHash, payloadPb.TxHash)
		require.DeepEqual(t, want.ReceiptHash, payloadPb.ReceiptHash)
		require.DeepEqual(t, want.Bloom, payloadPb.Bloom)
		require.DeepEqual(t, want.Difficulty, payloadPb.Difficulty)
		require.DeepEqual(t, payloadItems["totalDifficulty"], payloadPb.TotalDifficulty)
		require.DeepEqual(t, want.GasUsed, payloadPb.GasUsed)
		require.DeepEqual(t, want.GasLimit, payloadPb.GasLimit)
		require.DeepEqual(t, want.Time, payloadPb.Time)
		require.DeepEqual(t, want.BaseFee, payloadPb.BaseFee)
		require.DeepEqual(t, want.Extra, payloadPb.Extra)
		require.DeepEqual(t, want.MixDigest, payloadPb.MixDigest)
		require.DeepEqual(t, want.Nonce, payloadPb.Nonce)
	})
	t.Run("execution block with txs as hashes", func(t *testing.T) {
		baseFeePerGas := big.NewInt(1770307273)
		want := &gethtypes.Header{
			Number:      big.NewInt(1),
			ParentHash:  common.BytesToHash([]byte("parent")),
			UncleHash:   common.BytesToHash([]byte("uncle")),
			Coinbase:    common.BytesToAddress([]byte("coinbase")),
			Root:        common.BytesToHash([]byte("uncle")),
			TxHash:      common.BytesToHash([]byte("txHash")),
			ReceiptHash: common.BytesToHash([]byte("receiptHash")),
			Bloom:       gethtypes.BytesToBloom([]byte("bloom")),
			Difficulty:  big.NewInt(2),
			GasLimit:    3,
			GasUsed:     4,
			Time:        5,
			BaseFee:     baseFeePerGas,
			Extra:       []byte("extraData"),
			MixDigest:   common.BytesToHash([]byte("mix")),
			Nonce:       gethtypes.EncodeNonce(6),
		}
		enc, err := json.Marshal(want)
		require.NoError(t, err)

		payloadItems := make(map[string]interface{})
		require.NoError(t, json.Unmarshal(enc, &payloadItems))

		blockHash := want.Hash()
		payloadItems["hash"] = blockHash.String()
		payloadItems["totalDifficulty"] = "0x393a2e53de197c"
		payloadItems["transactions"] = []string{"0xd57870623ea84ac3e2ffafbee9417fd1263b825b1107b8d606c25460dabeb693"}

		encodedPayloadItems, err := json.Marshal(payloadItems)
		require.NoError(t, err)

		payloadPb := &enginev1.ExecutionBlock{}
		require.NoError(t, json.Unmarshal(encodedPayloadItems, payloadPb))

		require.DeepEqual(t, blockHash, payloadPb.Hash)
		require.DeepEqual(t, want.Number, payloadPb.Number)
		require.DeepEqual(t, want.ParentHash, payloadPb.ParentHash)
		require.DeepEqual(t, want.UncleHash, payloadPb.UncleHash)
		require.DeepEqual(t, want.Coinbase, payloadPb.Coinbase)
		require.DeepEqual(t, want.Root, payloadPb.Root)
		require.DeepEqual(t, want.TxHash, payloadPb.TxHash)
		require.DeepEqual(t, want.ReceiptHash, payloadPb.ReceiptHash)
		require.DeepEqual(t, want.Bloom, payloadPb.Bloom)
		require.DeepEqual(t, want.Difficulty, payloadPb.Difficulty)
		require.DeepEqual(t, payloadItems["totalDifficulty"], payloadPb.TotalDifficulty)
		require.DeepEqual(t, want.GasUsed, payloadPb.GasUsed)
		require.DeepEqual(t, want.GasLimit, payloadPb.GasLimit)
		require.DeepEqual(t, want.Time, payloadPb.Time)
		require.DeepEqual(t, want.BaseFee, payloadPb.BaseFee)
		require.DeepEqual(t, want.Extra, payloadPb.Extra)
		require.DeepEqual(t, want.MixDigest, payloadPb.MixDigest)
		require.DeepEqual(t, want.Nonce, payloadPb.Nonce)

		// Expect no transaction objects in the unmarshaled data.
		require.Equal(t, 0, len(payloadPb.Transactions))
	})
	t.Run("execution block with full transaction data", func(t *testing.T) {
		baseFeePerGas := big.NewInt(1770307273)
		want := &gethtypes.Header{
			Number:      big.NewInt(1),
			ParentHash:  common.BytesToHash([]byte("parent")),
			UncleHash:   common.BytesToHash([]byte("uncle")),
			Coinbase:    common.BytesToAddress([]byte("coinbase")),
			Root:        common.BytesToHash([]byte("uncle")),
			TxHash:      common.BytesToHash([]byte("txHash")),
			ReceiptHash: common.BytesToHash([]byte("receiptHash")),
			Bloom:       gethtypes.BytesToBloom([]byte("bloom")),
			Difficulty:  big.NewInt(2),
			GasLimit:    3,
			GasUsed:     4,
			Time:        5,
			BaseFee:     baseFeePerGas,
			Extra:       []byte("extraData"),
			MixDigest:   common.BytesToHash([]byte("mix")),
			Nonce:       gethtypes.EncodeNonce(6),
		}
		enc, err := json.Marshal(want)
		require.NoError(t, err)

		payloadItems := make(map[string]interface{})
		require.NoError(t, json.Unmarshal(enc, &payloadItems))

		tx := gethtypes.NewTransaction(
			1,
			common.BytesToAddress([]byte("hi")),
			big.NewInt(0),
			21000,
			big.NewInt(1e6),
			[]byte{},
		)
		txs := []*gethtypes.Transaction{tx}

		blockHash := want.Hash()
		payloadItems["hash"] = blockHash.String()
		payloadItems["totalDifficulty"] = "0x393a2e53de197c"
		payloadItems["transactions"] = txs

		encodedPayloadItems, err := json.Marshal(payloadItems)
		require.NoError(t, err)

		payloadPb := &enginev1.ExecutionBlock{}
		require.NoError(t, json.Unmarshal(encodedPayloadItems, payloadPb))

		require.DeepEqual(t, blockHash, payloadPb.Hash)
		require.DeepEqual(t, want.Number, payloadPb.Number)
		require.DeepEqual(t, want.ParentHash, payloadPb.ParentHash)
		require.DeepEqual(t, want.UncleHash, payloadPb.UncleHash)
		require.DeepEqual(t, want.Coinbase, payloadPb.Coinbase)
		require.DeepEqual(t, want.Root, payloadPb.Root)
		require.DeepEqual(t, want.TxHash, payloadPb.TxHash)
		require.DeepEqual(t, want.ReceiptHash, payloadPb.ReceiptHash)
		require.DeepEqual(t, want.Bloom, payloadPb.Bloom)
		require.DeepEqual(t, want.Difficulty, payloadPb.Difficulty)
		require.DeepEqual(t, payloadItems["totalDifficulty"], payloadPb.TotalDifficulty)
		require.DeepEqual(t, want.GasUsed, payloadPb.GasUsed)
		require.DeepEqual(t, want.GasLimit, payloadPb.GasLimit)
		require.DeepEqual(t, want.Time, payloadPb.Time)
		require.DeepEqual(t, want.BaseFee, payloadPb.BaseFee)
		require.DeepEqual(t, want.Extra, payloadPb.Extra)
		require.DeepEqual(t, want.MixDigest, payloadPb.MixDigest)
		require.DeepEqual(t, want.Nonce, payloadPb.Nonce)
		require.Equal(t, 1, len(payloadPb.Transactions))
		require.DeepEqual(t, txs[0].Hash(), payloadPb.Transactions[0].Hash())
	})
	t.Run("execution block with withdrawals", func(t *testing.T) {
		baseFeePerGas := big.NewInt(1770307273)
		want := &gethtypes.Header{
			Number:      big.NewInt(1),
			ParentHash:  common.BytesToHash([]byte("parent")),
			UncleHash:   common.BytesToHash([]byte("uncle")),
			Coinbase:    common.BytesToAddress([]byte("coinbase")),
			Root:        common.BytesToHash([]byte("uncle")),
			TxHash:      common.BytesToHash([]byte("txHash")),
			ReceiptHash: common.BytesToHash([]byte("receiptHash")),
			Bloom:       gethtypes.BytesToBloom([]byte("bloom")),
			Difficulty:  big.NewInt(2),
			GasLimit:    3,
			GasUsed:     4,
			Time:        5,
			BaseFee:     baseFeePerGas,
			Extra:       []byte("extraData"),
			MixDigest:   common.BytesToHash([]byte("mix")),
			Nonce:       gethtypes.EncodeNonce(6),
		}
		enc, err := json.Marshal(want)
		require.NoError(t, err)

		payloadItems := make(map[string]interface{})
		require.NoError(t, json.Unmarshal(enc, &payloadItems))

		blockHash := want.Hash()
		payloadItems["hash"] = blockHash.String()
		payloadItems["totalDifficulty"] = "0x393a2e53de197c"

		withdrawalIndex1 := hexutil.Uint64(1)
		withdrawalIndex2 := hexutil.Uint64(2)
		withdrawalAmount1 := hexutil.Uint64(100)
		withdrawalAmount2 := hexutil.Uint64(200)
		withdrawalValidator1 := hexutil.Uint64(1)
		withdrawalValidator2 := hexutil.Uint64(2)
		address1 := common.Address(bytesutil.ToBytes20([]byte("address1")))
		address2 := common.Address(bytesutil.ToBytes20([]byte("address2")))
		payloadItems["withdrawals"] = []*withdrawalJSON{
			{
				Index:     &withdrawalIndex1,
				Validator: &withdrawalValidator1,
				Address:   &address1,
				Amount:    &withdrawalAmount1,
			},
			{
				Index:     &withdrawalIndex2,
				Validator: &withdrawalValidator2,
				Address:   &address2,
				Amount:    &withdrawalAmount2,
			},
		}

		encodedPayloadItems, err := json.Marshal(payloadItems)
		require.NoError(t, err)

		payloadPb := &enginev1.ExecutionBlock{}
		require.NoError(t, json.Unmarshal(encodedPayloadItems, payloadPb))

		require.DeepEqual(t, blockHash, payloadPb.Hash)
		require.DeepEqual(t, want.Number, payloadPb.Number)
		require.DeepEqual(t, want.ParentHash, payloadPb.ParentHash)
		require.DeepEqual(t, want.UncleHash, payloadPb.UncleHash)
		require.DeepEqual(t, want.Coinbase, payloadPb.Coinbase)
		require.DeepEqual(t, want.Root, payloadPb.Root)
		require.DeepEqual(t, want.TxHash, payloadPb.TxHash)
		require.DeepEqual(t, want.ReceiptHash, payloadPb.ReceiptHash)
		require.DeepEqual(t, want.Bloom, payloadPb.Bloom)
		require.DeepEqual(t, want.Difficulty, payloadPb.Difficulty)
		require.DeepEqual(t, payloadItems["totalDifficulty"], payloadPb.TotalDifficulty)
		require.DeepEqual(t, want.GasUsed, payloadPb.GasUsed)
		require.DeepEqual(t, want.GasLimit, payloadPb.GasLimit)
		require.DeepEqual(t, want.Time, payloadPb.Time)
		require.DeepEqual(t, want.BaseFee, payloadPb.BaseFee)
		require.DeepEqual(t, want.Extra, payloadPb.Extra)
		require.DeepEqual(t, want.MixDigest, payloadPb.MixDigest)
		require.DeepEqual(t, want.Nonce, payloadPb.Nonce)
		require.Equal(t, 2, len(payloadPb.Withdrawals))
		require.Equal(t, uint64(1), payloadPb.Withdrawals[0].Index)
		require.Equal(t, primitives.ValidatorIndex(1), payloadPb.Withdrawals[0].ValidatorIndex)
		require.DeepEqual(t, bytesutil.PadTo([]byte("address1"), 20), payloadPb.Withdrawals[0].Address)
		require.Equal(t, uint64(100), payloadPb.Withdrawals[0].Amount)
		require.Equal(t, uint64(2), payloadPb.Withdrawals[1].Index)
		require.Equal(t, primitives.ValidatorIndex(2), payloadPb.Withdrawals[1].ValidatorIndex)
		require.DeepEqual(t, bytesutil.PadTo([]byte("address2"), 20), payloadPb.Withdrawals[1].Address)
		require.Equal(t, uint64(200), payloadPb.Withdrawals[1].Amount)
	})
	t.Run("execution block with deneb blob data", func(t *testing.T) {
		baseFeePerGas := big.NewInt(1770307273)
		blobGas := uint64(3000)
		excessGas := uint64(7000)
		want := &gethtypes.Header{
			Number:        big.NewInt(1),
			ParentHash:    common.BytesToHash([]byte("parent")),
			UncleHash:     common.BytesToHash([]byte("uncle")),
			Coinbase:      common.BytesToAddress([]byte("coinbase")),
			Root:          common.BytesToHash([]byte("uncle")),
			TxHash:        common.BytesToHash([]byte("txHash")),
			ReceiptHash:   common.BytesToHash([]byte("receiptHash")),
			Bloom:         gethtypes.BytesToBloom([]byte("bloom")),
			Difficulty:    big.NewInt(2),
			GasLimit:      3,
			GasUsed:       4,
			Time:          5,
			BaseFee:       baseFeePerGas,
			Extra:         []byte("extraData"),
			MixDigest:     common.BytesToHash([]byte("mix")),
			Nonce:         gethtypes.EncodeNonce(6),
			BlobGasUsed:   &blobGas,
			ExcessBlobGas: &excessGas,
		}
		enc, err := json.Marshal(want)
		require.NoError(t, err)

		payloadItems := make(map[string]interface{})
		require.NoError(t, json.Unmarshal(enc, &payloadItems))

		blockHash := want.Hash()
		payloadItems["hash"] = blockHash.String()
		payloadItems["totalDifficulty"] = "0x393a2e53de197c"
		payloadItems["transactions"] = []string{"0xd57870623ea84ac3e2ffafbee9417fd1263b825b1107b8d606c25460dabeb693"}

		encodedPayloadItems, err := json.Marshal(payloadItems)
		require.NoError(t, err)

		payloadPb := &enginev1.ExecutionBlock{}
		require.NoError(t, json.Unmarshal(encodedPayloadItems, payloadPb))

		require.DeepEqual(t, blockHash, payloadPb.Hash)
		require.DeepEqual(t, want.Number, payloadPb.Number)
		require.DeepEqual(t, want.ParentHash, payloadPb.ParentHash)
		require.DeepEqual(t, want.UncleHash, payloadPb.UncleHash)
		require.DeepEqual(t, want.Coinbase, payloadPb.Coinbase)
		require.DeepEqual(t, want.Root, payloadPb.Root)
		require.DeepEqual(t, want.TxHash, payloadPb.TxHash)
		require.DeepEqual(t, want.ReceiptHash, payloadPb.ReceiptHash)
		require.DeepEqual(t, want.Bloom, payloadPb.Bloom)
		require.DeepEqual(t, want.Difficulty, payloadPb.Difficulty)
		require.DeepEqual(t, payloadItems["totalDifficulty"], payloadPb.TotalDifficulty)
		require.DeepEqual(t, want.GasUsed, payloadPb.GasUsed)
		require.DeepEqual(t, want.GasLimit, payloadPb.GasLimit)
		require.DeepEqual(t, want.Time, payloadPb.Time)
		require.DeepEqual(t, want.BaseFee, payloadPb.BaseFee)
		require.DeepEqual(t, want.Extra, payloadPb.Extra)
		require.DeepEqual(t, want.MixDigest, payloadPb.MixDigest)
		require.DeepEqual(t, want.Nonce, payloadPb.Nonce)
		require.DeepEqual(t, want.BlobGasUsed, payloadPb.BlobGasUsed)
		require.DeepEqual(t, want.ExcessBlobGas, payloadPb.ExcessBlobGas)

		// Expect no transaction objects in the unmarshaled data.
		require.Equal(t, 0, len(payloadPb.Transactions))
	})
}

func TestPayloadIDBytes_MarshalUnmarshalJSON(t *testing.T) {
	item := [8]byte{1, 0, 0, 0, 0, 0, 0, 0}
	enc, err := json.Marshal(enginev1.PayloadIDBytes(item))
	require.NoError(t, err)
	require.DeepEqual(t, "\"0x0100000000000000\"", string(enc))
	res := &enginev1.PayloadIDBytes{}
	err = res.UnmarshalJSON(enc)
	require.NoError(t, err)
	require.Equal(t, true, item == *res)
}

func TestExecutionPayloadBody_MarshalUnmarshalJSON(t *testing.T) {
	pBody := &enginev1.ExecutionPayloadBodyV1{
		Transactions: [][]byte{[]byte("random1"), []byte("random2"), []byte("random3")},
		Withdrawals: []*enginev1.Withdrawal{
			{
				Index:          200,
				ValidatorIndex: 20303,
				Amount:         3200000000,
				Address:        bytesutil.PadTo([]byte("junk"), 20),
			},
			{
				Index:          200,
				ValidatorIndex: 70303,
				Amount:         3200000800,
				Address:        bytesutil.PadTo([]byte("junk2"), 20),
			},
		},
	}
	enc, err := json.Marshal(pBody)
	require.NoError(t, err)
	res := &enginev1.ExecutionPayloadBodyV1{}
	err = res.UnmarshalJSON(enc)
	require.NoError(t, err)
	require.DeepEqual(t, pBody, res)
}

func TestExecutionBlock_MarshalUnmarshalJSON_MainnetBlock(t *testing.T) {
	newBlock := &enginev1.ExecutionBlock{}
	require.NoError(t, newBlock.UnmarshalJSON([]byte(blockJson)))
	_, err := newBlock.MarshalJSON()
	require.NoError(t, err)

	newBlock = &enginev1.ExecutionBlock{}
	require.NoError(t, newBlock.UnmarshalJSON([]byte(blockNoTxJson)))
	_, err = newBlock.MarshalJSON()
	require.NoError(t, err)
}

var blockJson = `{"baseFeePerGas":"0x42110b4f7","difficulty":"0x280ae66012087c","extraData":"0xe4b883e5bda9e7a59ee4bb99e9b1bc4b3021","gasLimit":"0x1c9c380","gasUsed":"0xf829e","hash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","logsBloom":"0x002000000010100110000000800008200000000000000000000020001000200000040104000000000000101000000100820080800800080000a008000a01200000000000000001202042000c000000200841000000002001200004008000102002000000000200000000010440000042000000000000080000000010001000002000020000020000000000000000000002000001000010080020004008100000880001080000400000004080060200000800010000040002204000000000020000000002000000000000000001000008000000400000001002010804000000000020a40800000000070000000401080000000000000880400000000000001000","miner":"0x829bd824b016326a401d083b33d092293333a830","mixHash":"0xc1bcfb6dc83cdc106faad9870ab697dd6c7a5a05ca00b3a5f3c2e021b22e0747","nonce":"0xf09ffce459ff4a07","number":"0xe6f8db","parentHash":"0x5749469a59b1207d4b6d42dd9e31c059aa1586fe070573bf6e5442a626726959","receiptsRoot":"0x3b131e70a5d2e013c5946d6bf0290732ad1d195b05abd72bc0bfb7ed4be202b0","sha3Uncles":"0x4df8516d92fd18ca040f0af06d31afaa3a62dbc6ec7ec758336c81b719782a07","size":"0x18ad","stateRoot":"0xdff0d06049e5a7d5b4249eb2aa4b7c626f7a957733913786912441b89d20a3e1","timestamp":"0x62cf48c6","totalDifficulty":"0xb6c08f1eb97fd70fc5f","transactions":[{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0x10121cb2b3f64f0a6231178336aca3e3b87d5ca5","gas":"0x222e0","gasPrice":"0x6be56a00f","hash":"0x7d503dbb3661532e9bf51a23eeb284bb0d3a1cb99212108ceae70730a2617d7c","input":"0xb31c01fb66054fe7e80881e2dfed6bdd67d09c6a50461013b2ff4b3e9684f57fb58a9f07543c63a826a769aad2d6e3bfacdda2a930f25782caeeb3b6a66c7e6cc5a4811c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000419bb97c858f8c9d2ca3cf28f0236e15fa68a74c4263c28baecd00f603690dbf1c17bf2f4ad0767dbb92118e479b7a716ed465ed27a5b7decbcf9ba5cc1e911ae41b00000000000000000000000000000000000000000000000000000000000000","nonce":"0xc5f","to":"0x049b51e531fd8f90da6d92ea83dc4125002f20ef","transactionIndex":"0x0","value":"0x0","type":"0x0","v":"0x25","r":"0x8cb6e54a332bce463b2184ff252c35d400b5548fb5d5e1a711bf64d6bec5cd55","s":"0x42d5c57f90f5394814b10f1046e4188eebb72aff7f9c8e838b28159b9c47985f"},{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0xc8231eb0f6be12cca4e8de38fbd36382f827b615","gas":"0x33f9d","gasPrice":"0x4b613adf7","maxFeePerGas":"0x8dffb706a","maxPriorityFeePerGas":"0x9502f900","hash":"0x3a3d2c7624c0029d4865ca8e92ff737d971bcee393a22f4e231a801774ae5cda","input":"0xfb0f3ee100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000393bf5ab54e000000000000000000000000000e476199b37e70258d144a53d9522747c9d9cc82b000000000000000000000000004c00500000ad104d7dbd00e3ae0a5c00560c00000000000000000000000000dcaf23e44639daf29f6532da213999d737f15aa40000000000000000000000000000000000000000000000000000000000000937000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000062cf47430000000000000000000000000000000000000000000000000000000062f81edd00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000120bbba61bdc2df0000007b02230091a7ed01230072f7006a004d60a8d4e71d599b8104250f00000000007b02230091a7ed01230072f7006a004d60a8d4e71d599b8104250f00000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000196ffb68978000000000000000000000000008de9c5a032463c561423387a9648c5c7bcc5bc900000000000000000000000000000000000000000000000000004c4ff239c68000000000000000000000000002fef5a3fc423ab959a0d6e0f2316585a307aa9de000000000000000000000000000000000000000000000000000000000000004109c1e7267910fca7cfce18df320025d41a37b5341da36ad7c353f0bab91615e84022be07f890a9f05e739552b734a13b76b700cda759f922023f2d644a0238b71b00000000000000000000000000000000000000000000000000000000000000","nonce":"0x11f","to":"0x00000000006c3852cbef3e08e8df289169ede581","transactionIndex":"0x1","value":"0x3f97f4857ac000","type":"0x2","accessList":[],"chainId":"0x1","v":"0x0","r":"0x63a55120fd87fa8f84c8f888f37da83213e25abbe01f2690573d34e0e541ca6a","s":"0x47eb2a411538bb8b03e6a4fe8ddbe039888d73a0f45f26ecebd07d2069b62ab3"},{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0x84fa4d36d7bca1b7e69997ed812fb4d26c3a98ad","gas":"0xb416","gasPrice":"0x4b613adf7","maxFeePerGas":"0x95b3ec9ca","maxPriorityFeePerGas":"0x9502f900","hash":"0xe0bd91c32bc87146514a64f2cea7528a9d4e73d847a7ca03667a503cf52ba2cb","input":"0xa22cb4650000000000000000000000001e0049783f008a0085193e00003d00cd54003c710000000000000000000000000000000000000000000000000000000000000001","nonce":"0xed","to":"0xdcaf23e44639daf29f6532da213999d737f15aa4","transactionIndex":"0x2","value":"0x0","type":"0x2","accessList":[],"chainId":"0x1","v":"0x0","r":"0x7bc697c3731db3d308c79dd0c8e2cbfdae7d347a189faaa79274677786c2898","s":"0x611b6c480f08bd964c2f6c923f2d73b95d23360d203d40160e829b377b3801d0"},{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0xe1997c479a35ca8f6e3a5343ff866490b63debcf","gas":"0x68e6f","gasPrice":"0x4b1922547","maxFeePerGas":"0x6840297ff","maxPriorityFeePerGas":"0x90817050","hash":"0x843f21fe25a934099f6f311665d1e211ff09d4dc8de02b589ddf6eac74d3dfcb","input":"0x00e05147921005000000000000000000000064c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000023b872dd000000000000000000000000dfee68a9adb981cd08699891a11cabe10f25ec4400000000000000000000000012d4444f96c644385d8ab355f6ddf801315b625400000000000000000000000000000000000000000000000006b5a75ea8072000008412d4444f96c644385d8ab355f6ddf801315b625400000000000000000000022c0d9f00000000000000000000000000000000000000000000005093f4dbb5636ab8fa00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000007f150bd6f54c40a34d7c3d5e9f56000000000000000000000000000000000000000000000000000000000000002000a426607ac599266b21d13c7acf7942c7701a8b699c000000000000000000008201aa3f00000000000000000000000038e4adb44ef08f22f5b5b76a8f0c2d0dcbe7dca100000000000000000000000000000000000000000000005093f4dbb5614400000000000000000000000000001f9840a85d5af5bf1d1762f925bdaddc4201f9840000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000441f9840a85d5af5bf1d1762f925bdaddc4201f98400000000000000000000a9059cbb000000000000000000000000d3d2e2692501a5c9ca623199d38826e513033a17000000000000000000000000000000000000000000000004e0f33ca8f698c0000084d3d2e2692501a5c9ca623199d38826e513033a1700000000000000000000022c0d9f000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006ce00ae782d5d8b000000000000000000000000dfee68a9adb981cd08699891a11cabe10f25ec440000000000000000000000000000000000000000000000000000000000000000","nonce":"0x3358","to":"0x70526cc7a6d6320b44122ea9d2d07670accc85a1","transactionIndex":"0x3","value":"0xe6f8e2","type":"0x2","accessList":[],"chainId":"0x1","v":"0x1","r":"0xffe11c5dbdf42635610d9fa85774c2d95a37494962d1e3302c0fd5eac27f4147","s":"0x10282aa75d129b7b9afc04cf2c43061c259d7e498b9da36db927b67b85282938"},{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0xa4aa741c4db3eb5da2b616ee8f5c37cc562f47b9","gas":"0xaae60","gasPrice":"0x4a817c800","hash":"0xbf084d9e3a885bce9a27902aa394f572a1d3382eea003a19393aed9eb5a20be2","input":"0x5c11d79500000000000000000000000000000000000000000000000000000000c3996afa000000000000000000000000000000000000000000000fd10c33512e420d8ae800000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000a4aa741c4db3eb5da2b616ee8f5c37cc562f47b90000000000000000000000000000000000000000000000000000000062cf49790000000000000000000000000000000000000000000000000000000000000003000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec7000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000eca82185adce47f39c684352b0439f030f860318","nonce":"0x206","to":"0x7a250d5630b4cf539739df2c5dacb4c659f2488d","transactionIndex":"0x4","value":"0x0","type":"0x0","v":"0x26","r":"0x54f90db092a44f470697044232932f82e7e06b5f219df61adc99a93f8e263fbf","s":"0x26bc668d456289b0bd1d5b4f13b47536aae2637cb86e93bf0f819dae92fd31f9"},{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0x6f730c548c6d75e16971a619a2bc7a1f2539aa54","gas":"0x75300","gasPrice":"0x4a817c800","hash":"0x388fc716a00c94beae24f7e0b52aad43ac34060733890e9ea286273c7787a676","input":"0x0100000000000000000000000000000000000000000000000000000566c592169c9425d89b8d2834ba1b3c31688e084ce9792baa0ca2e2f700020e8c7769f9f1e5042c0809b8702e4b9947b1bcb3f3eca82185adce47f39c684352b0439f030f860318009b8d2834ba1b3c31688e084ce9792baa0ca2e2f7c02aaa39b223fe8d0a0e5c4f27ead9083c756cc226f200000000000000000000081e574f5e3f900000000000","nonce":"0x2080","to":"0x00000000000a47b1298f18cf67de547bbe0d723f","transactionIndex":"0x5","value":"0x0","type":"0x0","v":"0x25","r":"0x6364f53f1fe7ac58eaa6fff7ad06e920ef44c719f6068a9e8ed82b7b74ecd925","s":"0x669580d83fad57644779f91c7b8d1c8c7fa115a4b4a26c55b52d9ce690e1e125"},{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0x3cd751e6b0078be393132286c442345e5dc49699","gas":"0x3d090","gasPrice":"0x4984648f7","maxFeePerGas":"0x9502f9000","maxPriorityFeePerGas":"0x77359400","hash":"0xcf0e55b95af41c681d92a249a92f0aef8f023da25799efd7442b5c3ef6a52de6","input":"0xa9059cbb000000000000000000000000c4b0a24215df960dba4eee4a9519e9b69a55f747000000000000000000000000000000000000000000000000000000003a6c736d","nonce":"0x7fd10b","to":"0xdac17f958d2ee523a2206206994597c13d831ec7","transactionIndex":"0x6","value":"0x0","type":"0x2","accessList":[],"chainId":"0x1","v":"0x0","r":"0xb901f8b46ebe10c26b07f3bdbf34680c3336dcfd7b8c7e85244a7f11b0fed33a","s":"0x67d41039a1c510aaec712147287ff203c842f21b033bc898640bf5ad488d3897"},{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0xef9c8b0cf43e24b421111ca7ea82aca211ae04a7","gas":"0x493e0","gasPrice":"0x4984648f7","maxFeePerGas":"0xbaeb6d514","maxPriorityFeePerGas":"0x77359400","hash":"0xa94eaf385588e9596a61851a1d25b0a0007c0e565ad4112bc7d0e91f83888cda","input":"0xc18a84bc0000000000000000000000004f7ec9be30514129e6f672a7f6517445194755d2000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000445db3b4df000000000000000000000000eca82185adce47f39c684352b0439f030f8603180000000000000000000000000000000000000000000034f086f3b33b6840000000000000000000000000000000000000000000000000000000000000","nonce":"0x33a2","to":"0x000000000dfde7deaf24138722987c9a6991e2d4","transactionIndex":"0x7","value":"0x0","type":"0x2","accessList":[],"chainId":"0x1","v":"0x1","r":"0x469ff733bdab6c6cb2cbd60160e7a61b1afb7d573caa2c118f712d55e785d4c","s":"0x1f30e48ab9af25160616a201084c136eedd1ec5b59d8e4fd901776cf1ea8f020"},{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0x5c82929442529e67f9ebd9ed75854db7a5cd1755","gas":"0x5208","gasPrice":"0x4984648f7","maxFeePerGas":"0x8d8f9fc00","maxPriorityFeePerGas":"0x77359400","hash":"0xb360475e21e44e4d6b982387347c099ea8f2305773724db273128bbfdf82a1db","input":"0x","nonce":"0x1","to":"0xa090e606e30bd747d4e6245a1517ebe430f0057e","transactionIndex":"0x8","value":"0x21f4d6c5481103","type":"0x2","accessList":[],"chainId":"0x1","v":"0x0","r":"0x124e2c0f3773f6edded4530a2ccc68904fe0c7eb5932bbe22c5521ceb0e8b483","s":"0x32de5f21b3f52ac2141702c34fda2a05db1985e0ebb6b10f8606810dec6bfeaf"},{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0xad16a383bc802448659759ef40c4d1a6dbae87f7","gas":"0x40070","gasPrice":"0x49537f593","maxFeePerGas":"0x990282d92","maxPriorityFeePerGas":"0x7427409c","hash":"0xa95eba47cc617f16fa00735bd75cc245511e77c08efa8155ece7e59004265c2f","input":"0x5f5755290000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b1a2bc2ec5000000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000000c307846656544796e616d696300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000000000000000000000000000000021bfbda47a0b4b5b1248c767ee49f7caa9b2369700000000000000000000000000000000000000000000000000b014d4c6ae2800000000000000000000000000000000000000000000000003a4bfea6ceb020814000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000018de76816d800000000000000000000000000f326e4de8f66a0bdc0970b79e0924e33c79f191500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000128d9627aa4000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000b014d4c6ae2800000000000000000000000000000000000000000000000003a4bfea6ceb02081400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee00000000000000000000000021bfbda47a0b4b5b1248c767ee49f7caa9b23697869584cd00000000000000000000000011ededebf63bef0ea2d2d071bdf88f71543ec6fb0000000000000000000000000000000000000000000000d47be81e1a62cf484a00000000000000000000000000000000000000000000000066","nonce":"0xa","to":"0x881d40237659c251811cec9c364ef91dc08d300c","transactionIndex":"0x9","value":"0xb1a2bc2ec50000","type":"0x2","accessList":[],"chainId":"0x1","v":"0x1","r":"0xca99f35d497e33b60931750042c0c4697111eabb614242dc377b797cb376b46e","s":"0x69add212848c84f77b65ef5f1da1587f5b28c518d33b9b19b6b9264270bdf338"},{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0xc0868faeb27919a11425706a43ff428957d32d0c","gas":"0x5208","gasPrice":"0x47a78e3f7","maxFeePerGas":"0x5f2697f9b","maxPriorityFeePerGas":"0x59682f00","hash":"0xb7ca5adc1ba774c31d551d04aad1fb3c63729fdffe39d8cadf7305413df22f4c","input":"0x","nonce":"0x4","to":"0xe36338c1b2c10969a3e4ee93c11a45d7c1db3352","transactionIndex":"0xa","value":"0x4299a9ffe9fdd8","type":"0x2","accessList":[],"chainId":"0x1","v":"0x0","r":"0xccb0f44ecd8ccacf71d44cc453ff17b9f95f1c0708964ada03fc97641593d7c9","s":"0x5dcb93d823ccca457cf2d7cdfc665d78274b247e613b7d76f4bb4a571802f1fa"},{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0x48ddf6d748aed851a19aa33916b3d05f179a18d5","gas":"0x15526","gasPrice":"0x47a78e3f7","maxFeePerGas":"0x71a4db10c","maxPriorityFeePerGas":"0x59682f00","hash":"0xa27ccc3bf5dca531769c79795dc74ffeb1161963eeeebaa7ef365303b47b697d","input":"0xa9059cbb00000000000000000000000014060719865a0b03c04f53e7adb71538ca35082a00000000000000000000000000000000000000000000009770d9e7181a3bfec4","nonce":"0x111","to":"0x362bc847a3a9637d3af6624eec853618a43ed7d2","transactionIndex":"0xb","value":"0x0","type":"0x2","accessList":[],"chainId":"0x1","v":"0x0","r":"0x1b9175ce5746c7ec73c8fe1cdccde8871a3be014820ac0d2b961571384fe3d15","s":"0x403ea6a5fd39d28466fad064395d8be7aba9791b5ebbfaf2168367b8787e673"},{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0x14e323aa3c00e0cb64c8ba8a392290a480a81357","gas":"0x5208","gasPrice":"0x47a78e3f7","maxFeePerGas":"0x5f2697f9b","maxPriorityFeePerGas":"0x59682f00","hash":"0x42bfe585b3c4974206570b01e01e904ad8e3be8f6ae021acf645116549ef56b3","input":"0x","nonce":"0x1","to":"0x1128b435be2968c9d14b737ed4c4fc89fd89c6d1","transactionIndex":"0xc","value":"0x1fac9f0fb4d6dbc","type":"0x2","accessList":[],"chainId":"0x1","v":"0x1","r":"0xec4c1e4213a06a165b75368fb4c1b80f158f60b0b745ee78785cf613b3931eb1","s":"0x694e4ebbc4cc7df2c03549e19cafa389aced49fc115564c02990c0c8d698e120"},{"blockHash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","blockNumber":"0xe6f8db","from":"0x50270a9a29899eea6f485767fbc819b0b35f8702","gas":"0x5208","gasPrice":"0x47a78e3f7","maxFeePerGas":"0x6459d5bef","maxPriorityFeePerGas":"0x59682f00","hash":"0x03d033a7910eb2b5023ef9102805c06e30449b9926af32b47c6de3f5ccf45634","input":"0x","nonce":"0x0","to":"0x9218d124ad69378c0ebc2a4c7a219fda921d262b","transactionIndex":"0xd","value":"0x2901819154accd8","type":"0x2","accessList":[],"chainId":"0x1","v":"0x0","r":"0xe76cbf6256edb2c5b46c66d61820d99f6350a8cfa329a9b791c2c5fb18546ff9","s":"0x23827bd4c5c872ed6416f98bf7661fd5db168ec91095bd60e62a6a104294357b"}],"transactionsRoot":"0x46e27176677a4b37c1fa9bae97ffb48b86a316f9e6568b3320e10dd6954b5d1a","uncles":["0x0b15f885d283bb8044350ccb9b88fa42192926abb41302fefe0179051e4deadb"]}`
var blockNoTxJson = `{"baseFeePerGas":"0x42110b4f7","difficulty":"0x280ae66012087c","extraData":"0xe4b883e5bda9e7a59ee4bb99e9b1bc4b3021","gasLimit":"0x1c9c380","gasUsed":"0xf829e","hash":"0xf5bda634715a9d8af2693b600a725a0db285f0267f25b7f60f5b9c502691aef8","logsBloom":"0x002000000010100110000000800008200000000000000000000020001000200000040104000000000000101000000100820080800800080000a008000a01200000000000000001202042000c000000200841000000002001200004008000102002000000000200000000010440000042000000000000080000000010001000002000020000020000000000000000000002000001000010080020004008100000880001080000400000004080060200000800010000040002204000000000020000000002000000000000000001000008000000400000001002010804000000000020a40800000000070000000401080000000000000880400000000000001000","miner":"0x829bd824b016326a401d083b33d092293333a830","mixHash":"0xc1bcfb6dc83cdc106faad9870ab697dd6c7a5a05ca00b3a5f3c2e021b22e0747","nonce":"0xf09ffce459ff4a07","number":"0xe6f8db","parentHash":"0x5749469a59b1207d4b6d42dd9e31c059aa1586fe070573bf6e5442a626726959","receiptsRoot":"0x3b131e70a5d2e013c5946d6bf0290732ad1d195b05abd72bc0bfb7ed4be202b0","sha3Uncles":"0x4df8516d92fd18ca040f0af06d31afaa3a62dbc6ec7ec758336c81b719782a07","size":"0x18ad","stateRoot":"0xdff0d06049e5a7d5b4249eb2aa4b7c626f7a957733913786912441b89d20a3e1","timestamp":"0x62cf48c6","totalDifficulty":"0xb6c08f1eb97fd70fc5f","transactions":["0x7d503dbb3661532e9bf51a23eeb284bb0d3a1cb99212108ceae70730a2617d7c","0x3a3d2c7624c0029d4865ca8e92ff737d971bcee393a22f4e231a801774ae5cda","0xe0bd91c32bc87146514a64f2cea7528a9d4e73d847a7ca03667a503cf52ba2cb","0x843f21fe25a934099f6f311665d1e211ff09d4dc8de02b589ddf6eac74d3dfcb","0xbf084d9e3a885bce9a27902aa394f572a1d3382eea003a19393aed9eb5a20be2","0x388fc716a00c94beae24f7e0b52aad43ac34060733890e9ea286273c7787a676","0xcf0e55b95af41c681d92a249a92f0aef8f023da25799efd7442b5c3ef6a52de6","0xa94eaf385588e9596a61851a1d25b0a0007c0e565ad4112bc7d0e91f83888cda","0xb360475e21e44e4d6b982387347c099ea8f2305773724db273128bbfdf82a1db","0xa95eba47cc617f16fa00735bd75cc245511e77c08efa8155ece7e59004265c2f","0xb7ca5adc1ba774c31d551d04aad1fb3c63729fdffe39d8cadf7305413df22f4c","0xa27ccc3bf5dca531769c79795dc74ffeb1161963eeeebaa7ef365303b47b697d","0x42bfe585b3c4974206570b01e01e904ad8e3be8f6ae021acf645116549ef56b3","0x03d033a7910eb2b5023ef9102805c06e30449b9926af32b47c6de3f5ccf45634"],"transactionsRoot":"0x46e27176677a4b37c1fa9bae97ffb48b86a316f9e6568b3320e10dd6954b5d1a","uncles":["0x0b15f885d283bb8044350ccb9b88fa42192926abb41302fefe0179051e4deadb"]}`
