package types

var (
	// MinterKey is the key to use for the keeper store.
	MinterKey = []byte{0x00}
	ParamsKey = []byte{0x01}
)

const (
	// module name
	ModuleName = "mint"

	// StoreKey is the default store key for mint
	StoreKey = ModuleName

	// Query endpoints supported by the minting querier
	QueryParameters       = "parameters"
	QueryInflation        = "inflation"
	QueryAnnualProvisions = "annual_provisions"
)
