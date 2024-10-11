package types

import "fmt"

const (
	// ModuleName is the name of this module
	ModuleName = "upgrade"

	// RouterKey is used to route governance proposals
	RouterKey = ModuleName

	// StoreKey is the prefix under which we store this module's data
	StoreKey = ModuleName

	// GovModuleName duplicates the gov module's name to avoid a cyclic dependency with x/gov.
	// It should be synced with the gov module's name if it is ever changed.
	// See: https://github.com/cosmos/cosmos-sdk/blob/b62a28aac041829da5ded4aeacfcd7a42873d1c8/x/gov/types/keys.go#L9
	GovModuleName = "gov"
)

const (
	// PlanByte specifies the Byte under which a pending upgrade plan is stored in the store
	PlanByte = 0x0

	// DoneByte is a prefix to look up completed upgrade plan by name
	DoneByte = 0x1

	// VersionMapByte is a prefix to look up module names (key) and versions (value)
	VersionMapByte = 0x2

	// KeyUpgradedIBCState is the key under which upgraded ibc state is stored in the upgrade store
	KeyUpgradedIBCState = "upgradedIBCState"

	// KeyUpgradedClient is the sub-key under which upgraded client state will be stored
	KeyUpgradedClient = "upgradedClient"

	// KeyUpgradedConsState is the sub-key under which upgraded consensus state will be stored
	KeyUpgradedConsState = "upgradedConsState"
)

// PlanKey is the key under which the current plan is saved
// We store PlanByte as a const to keep it immutable (unlike a []byte)
func PlanKey() []byte {
	return []byte{PlanByte}
}

// UpgradedClientKey is the key under which the upgraded client state is saved
// Connecting IBC chains can verify against the upgraded client in this path before
// upgrading their clients
func UpgradedClientKey(height int64) []byte {
	return []byte(fmt.Sprintf("%s/%d/%s", KeyUpgradedIBCState, height, KeyUpgradedClient))
}

// UpgradedConsStateKey is the key under which the upgraded consensus state is saved
// Connecting IBC chains can verify against the upgraded consensus state in this path before
// upgrading their clients.
func UpgradedConsStateKey(height int64) []byte {
	return []byte(fmt.Sprintf("%s/%d/%s", KeyUpgradedIBCState, height, KeyUpgradedConsState))
}
