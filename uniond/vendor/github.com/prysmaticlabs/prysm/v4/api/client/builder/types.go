package builder

import (
	"encoding/json"
	"fmt"
	"math/big"
	"strconv"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	types "github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/math"
	v1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

var errInvalidUint256 = errors.New("invalid Uint256")
var errDecodeUint256 = errors.New("unable to decode into Uint256")

// Uint256 a wrapper representation of big.Int
type Uint256 struct {
	*big.Int
}

func stringToUint256(s string) (Uint256, error) {
	bi := new(big.Int)
	_, ok := bi.SetString(s, 10)
	if !ok || !math.IsValidUint256(bi) {
		return Uint256{}, errors.Wrapf(errDecodeUint256, "value=%s", s)
	}
	return Uint256{Int: bi}, nil
}

// sszBytesToUint256 creates a Uint256 from a ssz-style (little-endian byte slice) representation.
func sszBytesToUint256(b []byte) (Uint256, error) {
	bi := bytesutil.LittleEndianBytesToBigInt(b)
	if !math.IsValidUint256(bi) {
		return Uint256{}, errors.Wrapf(errDecodeUint256, "value=%s", b)
	}
	return Uint256{Int: bi}, nil
}

// SSZBytes creates an ssz-style (little-endian byte slice) representation of the Uint256.
func (s Uint256) SSZBytes() []byte {
	if !math.IsValidUint256(s.Int) {
		return []byte{}
	}
	return bytesutil.PadTo(bytesutil.ReverseByteOrder(s.Int.Bytes()), 32)
}

// UnmarshalJSON takes in a byte array and unmarshals the value in Uint256
func (s *Uint256) UnmarshalJSON(t []byte) error {
	end := len(t)
	if len(t) < 2 {
		return errors.Errorf("provided Uint256 json string is too short: %s", string(t))
	}
	if t[0] != '"' || t[end-1] != '"' {
		return errors.Errorf("provided Uint256 json string is malformed: %s", string(t))
	}
	return s.UnmarshalText(t[1 : end-1])
}

// UnmarshalText takes in a byte array and unmarshals the text in Uint256
func (s *Uint256) UnmarshalText(t []byte) error {
	if s.Int == nil {
		s.Int = big.NewInt(0)
	}
	z, ok := s.SetString(string(t), 10)
	if !ok {
		return errors.Wrapf(errDecodeUint256, "value=%s", t)
	}
	if !math.IsValidUint256(z) {
		return errors.Wrapf(errDecodeUint256, "value=%s", t)
	}
	s.Int = z
	return nil
}

// MarshalJSON returns a json byte representation of Uint256.
func (s Uint256) MarshalJSON() ([]byte, error) {
	t, err := s.MarshalText()
	if err != nil {
		return nil, err
	}
	t = append([]byte{'"'}, t...)
	t = append(t, '"')
	return t, nil
}

// MarshalText returns a text byte representation of Uint256.
func (s Uint256) MarshalText() ([]byte, error) {
	if !math.IsValidUint256(s.Int) {
		return nil, errors.Wrapf(errInvalidUint256, "value=%s", s.Int)
	}
	return []byte(s.String()), nil
}

// Uint64String is a custom type that allows marshalling from text to uint64 and vice versa.
type Uint64String uint64

// UnmarshalText takes a byte array and unmarshals the text in Uint64String.
func (s *Uint64String) UnmarshalText(t []byte) error {
	u, err := strconv.ParseUint(string(t), 10, 64)
	*s = Uint64String(u)
	return err
}

// MarshalText returns a byte representation of the text from Uint64String.
func (s Uint64String) MarshalText() ([]byte, error) {
	return []byte(fmt.Sprintf("%d", s)), nil
}

// VersionResponse is a JSON representation of a field in the builder API header response.
type VersionResponse struct {
	Version string `json:"version"`
}

// ExecHeaderResponse is a JSON representation of the builder API header response for Bellatrix.
type ExecHeaderResponse struct {
	Version string `json:"version"`
	Data    struct {
		Signature hexutil.Bytes `json:"signature"`
		Message   *BuilderBid   `json:"message"`
	} `json:"data"`
}

// ToProto returns a SignedBuilderBid from ExecHeaderResponse for Bellatrix.
func (ehr *ExecHeaderResponse) ToProto() (*eth.SignedBuilderBid, error) {
	bb, err := ehr.Data.Message.ToProto()
	if err != nil {
		return nil, err
	}
	return &eth.SignedBuilderBid{
		Message:   bb,
		Signature: ehr.Data.Signature,
	}, nil
}

// ToProto returns a BuilderBid Proto for Bellatrix.
func (bb *BuilderBid) ToProto() (*eth.BuilderBid, error) {
	header, err := bb.Header.ToProto()
	if err != nil {
		return nil, err
	}
	return &eth.BuilderBid{
		Header: header,
		Value:  bb.Value.SSZBytes(),
		Pubkey: bb.Pubkey,
	}, nil
}

// ToProto returns a ExecutionPayloadHeader for Bellatrix.
func (h *ExecutionPayloadHeader) ToProto() (*v1.ExecutionPayloadHeader, error) {
	return &v1.ExecutionPayloadHeader{
		ParentHash:       bytesutil.SafeCopyBytes(h.ParentHash),
		FeeRecipient:     bytesutil.SafeCopyBytes(h.FeeRecipient),
		StateRoot:        bytesutil.SafeCopyBytes(h.StateRoot),
		ReceiptsRoot:     bytesutil.SafeCopyBytes(h.ReceiptsRoot),
		LogsBloom:        bytesutil.SafeCopyBytes(h.LogsBloom),
		PrevRandao:       bytesutil.SafeCopyBytes(h.PrevRandao),
		BlockNumber:      uint64(h.BlockNumber),
		GasLimit:         uint64(h.GasLimit),
		GasUsed:          uint64(h.GasUsed),
		Timestamp:        uint64(h.Timestamp),
		ExtraData:        bytesutil.SafeCopyBytes(h.ExtraData),
		BaseFeePerGas:    bytesutil.SafeCopyBytes(h.BaseFeePerGas.SSZBytes()),
		BlockHash:        bytesutil.SafeCopyBytes(h.BlockHash),
		TransactionsRoot: bytesutil.SafeCopyBytes(h.TransactionsRoot),
	}, nil
}

// BuilderBid is part of ExecHeaderResponse for Bellatrix.
type BuilderBid struct {
	Header *ExecutionPayloadHeader `json:"header"`
	Value  Uint256                 `json:"value"`
	Pubkey hexutil.Bytes           `json:"pubkey"`
}

// ExecutionPayloadHeader is a field in BuilderBid.
type ExecutionPayloadHeader struct {
	ParentHash       hexutil.Bytes `json:"parent_hash"`
	FeeRecipient     hexutil.Bytes `json:"fee_recipient"`
	StateRoot        hexutil.Bytes `json:"state_root"`
	ReceiptsRoot     hexutil.Bytes `json:"receipts_root"`
	LogsBloom        hexutil.Bytes `json:"logs_bloom"`
	PrevRandao       hexutil.Bytes `json:"prev_randao"`
	BlockNumber      Uint64String  `json:"block_number"`
	GasLimit         Uint64String  `json:"gas_limit"`
	GasUsed          Uint64String  `json:"gas_used"`
	Timestamp        Uint64String  `json:"timestamp"`
	ExtraData        hexutil.Bytes `json:"extra_data"`
	BaseFeePerGas    Uint256       `json:"base_fee_per_gas"`
	BlockHash        hexutil.Bytes `json:"block_hash"`
	TransactionsRoot hexutil.Bytes `json:"transactions_root"`
	*v1.ExecutionPayloadHeader
}

// MarshalJSON returns the JSON bytes representation of ExecutionPayloadHeader.
func (h *ExecutionPayloadHeader) MarshalJSON() ([]byte, error) {
	type MarshalCaller ExecutionPayloadHeader
	baseFeePerGas, err := sszBytesToUint256(h.ExecutionPayloadHeader.BaseFeePerGas)
	if err != nil {
		return []byte{}, errors.Wrapf(err, "invalid BaseFeePerGas")
	}
	return json.Marshal(&MarshalCaller{
		ParentHash:       h.ExecutionPayloadHeader.ParentHash,
		FeeRecipient:     h.ExecutionPayloadHeader.FeeRecipient,
		StateRoot:        h.ExecutionPayloadHeader.StateRoot,
		ReceiptsRoot:     h.ExecutionPayloadHeader.ReceiptsRoot,
		LogsBloom:        h.ExecutionPayloadHeader.LogsBloom,
		PrevRandao:       h.ExecutionPayloadHeader.PrevRandao,
		BlockNumber:      Uint64String(h.ExecutionPayloadHeader.BlockNumber),
		GasLimit:         Uint64String(h.ExecutionPayloadHeader.GasLimit),
		GasUsed:          Uint64String(h.ExecutionPayloadHeader.GasUsed),
		Timestamp:        Uint64String(h.ExecutionPayloadHeader.Timestamp),
		ExtraData:        h.ExecutionPayloadHeader.ExtraData,
		BaseFeePerGas:    baseFeePerGas,
		BlockHash:        h.ExecutionPayloadHeader.BlockHash,
		TransactionsRoot: h.ExecutionPayloadHeader.TransactionsRoot,
	})
}

// UnmarshalJSON takes in a JSON byte array and sets ExecutionPayloadHeader.
func (h *ExecutionPayloadHeader) UnmarshalJSON(b []byte) error {
	type UnmarshalCaller ExecutionPayloadHeader
	uc := &UnmarshalCaller{}
	if err := json.Unmarshal(b, uc); err != nil {
		return err
	}
	ep := ExecutionPayloadHeader(*uc)
	*h = ep
	var err error
	h.ExecutionPayloadHeader, err = h.ToProto()
	return err
}

// ExecPayloadResponse is the builder API /eth/v1/builder/blinded_blocks for Bellatrix.
type ExecPayloadResponse struct {
	Version string           `json:"version"`
	Data    ExecutionPayload `json:"data"`
}

// ExecutionPayload is a field of ExecPayloadResponse
type ExecutionPayload struct {
	ParentHash    hexutil.Bytes   `json:"parent_hash"`
	FeeRecipient  hexutil.Bytes   `json:"fee_recipient"`
	StateRoot     hexutil.Bytes   `json:"state_root"`
	ReceiptsRoot  hexutil.Bytes   `json:"receipts_root"`
	LogsBloom     hexutil.Bytes   `json:"logs_bloom"`
	PrevRandao    hexutil.Bytes   `json:"prev_randao"`
	BlockNumber   Uint64String    `json:"block_number"`
	GasLimit      Uint64String    `json:"gas_limit"`
	GasUsed       Uint64String    `json:"gas_used"`
	Timestamp     Uint64String    `json:"timestamp"`
	ExtraData     hexutil.Bytes   `json:"extra_data"`
	BaseFeePerGas Uint256         `json:"base_fee_per_gas"`
	BlockHash     hexutil.Bytes   `json:"block_hash"`
	Transactions  []hexutil.Bytes `json:"transactions"`
}

// ToProto returns a ExecutionPayload Proto from ExecPayloadResponse
func (r *ExecPayloadResponse) ToProto() (*v1.ExecutionPayload, error) {
	return r.Data.ToProto()
}

// ToProto returns a ExecutionPayload Proto
func (p *ExecutionPayload) ToProto() (*v1.ExecutionPayload, error) {
	txs := make([][]byte, len(p.Transactions))
	for i := range p.Transactions {
		txs[i] = bytesutil.SafeCopyBytes(p.Transactions[i])
	}
	return &v1.ExecutionPayload{
		ParentHash:    bytesutil.SafeCopyBytes(p.ParentHash),
		FeeRecipient:  bytesutil.SafeCopyBytes(p.FeeRecipient),
		StateRoot:     bytesutil.SafeCopyBytes(p.StateRoot),
		ReceiptsRoot:  bytesutil.SafeCopyBytes(p.ReceiptsRoot),
		LogsBloom:     bytesutil.SafeCopyBytes(p.LogsBloom),
		PrevRandao:    bytesutil.SafeCopyBytes(p.PrevRandao),
		BlockNumber:   uint64(p.BlockNumber),
		GasLimit:      uint64(p.GasLimit),
		GasUsed:       uint64(p.GasUsed),
		Timestamp:     uint64(p.Timestamp),
		ExtraData:     bytesutil.SafeCopyBytes(p.ExtraData),
		BaseFeePerGas: bytesutil.SafeCopyBytes(p.BaseFeePerGas.SSZBytes()),
		BlockHash:     bytesutil.SafeCopyBytes(p.BlockHash),
		Transactions:  txs,
	}, nil
}

// FromProto converts a proto execution payload type to our builder
// compatible payload type.
func FromProto(payload *v1.ExecutionPayload) (ExecutionPayload, error) {
	bFee, err := sszBytesToUint256(payload.BaseFeePerGas)
	if err != nil {
		return ExecutionPayload{}, err
	}
	txs := make([]hexutil.Bytes, len(payload.Transactions))
	for i := range payload.Transactions {
		txs[i] = bytesutil.SafeCopyBytes(payload.Transactions[i])
	}
	return ExecutionPayload{
		ParentHash:    bytesutil.SafeCopyBytes(payload.ParentHash),
		FeeRecipient:  bytesutil.SafeCopyBytes(payload.FeeRecipient),
		StateRoot:     bytesutil.SafeCopyBytes(payload.StateRoot),
		ReceiptsRoot:  bytesutil.SafeCopyBytes(payload.ReceiptsRoot),
		LogsBloom:     bytesutil.SafeCopyBytes(payload.LogsBloom),
		PrevRandao:    bytesutil.SafeCopyBytes(payload.PrevRandao),
		BlockNumber:   Uint64String(payload.BlockNumber),
		GasLimit:      Uint64String(payload.GasLimit),
		GasUsed:       Uint64String(payload.GasUsed),
		Timestamp:     Uint64String(payload.Timestamp),
		ExtraData:     bytesutil.SafeCopyBytes(payload.ExtraData),
		BaseFeePerGas: bFee,
		BlockHash:     bytesutil.SafeCopyBytes(payload.BlockHash),
		Transactions:  txs,
	}, nil
}

// FromProtoCapella converts a proto execution payload type for capella to our
// builder compatible payload type.
func FromProtoCapella(payload *v1.ExecutionPayloadCapella) (ExecutionPayloadCapella, error) {
	bFee, err := sszBytesToUint256(payload.BaseFeePerGas)
	if err != nil {
		return ExecutionPayloadCapella{}, err
	}
	txs := make([]hexutil.Bytes, len(payload.Transactions))
	for i := range payload.Transactions {
		txs[i] = bytesutil.SafeCopyBytes(payload.Transactions[i])
	}
	withdrawals := make([]Withdrawal, len(payload.Withdrawals))
	for i, w := range payload.Withdrawals {
		withdrawals[i] = Withdrawal{
			Index:          Uint256{Int: big.NewInt(0).SetUint64(w.Index)},
			ValidatorIndex: Uint256{Int: big.NewInt(0).SetUint64(uint64(w.ValidatorIndex))},
			Address:        bytesutil.SafeCopyBytes(w.Address),
			Amount:         Uint256{Int: big.NewInt(0).SetUint64(w.Amount)},
		}
	}
	return ExecutionPayloadCapella{
		ParentHash:    bytesutil.SafeCopyBytes(payload.ParentHash),
		FeeRecipient:  bytesutil.SafeCopyBytes(payload.FeeRecipient),
		StateRoot:     bytesutil.SafeCopyBytes(payload.StateRoot),
		ReceiptsRoot:  bytesutil.SafeCopyBytes(payload.ReceiptsRoot),
		LogsBloom:     bytesutil.SafeCopyBytes(payload.LogsBloom),
		PrevRandao:    bytesutil.SafeCopyBytes(payload.PrevRandao),
		BlockNumber:   Uint64String(payload.BlockNumber),
		GasLimit:      Uint64String(payload.GasLimit),
		GasUsed:       Uint64String(payload.GasUsed),
		Timestamp:     Uint64String(payload.Timestamp),
		ExtraData:     bytesutil.SafeCopyBytes(payload.ExtraData),
		BaseFeePerGas: bFee,
		BlockHash:     bytesutil.SafeCopyBytes(payload.BlockHash),
		Transactions:  txs,
		Withdrawals:   withdrawals,
	}, nil
}

func FromProtoDeneb(payload *v1.ExecutionPayloadDeneb) (ExecutionPayloadDeneb, error) {
	bFee, err := sszBytesToUint256(payload.BaseFeePerGas)
	if err != nil {
		return ExecutionPayloadDeneb{}, err
	}
	txs := make([]hexutil.Bytes, len(payload.Transactions))
	for i := range payload.Transactions {
		txs[i] = bytesutil.SafeCopyBytes(payload.Transactions[i])
	}
	withdrawals := make([]Withdrawal, len(payload.Withdrawals))
	for i, w := range payload.Withdrawals {
		withdrawals[i] = Withdrawal{
			Index:          Uint256{Int: big.NewInt(0).SetUint64(w.Index)},
			ValidatorIndex: Uint256{Int: big.NewInt(0).SetUint64(uint64(w.ValidatorIndex))},
			Address:        bytesutil.SafeCopyBytes(w.Address),
			Amount:         Uint256{Int: big.NewInt(0).SetUint64(w.Amount)},
		}
	}
	return ExecutionPayloadDeneb{
		ParentHash:    bytesutil.SafeCopyBytes(payload.ParentHash),
		FeeRecipient:  bytesutil.SafeCopyBytes(payload.FeeRecipient),
		StateRoot:     bytesutil.SafeCopyBytes(payload.StateRoot),
		ReceiptsRoot:  bytesutil.SafeCopyBytes(payload.ReceiptsRoot),
		LogsBloom:     bytesutil.SafeCopyBytes(payload.LogsBloom),
		PrevRandao:    bytesutil.SafeCopyBytes(payload.PrevRandao),
		BlockNumber:   Uint64String(payload.BlockNumber),
		GasLimit:      Uint64String(payload.GasLimit),
		GasUsed:       Uint64String(payload.GasUsed),
		Timestamp:     Uint64String(payload.Timestamp),
		ExtraData:     bytesutil.SafeCopyBytes(payload.ExtraData),
		BaseFeePerGas: bFee,
		BlockHash:     bytesutil.SafeCopyBytes(payload.BlockHash),
		Transactions:  txs,
		Withdrawals:   withdrawals,
		BlobGasUsed:   Uint64String(payload.BlobGasUsed),
		ExcessBlobGas: Uint64String(payload.ExcessBlobGas),
	}, nil
}

// ExecHeaderResponseCapella is the response of builder API /eth/v1/builder/header/{slot}/{parent_hash}/{pubkey} for Capella.
type ExecHeaderResponseCapella struct {
	Data struct {
		Signature hexutil.Bytes      `json:"signature"`
		Message   *BuilderBidCapella `json:"message"`
	} `json:"data"`
}

// ToProto returns a SignedBuilderBidCapella Proto from ExecHeaderResponseCapella.
func (ehr *ExecHeaderResponseCapella) ToProto() (*eth.SignedBuilderBidCapella, error) {
	bb, err := ehr.Data.Message.ToProto()
	if err != nil {
		return nil, err
	}
	return &eth.SignedBuilderBidCapella{
		Message:   bb,
		Signature: bytesutil.SafeCopyBytes(ehr.Data.Signature),
	}, nil
}

// ToProto returns a BuilderBidCapella Proto.
func (bb *BuilderBidCapella) ToProto() (*eth.BuilderBidCapella, error) {
	header, err := bb.Header.ToProto()
	if err != nil {
		return nil, err
	}
	return &eth.BuilderBidCapella{
		Header: header,
		Value:  bytesutil.SafeCopyBytes(bb.Value.SSZBytes()),
		Pubkey: bytesutil.SafeCopyBytes(bb.Pubkey),
	}, nil
}

// ToProto returns a ExecutionPayloadHeaderCapella Proto
func (h *ExecutionPayloadHeaderCapella) ToProto() (*v1.ExecutionPayloadHeaderCapella, error) {
	return &v1.ExecutionPayloadHeaderCapella{
		ParentHash:       bytesutil.SafeCopyBytes(h.ParentHash),
		FeeRecipient:     bytesutil.SafeCopyBytes(h.FeeRecipient),
		StateRoot:        bytesutil.SafeCopyBytes(h.StateRoot),
		ReceiptsRoot:     bytesutil.SafeCopyBytes(h.ReceiptsRoot),
		LogsBloom:        bytesutil.SafeCopyBytes(h.LogsBloom),
		PrevRandao:       bytesutil.SafeCopyBytes(h.PrevRandao),
		BlockNumber:      uint64(h.BlockNumber),
		GasLimit:         uint64(h.GasLimit),
		GasUsed:          uint64(h.GasUsed),
		Timestamp:        uint64(h.Timestamp),
		ExtraData:        bytesutil.SafeCopyBytes(h.ExtraData),
		BaseFeePerGas:    bytesutil.SafeCopyBytes(h.BaseFeePerGas.SSZBytes()),
		BlockHash:        bytesutil.SafeCopyBytes(h.BlockHash),
		TransactionsRoot: bytesutil.SafeCopyBytes(h.TransactionsRoot),
		WithdrawalsRoot:  bytesutil.SafeCopyBytes(h.WithdrawalsRoot),
	}, nil
}

// BuilderBidCapella is field of ExecHeaderResponseCapella.
type BuilderBidCapella struct {
	Header *ExecutionPayloadHeaderCapella `json:"header"`
	Value  Uint256                        `json:"value"`
	Pubkey hexutil.Bytes                  `json:"pubkey"`
}

// ExecutionPayloadHeaderCapella is a field in BuilderBidCapella.
type ExecutionPayloadHeaderCapella struct {
	ParentHash       hexutil.Bytes `json:"parent_hash"`
	FeeRecipient     hexutil.Bytes `json:"fee_recipient"`
	StateRoot        hexutil.Bytes `json:"state_root"`
	ReceiptsRoot     hexutil.Bytes `json:"receipts_root"`
	LogsBloom        hexutil.Bytes `json:"logs_bloom"`
	PrevRandao       hexutil.Bytes `json:"prev_randao"`
	BlockNumber      Uint64String  `json:"block_number"`
	GasLimit         Uint64String  `json:"gas_limit"`
	GasUsed          Uint64String  `json:"gas_used"`
	Timestamp        Uint64String  `json:"timestamp"`
	ExtraData        hexutil.Bytes `json:"extra_data"`
	BaseFeePerGas    Uint256       `json:"base_fee_per_gas"`
	BlockHash        hexutil.Bytes `json:"block_hash"`
	TransactionsRoot hexutil.Bytes `json:"transactions_root"`
	WithdrawalsRoot  hexutil.Bytes `json:"withdrawals_root"`
	*v1.ExecutionPayloadHeaderCapella
}

// MarshalJSON returns a JSON byte representation of ExecutionPayloadHeaderCapella.
func (h *ExecutionPayloadHeaderCapella) MarshalJSON() ([]byte, error) {
	type MarshalCaller ExecutionPayloadHeaderCapella
	baseFeePerGas, err := sszBytesToUint256(h.ExecutionPayloadHeaderCapella.BaseFeePerGas)
	if err != nil {
		return []byte{}, errors.Wrapf(err, "invalid BaseFeePerGas")
	}
	return json.Marshal(&MarshalCaller{
		ParentHash:       h.ExecutionPayloadHeaderCapella.ParentHash,
		FeeRecipient:     h.ExecutionPayloadHeaderCapella.FeeRecipient,
		StateRoot:        h.ExecutionPayloadHeaderCapella.StateRoot,
		ReceiptsRoot:     h.ExecutionPayloadHeaderCapella.ReceiptsRoot,
		LogsBloom:        h.ExecutionPayloadHeaderCapella.LogsBloom,
		PrevRandao:       h.ExecutionPayloadHeaderCapella.PrevRandao,
		BlockNumber:      Uint64String(h.ExecutionPayloadHeaderCapella.BlockNumber),
		GasLimit:         Uint64String(h.ExecutionPayloadHeaderCapella.GasLimit),
		GasUsed:          Uint64String(h.ExecutionPayloadHeaderCapella.GasUsed),
		Timestamp:        Uint64String(h.ExecutionPayloadHeaderCapella.Timestamp),
		ExtraData:        h.ExecutionPayloadHeaderCapella.ExtraData,
		BaseFeePerGas:    baseFeePerGas,
		BlockHash:        h.ExecutionPayloadHeaderCapella.BlockHash,
		TransactionsRoot: h.ExecutionPayloadHeaderCapella.TransactionsRoot,
		WithdrawalsRoot:  h.ExecutionPayloadHeaderCapella.WithdrawalsRoot,
	})
}

// UnmarshalJSON takes a JSON byte array and sets ExecutionPayloadHeaderCapella.
func (h *ExecutionPayloadHeaderCapella) UnmarshalJSON(b []byte) error {
	type UnmarshalCaller ExecutionPayloadHeaderCapella
	uc := &UnmarshalCaller{}
	if err := json.Unmarshal(b, uc); err != nil {
		return err
	}
	ep := ExecutionPayloadHeaderCapella(*uc)
	*h = ep
	var err error
	h.ExecutionPayloadHeaderCapella, err = h.ToProto()
	return err
}

// ExecPayloadResponseCapella is the builder API /eth/v1/builder/blinded_blocks for Capella.
type ExecPayloadResponseCapella struct {
	Version string                  `json:"version"`
	Data    ExecutionPayloadCapella `json:"data"`
}

// ExecutionPayloadCapella is a field of ExecPayloadResponseCapella.
type ExecutionPayloadCapella struct {
	ParentHash    hexutil.Bytes   `json:"parent_hash"`
	FeeRecipient  hexutil.Bytes   `json:"fee_recipient"`
	StateRoot     hexutil.Bytes   `json:"state_root"`
	ReceiptsRoot  hexutil.Bytes   `json:"receipts_root"`
	LogsBloom     hexutil.Bytes   `json:"logs_bloom"`
	PrevRandao    hexutil.Bytes   `json:"prev_randao"`
	BlockNumber   Uint64String    `json:"block_number"`
	GasLimit      Uint64String    `json:"gas_limit"`
	GasUsed       Uint64String    `json:"gas_used"`
	Timestamp     Uint64String    `json:"timestamp"`
	ExtraData     hexutil.Bytes   `json:"extra_data"`
	BaseFeePerGas Uint256         `json:"base_fee_per_gas"`
	BlockHash     hexutil.Bytes   `json:"block_hash"`
	Transactions  []hexutil.Bytes `json:"transactions"`
	Withdrawals   []Withdrawal    `json:"withdrawals"`
}

// ToProto returns a ExecutionPayloadCapella Proto.
func (r *ExecPayloadResponseCapella) ToProto() (*v1.ExecutionPayloadCapella, error) {
	return r.Data.ToProto()
}

// ToProto returns a ExecutionPayloadCapella Proto.
func (p *ExecutionPayloadCapella) ToProto() (*v1.ExecutionPayloadCapella, error) {
	txs := make([][]byte, len(p.Transactions))
	for i := range p.Transactions {
		txs[i] = bytesutil.SafeCopyBytes(p.Transactions[i])
	}
	withdrawals := make([]*v1.Withdrawal, len(p.Withdrawals))
	for i, w := range p.Withdrawals {
		withdrawals[i] = &v1.Withdrawal{
			Index:          w.Index.Uint64(),
			ValidatorIndex: types.ValidatorIndex(w.ValidatorIndex.Uint64()),
			Address:        bytesutil.SafeCopyBytes(w.Address),
			Amount:         w.Amount.Uint64(),
		}
	}
	return &v1.ExecutionPayloadCapella{
		ParentHash:    bytesutil.SafeCopyBytes(p.ParentHash),
		FeeRecipient:  bytesutil.SafeCopyBytes(p.FeeRecipient),
		StateRoot:     bytesutil.SafeCopyBytes(p.StateRoot),
		ReceiptsRoot:  bytesutil.SafeCopyBytes(p.ReceiptsRoot),
		LogsBloom:     bytesutil.SafeCopyBytes(p.LogsBloom),
		PrevRandao:    bytesutil.SafeCopyBytes(p.PrevRandao),
		BlockNumber:   uint64(p.BlockNumber),
		GasLimit:      uint64(p.GasLimit),
		GasUsed:       uint64(p.GasUsed),
		Timestamp:     uint64(p.Timestamp),
		ExtraData:     bytesutil.SafeCopyBytes(p.ExtraData),
		BaseFeePerGas: bytesutil.SafeCopyBytes(p.BaseFeePerGas.SSZBytes()),
		BlockHash:     bytesutil.SafeCopyBytes(p.BlockHash),
		Transactions:  txs,
		Withdrawals:   withdrawals,
	}, nil
}

// Withdrawal is a field of ExecutionPayloadCapella.
type Withdrawal struct {
	Index          Uint256       `json:"index"`
	ValidatorIndex Uint256       `json:"validator_index"`
	Address        hexutil.Bytes `json:"address"`
	Amount         Uint256       `json:"amount"`
}

// SignedBlindedBeaconBlockBellatrix is the request object for builder API /eth/v1/builder/blinded_blocks.
type SignedBlindedBeaconBlockBellatrix struct {
	*eth.SignedBlindedBeaconBlockBellatrix
}

// ProposerSlashing is a field in BlindedBeaconBlockBodyCapella.
type ProposerSlashing struct {
	*eth.ProposerSlashing
}

// MarshalJSON returns a JSON byte array representation of ProposerSlashing.
func (s *ProposerSlashing) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		SignedHeader1 *SignedBeaconBlockHeader `json:"signed_header_1"`
		SignedHeader2 *SignedBeaconBlockHeader `json:"signed_header_2"`
	}{
		SignedHeader1: &SignedBeaconBlockHeader{s.ProposerSlashing.Header_1},
		SignedHeader2: &SignedBeaconBlockHeader{s.ProposerSlashing.Header_2},
	})
}

