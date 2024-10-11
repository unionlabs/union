package types

import (
	"encoding/json"
	"errors"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
)

// HeaderInfo specifies the block header information in the ETH 1.0 chain.
type HeaderInfo struct {
	Number *big.Int    `json:"number"`
	Hash   common.Hash `json:"hash"`
	Time   uint64      `json:"timestamp"`
}

// Copy sends out a copy of the current header info.
func (h *HeaderInfo) Copy() *HeaderInfo {
	return &HeaderInfo{
		Hash:   bytesutil.ToBytes32(h.Hash[:]),
		Number: new(big.Int).Set(h.Number),
		Time:   h.Time,
	}
}

// MarshalJSON marshals as JSON.
func (h *HeaderInfo) MarshalJSON() ([]byte, error) {
	type HeaderInfoJson struct {
		Number *hexutil.Big   `json:"number"`
		Hash   common.Hash    `json:"hash"`
		Time   hexutil.Uint64 `json:"timestamp"`
	}
	var enc HeaderInfoJson

	enc.Number = (*hexutil.Big)(h.Number)
	enc.Hash = h.Hash
	enc.Time = hexutil.Uint64(h.Time)
	return json.Marshal(enc)
}

// UnmarshalJSON unmarshals from JSON.
func (h *HeaderInfo) UnmarshalJSON(data []byte) error {
	type HeaderInfoJson struct {
		Number *hexutil.Big    `json:"number"`
		Hash   *common.Hash    `json:"hash"`
		Time   *hexutil.Uint64 `json:"timestamp"`
	}
	var dec HeaderInfoJson
	if err := json.Unmarshal(data, &dec); err != nil {
		return err
	}
	if dec.Time == nil {
		return errors.New("missing required field 'timestamp'")
	}
	h.Time = uint64(*dec.Time)
	if dec.Number == nil {
		return errors.New("missing required field 'number'")
	}
	h.Number = (*big.Int)(dec.Number)
	if dec.Hash == nil {
		return errors.New("missing required field 'hash'")
	}
	h.Hash = *dec.Hash
	return nil
}
