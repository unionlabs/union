package types

import (
	"encoding/json"
	"fmt"
	"strconv"

	"github.com/shamaton/msgpack/v2"
)

// Uint64 is a wrapper for uint64, but it is marshalled to and from JSON as a string
type Uint64 uint64

func (u Uint64) MarshalJSON() ([]byte, error) {
	return json.Marshal(strconv.FormatUint(uint64(u), 10))
}

func (u *Uint64) UnmarshalJSON(data []byte) error {
	var s string
	if err := json.Unmarshal(data, &s); err != nil {
		return fmt.Errorf("cannot unmarshal %s into Uint64, expected string-encoded integer", data)
	}
	v, err := strconv.ParseUint(s, 10, 64)
	if err != nil {
		return fmt.Errorf("cannot unmarshal %s into Uint64, failed to parse integer", data)
	}
	*u = Uint64(v)
	return nil
}

// Int64 is a wrapper for int64, but it is marshalled to and from JSON as a string
type Int64 int64

func (i Int64) MarshalJSON() ([]byte, error) {
	return json.Marshal(strconv.FormatInt(int64(i), 10))
}

func (i *Int64) UnmarshalJSON(data []byte) error {
	var s string
	if err := json.Unmarshal(data, &s); err != nil {
		return fmt.Errorf("cannot unmarshal %s into Int64, expected string-encoded integer", data)
	}
	v, err := strconv.ParseInt(s, 10, 64)
	if err != nil {
		return fmt.Errorf("cannot unmarshal %s into Int64, failed to parse integer", data)
	}
	*i = Int64(v)
	return nil
}

// HumanAddress is a printable (typically bech32 encoded) address string. Just use it as a label for developers.
type HumanAddress = string

// CanonicalAddress uses standard base64 encoding, just use it as a label for developers
type CanonicalAddress = []byte

// Coin is a string representation of the sdk.Coin type (more portable than sdk.Int)
type Coin struct {
	Denom  string `json:"denom"`  // type, eg. "ATOM"
	Amount string `json:"amount"` // string encoding of decimal value, eg. "12.3456"
}

func NewCoin(amount uint64, denom string) Coin {
	return Coin{
		Denom:  denom,
		Amount: strconv.FormatUint(amount, 10),
	}
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
	// Some chains have chosen atto (10^-18) for their token's base denomination. If we used `Decimal` here, we could only store 340282366920938463463.374607431768211455atoken which is 340.28 TOKEN.
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
	HasIBCEntryPoints    bool
	RequiredCapabilities string
	Entrypoints          []string
	// ContractMigrateVersion is the migrate version of the contract
	// This is nil if the contract does not have a migrate version and the `migrate` entrypoint
	// needs to be called for every migration (if present).
	// If it is some number, the entrypoint only needs to be called if it increased.
	ContractMigrateVersion *uint64
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

type PerModuleMetrics struct {
	Hits uint32 `msgpack:"hits"`
	Size uint64 `msgpack:"size"`
}

type PerModuleEntry struct {
	Checksum Checksum
	Metrics  PerModuleMetrics
}

type PinnedMetrics struct {
	PerModule []PerModuleEntry `msgpack:"per_module"`
}

func (pm *PinnedMetrics) UnmarshalMessagePack(data []byte) error {
	return msgpack.UnmarshalAsArray(data, pm)
}

// Array is a wrapper around a slice that ensures that we get "[]" JSON for nil values.
// When unmarshaling, we get an empty slice for "[]" and "null".
//
// This is needed for fields that are "Vec<C>" on the Rust side because `null` values
// will result in an error there.
// Using this on a field with an "Option<Vec<C>>" type on the Rust side would
// never result in a "None" value on the Rust side, making the "Option" pointless.
type Array[C any] []C

// MarshalJSON ensures that we get "[]" for nil arrays
func (a Array[C]) MarshalJSON() ([]byte, error) {
	if len(a) == 0 {
		return []byte("[]"), nil
	}
	var raw []C = a
	return json.Marshal(raw)
}

// UnmarshalJSON ensures that we get an empty slice for "[]" and "null"
func (a *Array[C]) UnmarshalJSON(data []byte) error {
	var raw []C
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}
	// make sure we deserialize [] back to empty slice
	if len(raw) == 0 {
		raw = []C{}
	}
	*a = raw
	return nil
}