// SignedBeaconBlockHeader is a field of ProposerSlashing.
type SignedBeaconBlockHeader struct {
	*eth.SignedBeaconBlockHeader
}

// MarshalJSON returns a JSON byte array representation of SignedBeaconBlockHeader.
func (h *SignedBeaconBlockHeader) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		Header    *BeaconBlockHeader `json:"message"`
		Signature hexutil.Bytes      `json:"signature"`
	}{
		Header:    &BeaconBlockHeader{h.SignedBeaconBlockHeader.Header},
		Signature: h.SignedBeaconBlockHeader.Signature,
	})
}

// BeaconBlockHeader is a field of SignedBeaconBlockHeader.
type BeaconBlockHeader struct {
	*eth.BeaconBlockHeader
}

// MarshalJSON returns a JSON byte array representation of BeaconBlockHeader.
func (h *BeaconBlockHeader) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		Slot          string        `json:"slot"`
		ProposerIndex string        `json:"proposer_index"`
		ParentRoot    hexutil.Bytes `json:"parent_root"`
		StateRoot     hexutil.Bytes `json:"state_root"`
		BodyRoot      hexutil.Bytes `json:"body_root"`
	}{
		Slot:          fmt.Sprintf("%d", h.BeaconBlockHeader.Slot),
		ProposerIndex: fmt.Sprintf("%d", h.BeaconBlockHeader.ProposerIndex),
		ParentRoot:    h.BeaconBlockHeader.ParentRoot,
		StateRoot:     h.BeaconBlockHeader.StateRoot,
		BodyRoot:      h.BeaconBlockHeader.BodyRoot,
	})
}

