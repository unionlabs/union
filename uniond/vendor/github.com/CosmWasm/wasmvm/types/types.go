package types

import (
	"encoding/json"
	"strconv"
)

// HumanAddress is a printable (typically bech32 encoded) address string. Just use it as a label for developers.
type HumanAddress = string

// CanonicalAddress uses standard base64 encoding, just use it as a label for developers
type CanonicalAddress = []byte

// Coin is a string representation of the sdk.Coin type (more portable than sdk.Int)
type Coin struct {
	Denom  string `json:"denom"`  // type, eg. "ATOM"
	Amount string `json:"amount"` // string encoing of decimal value, eg. "12.3456"
}

func NewCoin(amount uint64, denom string) Coin {
	return Coin{
		Denom:  denom,
		Amount: strconv.FormatUint(amount, 10),
	}
}

// Coins handles properly serializing empty amounts
type Coins []Coin

// MarshalJSON ensures that we get [] for empty arrays
func (c Coins) MarshalJSON() ([]byte, error) {
	if len(c) == 0 {
		return []byte("[]"), nil
	}
	var d []Coin = c
	return json.Marshal(d)
}

// UnmarshalJSON ensures that we get [] for empty arrays
func (c *Coins) UnmarshalJSON(data []byte) error {
	// make sure we deserialize [] back to null
	if string(data) == "[]" || string(data) == "null" {
		return nil
	}
	var d []Coin
	if err := json.Unmarshal(data, &d); err != nil {
		return err
	}
	*c = d
	return nil
}

type OutOfGasError struct{}

var _ error = OutOfGasError{}

func (o OutOfGasError) Error() string {
	return "Out of gas"
}

// Contains static analysis info of the contract (the Wasm code to be precise).
// This type is returned by VM.AnalyzeCode().
type AnalysisReport struct {
	HasIBCEntryPoints bool
	// Deprecated, use RequiredCapabilities. For now both fields contain the same value.
	RequiredFeatures     string
	RequiredCapabilities string
}

type Metrics struct {
	HitsPinnedMemoryCache     uint32
	HitsMemoryCache           uint32
	HitsFsCache               uint32
	Misses                    uint32
	ElementsPinnedMemoryCache uint64
	ElementsMemoryCache       uint64
	// Cumulative size of all elements in pinned memory cache (in bytes)
	SizePinnedMemoryCache uint64
	// Cumulative size of all elements in memory cache (in bytes)
	SizeMemoryCache uint64
}
