package types

type (
	// HumanizeAddressFunc is a type for functions that convert a canonical address (bytes)
	// to a human readable address (typically bech32).
	HumanizeAddressFunc func([]byte) (string, uint64, error)
	// CanonicalizeAddressFunc is a type for functions that convert a human readable address (typically bech32)
	// to a canonical address (bytes).
	CanonicalizeAddressFunc func(string) ([]byte, uint64, error)
	// ValidateAddressFunc is a type for functions that validate a human readable address (typically bech32).
	ValidateAddressFunc func(string) (uint64, error)
)

type GoAPI struct {
	HumanizeAddress     HumanizeAddressFunc
	CanonicalizeAddress CanonicalizeAddressFunc
	ValidateAddress     ValidateAddressFunc
}