// IndexedAttestation is a field of AttesterSlashing.
type IndexedAttestation struct {
	*eth.IndexedAttestation
}

// MarshalJSON returns a JSON byte array representation of IndexedAttestation.
func (a *IndexedAttestation) MarshalJSON() ([]byte, error) {
	indices := make([]string, len(a.IndexedAttestation.AttestingIndices))
	for i := range a.IndexedAttestation.AttestingIndices {
		indices[i] = fmt.Sprintf("%d", a.AttestingIndices[i])
	}
	return json.Marshal(struct {
		AttestingIndices []string         `json:"attesting_indices"`
		Data             *AttestationData `json:"data"`
		Signature        hexutil.Bytes    `json:"signature"`
	}{
		AttestingIndices: indices,
		Data:             &AttestationData{a.IndexedAttestation.Data},
		Signature:        a.IndexedAttestation.Signature,
	})
}

// AttesterSlashing is a field of a Beacon Block Body.
type AttesterSlashing struct {
	*eth.AttesterSlashing
}

// MarshalJSON returns a JSON byte array representation of AttesterSlashing.
func (s *AttesterSlashing) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		Attestation1 *IndexedAttestation `json:"attestation_1"`
		Attestation2 *IndexedAttestation `json:"attestation_2"`
	}{
		Attestation1: &IndexedAttestation{s.Attestation_1},
		Attestation2: &IndexedAttestation{s.Attestation_2},
	})
}

