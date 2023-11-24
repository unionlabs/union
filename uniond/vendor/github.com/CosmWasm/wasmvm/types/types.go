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

// Replicating the cosmos-sdk bank module Metadata type
type DenomMetadata struct {
	Description string `json:"description"`
	// DenomUnits represents the list of DenomUnits for a given coin
	DenomUnits []DenomUnit `json:"denom_units"`
	// Base represents the base denom (should be the DenomUnit with exponent = 0).
	Base string `json:"base"`
	// Display indicates the suggested denom that should be
	// displayed in clients.
	Display string `json:"display"`
	// Name defines the name of the token (eg: Cosmos Atom)
	//
	// Since: cosmos-sdk 0.43
	Name string `json:"name"`
	// Symbol is the token symbol usually shown on exchanges (eg: ATOM). This can
	// be the same as the display.
	//
	// Since: cosmos-sdk 0.43
	Symbol string `json:"symbol"`
	// URI to a document (on or off-chain) that contains additional information. Optional.
	//
	// Since: cosmos-sdk 0.46
	URI string `json:"uri"`
	// URIHash is a sha256 hash of a document pointed by URI. It's used to verify that
	// the document didn't change. Optional.
	//
	// Since: cosmos-sdk 0.46
	URIHash string `json:"uri_hash"`
}

// Replicating the cosmos-sdk bank module DenomUnit type
type DenomUnit struct {
	// Denom represents the string name of the given denom unit (e.g uatom).
	Denom string `json:"denom"`
	// Exponent represents power of 10 exponent that one must
	// raise the base_denom to in order to equal the given DenomUnit's denom
	// 1 denom = 10^exponent base_denom
	// (e.g. with a base_denom of uatom, one can create a DenomUnit of 'atom' with
	// exponent = 6, thus: 1 atom = 10^6 uatom).
	Exponent uint32 `json:"exponent"`
	// Aliases is a list of string aliases for the given denom
	Aliases []string `json:"aliases"`
}

// A coin type with decimal amount.
// Modeled after the Cosmos SDK's [DecCoin] type.
// However, in contrast to the Cosmos SDK the `amount` string MUST always have a dot at JSON level,
// see <https://github.com/cosmos/cosmos-sdk/issues/10863>.
// Also if Cosmos SDK choses to migrate away from fixed point decimals
// (as shown [here](https://github.com/cosmos/cosmos-sdk/blob/v0.47.4/x/group/internal/math/dec.go#L13-L21) and discussed [here](https://github.com/cosmos/cosmos-sdk/issues/11783)),
// wasmd needs to truncate the decimal places to 18.
//
// [DecCoin]: (https://github.com/cosmos/cosmos-sdk/blob/v0.47.4/proto/cosmos/base/v1beta1/coin.proto#L28-L38)
type DecCoin struct {
	// An amount in the base denom of the distributed token.
	//
	// Some chains have choosen atto (10^-18) for their token's base denomination. If we used `Decimal` here, we could only store 340282366920938463463.374607431768211455atoken which is 340.28 TOKEN.
	Amount string `json:"amount"`
	Denom  string `json:"denom"`
}

// Simplified version of the cosmos-sdk PageRequest type
type PageRequest struct {
	// Key is a value returned in PageResponse.next_key to begin
	// querying the next page most efficiently. Only one of offset or key
	// should be set.
	Key []byte `json:"key"`
	// Limit is the total number of results to be returned in the result page.
	// If left empty it will default to a value to be set by each app.
	Limit uint32 `json:"limit"`
	// Reverse is set to true if results are to be returned in the descending order.
	Reverse bool `json:"reverse"`
}

type OutOfGasError struct{}

var _ error = OutOfGasError{}

func (o OutOfGasError) Error() string {
	return "Out of gas"
}

type GasReport struct {
	Limit          uint64
	Remaining      uint64
	UsedExternally uint64
	UsedInternally uint64
}

func EmptyGasReport(limit uint64) GasReport {
	return GasReport{
		Limit:          limit,
		Remaining:      limit,
		UsedExternally: 0,
		UsedInternally: 0,
	}
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