// Checkpoint is a field of AttestationData.
type Checkpoint struct {
	*eth.Checkpoint
}

// MarshalJSON returns a JSON byte array representation of Checkpoint.
func (c *Checkpoint) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		Epoch string        `json:"epoch"`
		Root  hexutil.Bytes `json:"root"`
	}{
		Epoch: fmt.Sprintf("%d", c.Checkpoint.Epoch),
		Root:  c.Checkpoint.Root,
	})
}

// AttestationData is a field of IndexedAttestation.
type AttestationData struct {
	*eth.AttestationData
}

// MarshalJSON returns a JSON byte array representation of AttestationData.
func (a *AttestationData) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		Slot            string        `json:"slot"`
		Index           string        `json:"index"`
		BeaconBlockRoot hexutil.Bytes `json:"beacon_block_root"`
		Source          *Checkpoint   `json:"source"`
		Target          *Checkpoint   `json:"target"`
	}{
		Slot:            fmt.Sprintf("%d", a.AttestationData.Slot),
		Index:           fmt.Sprintf("%d", a.AttestationData.CommitteeIndex),
		BeaconBlockRoot: a.AttestationData.BeaconBlockRoot,
		Source:          &Checkpoint{a.AttestationData.Source},
		Target:          &Checkpoint{a.AttestationData.Target},
	})
}

// Attestation is a field of Beacon Block Body.
type Attestation struct {
	*eth.Attestation
}

// MarshalJSON returns a JSON byte array representation of Attestation.
func (a *Attestation) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		AggregationBits hexutil.Bytes    `json:"aggregation_bits"`
		Data            *AttestationData `json:"data"`
		Signature       hexutil.Bytes    `json:"signature" ssz-size:"96"`
	}{
		AggregationBits: hexutil.Bytes(a.Attestation.AggregationBits),
		Data:            &AttestationData{a.Attestation.Data},
		Signature:       a.Attestation.Signature,
	})
}

// DepositData is a field of Deposit.
type DepositData struct {
	*eth.Deposit_Data
}

// MarshalJSON returns a JSON byte array representation of DepositData.
func (d *DepositData) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		PublicKey             hexutil.Bytes `json:"pubkey"`
		WithdrawalCredentials hexutil.Bytes `json:"withdrawal_credentials"`
		Amount                string        `json:"amount"`
		Signature             hexutil.Bytes `json:"signature"`
	}{
		PublicKey:             d.PublicKey,
		WithdrawalCredentials: d.WithdrawalCredentials,
		Amount:                fmt.Sprintf("%d", d.Amount),
		Signature:             d.Signature,
	})
}

// Deposit is a field of Beacon Block Body.
type Deposit struct {
	*eth.Deposit
}

// MarshalJSON returns a JSON byte array representation of Deposit.
func (d *Deposit) MarshalJSON() ([]byte, error) {
	proof := make([]hexutil.Bytes, len(d.Proof))
	for i := range d.Proof {
		proof[i] = d.Proof[i]
	}
	return json.Marshal(struct {
		Proof []hexutil.Bytes `json:"proof"`
		Data  *DepositData    `json:"data"`
	}{
		Proof: proof,
		Data:  &DepositData{Deposit_Data: d.Deposit.Data},
	})
}

// SignedVoluntaryExit is a field of Beacon Block Body.
type SignedVoluntaryExit struct {
	*eth.SignedVoluntaryExit
}

// MarshalJSON returns a JSON byte array representation of SignedVoluntaryExit.
func (sve *SignedVoluntaryExit) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		Message   *VoluntaryExit `json:"message"`
		Signature hexutil.Bytes  `json:"signature"`
	}{
		Signature: sve.SignedVoluntaryExit.Signature,
		Message:   &VoluntaryExit{sve.SignedVoluntaryExit.Exit},
	})
}

// VoluntaryExit is a field in SignedVoluntaryExit
type VoluntaryExit struct {
	*eth.VoluntaryExit
}

// MarshalJSON returns a JSON byte array representation of VoluntaryExit
func (ve *VoluntaryExit) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		Epoch          string `json:"epoch"`
		ValidatorIndex string `json:"validator_index"`
	}{
		Epoch:          fmt.Sprintf("%d", ve.Epoch),
		ValidatorIndex: fmt.Sprintf("%d", ve.ValidatorIndex),
	})
}

// SyncAggregate is a field of Beacon Block Body.
type SyncAggregate struct {
	*eth.SyncAggregate
}

// MarshalJSON returns a JSON byte array representation of SyncAggregate.
func (s *SyncAggregate) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		SyncCommitteeBits      hexutil.Bytes `json:"sync_committee_bits"`
		SyncCommitteeSignature hexutil.Bytes `json:"sync_committee_signature"`
	}{
		SyncCommitteeBits:      hexutil.Bytes(s.SyncAggregate.SyncCommitteeBits),
		SyncCommitteeSignature: s.SyncAggregate.SyncCommitteeSignature,
	})
}

// Eth1Data is a field of Beacon Block Body.
type Eth1Data struct {
	*eth.Eth1Data
}

// MarshalJSON returns a JSON byte array representation of Eth1Data.
func (e *Eth1Data) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		DepositRoot  hexutil.Bytes `json:"deposit_root"`
		DepositCount string        `json:"deposit_count"`
		BlockHash    hexutil.Bytes `json:"block_hash"`
	}{
		DepositRoot:  e.DepositRoot,
		DepositCount: fmt.Sprintf("%d", e.DepositCount),
		BlockHash:    e.BlockHash,
	})
}

// SignedBLSToExecutionChange is a field in Beacon Block Body for capella and above.
type SignedBLSToExecutionChange struct {
	*eth.SignedBLSToExecutionChange
}

// MarshalJSON returns a JSON byte array representation of SignedBLSToExecutionChange.
func (ch *SignedBLSToExecutionChange) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		Message   *BLSToExecutionChange `json:"message"`
		Signature hexutil.Bytes         `json:"signature"`
	}{
		Signature: ch.Signature,
		Message:   &BLSToExecutionChange{ch.Message},
	})
}

// BLSToExecutionChange is a field in SignedBLSToExecutionChange.
type BLSToExecutionChange struct {
	*eth.BLSToExecutionChange
}

// MarshalJSON returns a JSON byte array representation of BLSToExecutionChange.
func (ch *BLSToExecutionChange) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		ValidatorIndex     string        `json:"validator_index"`
		FromBlsPubkey      hexutil.Bytes `json:"from_bls_pubkey"`
		ToExecutionAddress hexutil.Bytes `json:"to_execution_address"`
	}{
		ValidatorIndex:     fmt.Sprintf("%d", ch.ValidatorIndex),
		FromBlsPubkey:      ch.FromBlsPubkey,
		ToExecutionAddress: ch.ToExecutionAddress,
	})
}

// ExecHeaderResponseDeneb is the header response for builder API /eth/v1/builder/header/{slot}/{parent_hash}/{pubkey}.
type ExecHeaderResponseDeneb struct {
	Data struct {
		Signature hexutil.Bytes    `json:"signature"`
		Message   *BuilderBidDeneb `json:"message"`
	} `json:"data"`
}

// ToProto creates a SignedBuilderBidDeneb Proto from ExecHeaderResponseDeneb.
func (ehr *ExecHeaderResponseDeneb) ToProto() (*eth.SignedBuilderBidDeneb, error) {
	bb, err := ehr.Data.Message.ToProto()
	if err != nil {
		return nil, err
	}
	return &eth.SignedBuilderBidDeneb{
		Message:   bb,
		Signature: bytesutil.SafeCopyBytes(ehr.Data.Signature),
	}, nil
}

// ToProto creates a BuilderBidDeneb Proto from BuilderBidDeneb.
func (bb *BuilderBidDeneb) ToProto() (*eth.BuilderBidDeneb, error) {
	header, err := bb.Header.ToProto()
	if err != nil {
		return nil, err
	}
	if len(bb.BlobKzgCommitments) > fieldparams.MaxBlobsPerBlock {
		return nil, fmt.Errorf("too many blob commitments: %d", len(bb.BlobKzgCommitments))
	}
	kzgCommitments := make([][]byte, len(bb.BlobKzgCommitments))
	for i, commit := range bb.BlobKzgCommitments {
		if len(commit) != fieldparams.BLSPubkeyLength {
			return nil, fmt.Errorf("commitment length %d is not %d", len(commit), fieldparams.BLSPubkeyLength)
		}
		kzgCommitments[i] = bytesutil.SafeCopyBytes(commit)
	}
	return &eth.BuilderBidDeneb{
		Header:             header,
		BlobKzgCommitments: kzgCommitments,
		Value:              bytesutil.SafeCopyBytes(bb.Value.SSZBytes()),
		Pubkey:             bytesutil.SafeCopyBytes(bb.Pubkey),
	}, nil
}

// BuilderBidDeneb is a field of ExecHeaderResponseDeneb.
type BuilderBidDeneb struct {
	Header             *ExecutionPayloadHeaderDeneb `json:"header"`
	BlobKzgCommitments []hexutil.Bytes              `json:"blob_kzg_commitments"`
	Value              Uint256                      `json:"value"`
	Pubkey             hexutil.Bytes                `json:"pubkey"`
}

// ExecutionPayloadHeaderDeneb a field part of the BuilderBidDeneb.
type ExecutionPayloadHeaderDeneb struct {
	ParentHash       hexutil.Bytes `json:"parent_hash"`
	FeeRecipient     hexutil.Bytes `json:"fee_recipient"`
	StateRoot        hexutil.Bytes `json:"state_root"`
	ReceiptsRoot     hexutil.Bytes `json:"receipts_root"`
	LogsBloom        hexutil.Bytes `json:"logs_bloom"`
	PrevRandao       hexutil.Bytes `json:"prev_randao"`
	BlockNumber      Uint64String  `json:"block_number"`
	GasLimit         Uint64String  `json:"gas_limit"`
	GasUsed          Uint64String  `json:"gas_used"`
	Timestamp        Uint64String  `json:"timestamp"`
	ExtraData        hexutil.Bytes `json:"extra_data"`
	BaseFeePerGas    Uint256       `json:"base_fee_per_gas"`
	BlockHash        hexutil.Bytes `json:"block_hash"`
	TransactionsRoot hexutil.Bytes `json:"transactions_root"`
	WithdrawalsRoot  hexutil.Bytes `json:"withdrawals_root"`
	BlobGasUsed      Uint64String  `json:"blob_gas_used"`   // new in deneb
	ExcessBlobGas    Uint64String  `json:"excess_blob_gas"` // new in deneb
	*v1.ExecutionPayloadHeaderDeneb
}

// MarshalJSON returns a JSON byte array representing the ExecutionPayloadHeaderDeneb struct.
func (h *ExecutionPayloadHeaderDeneb) MarshalJSON() ([]byte, error) {
	type MarshalCaller ExecutionPayloadHeaderDeneb
	baseFeePerGas, err := sszBytesToUint256(h.ExecutionPayloadHeaderDeneb.BaseFeePerGas)
	if err != nil {
		return []byte{}, errors.Wrapf(err, "invalid BaseFeePerGas")
	}
	return json.Marshal(&MarshalCaller{
		ParentHash:       h.ExecutionPayloadHeaderDeneb.ParentHash,
		FeeRecipient:     h.ExecutionPayloadHeaderDeneb.FeeRecipient,
		StateRoot:        h.ExecutionPayloadHeaderDeneb.StateRoot,
		ReceiptsRoot:     h.ExecutionPayloadHeaderDeneb.ReceiptsRoot,
		LogsBloom:        h.ExecutionPayloadHeaderDeneb.LogsBloom,
		PrevRandao:       h.ExecutionPayloadHeaderDeneb.PrevRandao,
		BlockNumber:      Uint64String(h.ExecutionPayloadHeaderDeneb.BlockNumber),
		GasLimit:         Uint64String(h.ExecutionPayloadHeaderDeneb.GasLimit),
		GasUsed:          Uint64String(h.ExecutionPayloadHeaderDeneb.GasUsed),
		Timestamp:        Uint64String(h.ExecutionPayloadHeaderDeneb.Timestamp),
		ExtraData:        h.ExecutionPayloadHeaderDeneb.ExtraData,
		BaseFeePerGas:    baseFeePerGas,
		BlockHash:        h.ExecutionPayloadHeaderDeneb.BlockHash,
		TransactionsRoot: h.ExecutionPayloadHeaderDeneb.TransactionsRoot,
		WithdrawalsRoot:  h.ExecutionPayloadHeaderDeneb.WithdrawalsRoot,
		BlobGasUsed:      Uint64String(h.ExecutionPayloadHeaderDeneb.BlobGasUsed),
		ExcessBlobGas:    Uint64String(h.ExecutionPayloadHeaderDeneb.ExcessBlobGas),
	})
}

// UnmarshalJSON takes in a byte array and unmarshals the value into ExecutionPayloadHeaderDeneb.
func (h *ExecutionPayloadHeaderDeneb) UnmarshalJSON(b []byte) error {
	type UnmarshalCaller ExecutionPayloadHeaderDeneb
	uc := &UnmarshalCaller{}
	if err := json.Unmarshal(b, uc); err != nil {
		return err
	}
	ep := ExecutionPayloadHeaderDeneb(*uc)
	*h = ep
	var err error
	h.ExecutionPayloadHeaderDeneb, err = h.ToProto()
	return err
}

// ToProto returns a ExecutionPayloadHeaderDeneb Proto object.
func (h *ExecutionPayloadHeaderDeneb) ToProto() (*v1.ExecutionPayloadHeaderDeneb, error) {
	return &v1.ExecutionPayloadHeaderDeneb{
		ParentHash:       bytesutil.SafeCopyBytes(h.ParentHash),
		FeeRecipient:     bytesutil.SafeCopyBytes(h.FeeRecipient),
		StateRoot:        bytesutil.SafeCopyBytes(h.StateRoot),
		ReceiptsRoot:     bytesutil.SafeCopyBytes(h.ReceiptsRoot),
		LogsBloom:        bytesutil.SafeCopyBytes(h.LogsBloom),
		PrevRandao:       bytesutil.SafeCopyBytes(h.PrevRandao),
		BlockNumber:      uint64(h.BlockNumber),
		GasLimit:         uint64(h.GasLimit),
		GasUsed:          uint64(h.GasUsed),
		Timestamp:        uint64(h.Timestamp),
		ExtraData:        bytesutil.SafeCopyBytes(h.ExtraData),
		BaseFeePerGas:    bytesutil.SafeCopyBytes(h.BaseFeePerGas.SSZBytes()),
		BlockHash:        bytesutil.SafeCopyBytes(h.BlockHash),
		TransactionsRoot: bytesutil.SafeCopyBytes(h.TransactionsRoot),
		WithdrawalsRoot:  bytesutil.SafeCopyBytes(h.WithdrawalsRoot),
		BlobGasUsed:      uint64(h.BlobGasUsed),
		ExcessBlobGas:    uint64(h.ExcessBlobGas),
	}, nil
}

// ExecPayloadResponseDeneb the response to the build API /eth/v1/builder/blinded_blocks that includes the version, execution payload object , and blobs bundle object.
type ExecPayloadResponseDeneb struct {
	Version string                               `json:"version"`
	Data    *ExecutionPayloadDenebAndBlobsBundle `json:"data"`
}

// ExecutionPayloadDenebAndBlobsBundle the main field used in ExecPayloadResponseDeneb.
type ExecutionPayloadDenebAndBlobsBundle struct {
	ExecutionPayload *ExecutionPayloadDeneb `json:"execution_payload"`
	BlobsBundle      *BlobsBundle           `json:"blobs_bundle"`
}

// ExecutionPayloadDeneb is a field used in ExecutionPayloadDenebAndBlobsBundle.
type ExecutionPayloadDeneb struct {
	ParentHash    hexutil.Bytes   `json:"parent_hash"`
	FeeRecipient  hexutil.Bytes   `json:"fee_recipient"`
	StateRoot     hexutil.Bytes   `json:"state_root"`
	ReceiptsRoot  hexutil.Bytes   `json:"receipts_root"`
	LogsBloom     hexutil.Bytes   `json:"logs_bloom"`
	PrevRandao    hexutil.Bytes   `json:"prev_randao"`
	BlockNumber   Uint64String    `json:"block_number"`
	GasLimit      Uint64String    `json:"gas_limit"`
	GasUsed       Uint64String    `json:"gas_used"`
	Timestamp     Uint64String    `json:"timestamp"`
	ExtraData     hexutil.Bytes   `json:"extra_data"`
	BaseFeePerGas Uint256         `json:"base_fee_per_gas"`
	BlockHash     hexutil.Bytes   `json:"block_hash"`
	Transactions  []hexutil.Bytes `json:"transactions"`
	Withdrawals   []Withdrawal    `json:"withdrawals"`
	BlobGasUsed   Uint64String    `json:"blob_gas_used"`   // new in deneb
	ExcessBlobGas Uint64String    `json:"excess_blob_gas"` // new in deneb
}

// BlobsBundle is a field in ExecutionPayloadDenebAndBlobsBundle.
type BlobsBundle struct {
	Commitments []hexutil.Bytes `json:"commitments"`
	Proofs      []hexutil.Bytes `json:"proofs"`
	Blobs       []hexutil.Bytes `json:"blobs"`
}

// ToProto returns a BlobsBundle Proto.
func (b BlobsBundle) ToProto() (*v1.BlobsBundle, error) {
	if len(b.Blobs) > fieldparams.MaxBlobCommitmentsPerBlock {
		return nil, fmt.Errorf("blobs length %d is more than max %d", len(b.Blobs), fieldparams.MaxBlobCommitmentsPerBlock)
	}
	if len(b.Commitments) != len(b.Blobs) {
		return nil, fmt.Errorf("commitments length %d does not equal blobs length %d", len(b.Commitments), len(b.Blobs))
	}
	if len(b.Proofs) != len(b.Blobs) {
		return nil, fmt.Errorf("proofs length %d does not equal blobs length %d", len(b.Proofs), len(b.Blobs))
	}

	commitments := make([][]byte, len(b.Commitments))
	for i := range b.Commitments {
		if len(b.Commitments[i]) != fieldparams.BLSPubkeyLength {
			return nil, fmt.Errorf("commitment length %d is not %d", len(b.Commitments[i]), fieldparams.BLSPubkeyLength)
		}
		commitments[i] = bytesutil.SafeCopyBytes(b.Commitments[i])
	}
	proofs := make([][]byte, len(b.Proofs))
	for i := range b.Proofs {
		if len(b.Proofs[i]) != fieldparams.BLSPubkeyLength {
			return nil, fmt.Errorf("proof length %d is not %d", len(b.Proofs[i]), fieldparams.BLSPubkeyLength)
		}
		proofs[i] = bytesutil.SafeCopyBytes(b.Proofs[i])
	}
	blobs := make([][]byte, len(b.Blobs))
	for i := range b.Blobs {
		if len(b.Blobs[i]) != fieldparams.BlobLength {
			return nil, fmt.Errorf("blob length %d is not %d", len(b.Blobs[i]), fieldparams.BlobLength)
		}
		blobs[i] = bytesutil.SafeCopyBytes(b.Blobs[i])
	}
	return &v1.BlobsBundle{
		KzgCommitments: commitments,
		Proofs:         proofs,
		Blobs:          blobs,
	}, nil
}

// FromBundleProto converts the proto bundle type to the builder
// type.
func FromBundleProto(bundle *v1.BlobsBundle) *BlobsBundle {
	commitments := make([]hexutil.Bytes, len(bundle.KzgCommitments))
	for i := range bundle.KzgCommitments {
		commitments[i] = bytesutil.SafeCopyBytes(bundle.KzgCommitments[i])
	}
	proofs := make([]hexutil.Bytes, len(bundle.Proofs))
	for i := range bundle.Proofs {
		proofs[i] = bytesutil.SafeCopyBytes(bundle.Proofs[i])
	}
	blobs := make([]hexutil.Bytes, len(bundle.Blobs))
	for i := range bundle.Blobs {
		blobs[i] = bytesutil.SafeCopyBytes(bundle.Blobs[i])
	}
	return &BlobsBundle{
		Commitments: commitments,
		Proofs:      proofs,
		Blobs:       blobs,
	}
}

// ToProto returns ExecutionPayloadDeneb Proto and BlobsBundle Proto separately.
func (r *ExecPayloadResponseDeneb) ToProto() (*v1.ExecutionPayloadDeneb, *v1.BlobsBundle, error) {
	if r.Data == nil {
		return nil, nil, errors.New("data field in response is empty")
	}
	payload, err := r.Data.ExecutionPayload.ToProto()
	if err != nil {
		return nil, nil, err
	}
	bundle, err := r.Data.BlobsBundle.ToProto()
	if err != nil {
		return nil, nil, err
	}
	return payload, bundle, nil
}

// ToProto returns the ExecutionPayloadDeneb Proto.
func (p *ExecutionPayloadDeneb) ToProto() (*v1.ExecutionPayloadDeneb, error) {
	txs := make([][]byte, len(p.Transactions))
	for i := range p.Transactions {
		txs[i] = bytesutil.SafeCopyBytes(p.Transactions[i])
	}
	withdrawals := make([]*v1.Withdrawal, len(p.Withdrawals))
	for i, w := range p.Withdrawals {
		withdrawals[i] = &v1.Withdrawal{
			Index:          w.Index.Uint64(),
			ValidatorIndex: types.ValidatorIndex(w.ValidatorIndex.Uint64()),
			Address:        bytesutil.SafeCopyBytes(w.Address),
			Amount:         w.Amount.Uint64(),
		}
	}
	return &v1.ExecutionPayloadDeneb{
		ParentHash:    bytesutil.SafeCopyBytes(p.ParentHash),
		FeeRecipient:  bytesutil.SafeCopyBytes(p.FeeRecipient),
		StateRoot:     bytesutil.SafeCopyBytes(p.StateRoot),
		ReceiptsRoot:  bytesutil.SafeCopyBytes(p.ReceiptsRoot),
		LogsBloom:     bytesutil.SafeCopyBytes(p.LogsBloom),
		PrevRandao:    bytesutil.SafeCopyBytes(p.PrevRandao),
		BlockNumber:   uint64(p.BlockNumber),
		GasLimit:      uint64(p.GasLimit),
		GasUsed:       uint64(p.GasUsed),
		Timestamp:     uint64(p.Timestamp),
		ExtraData:     bytesutil.SafeCopyBytes(p.ExtraData),
		BaseFeePerGas: bytesutil.SafeCopyBytes(p.BaseFeePerGas.SSZBytes()),
		BlockHash:     bytesutil.SafeCopyBytes(p.BlockHash),
		Transactions:  txs,
		Withdrawals:   withdrawals,
		BlobGasUsed:   uint64(p.BlobGasUsed),
		ExcessBlobGas: uint64(p.ExcessBlobGas),
	}, nil
}

// ErrorMessage is a JSON representation of the builder API's returned error message.
type ErrorMessage struct {
	Code        int      `json:"code"`
	Message     string   `json:"message"`
	Stacktraces []string `json:"stacktraces,omitempty"`
}
