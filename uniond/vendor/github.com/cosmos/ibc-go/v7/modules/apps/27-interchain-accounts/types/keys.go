package types

import (
	"fmt"
)

const (
	// ModuleName defines the interchain accounts module name
	ModuleName = "interchainaccounts"

	// HostPortID is the default port id that the interchain accounts host submodule binds to
	HostPortID = "icahost"

	// ControllerPortPrefix is the default port prefix that the interchain accounts controller submodule binds to
	ControllerPortPrefix = "icacontroller-"

	// Version defines the current version for interchain accounts
	Version = "ics27-1"

	// StoreKey is the store key string for interchain accounts
	StoreKey = ModuleName

	// RouterKey is the message route for interchain accounts
	RouterKey = ModuleName

	// QuerierRoute is the querier route for interchain accounts
	QuerierRoute = ModuleName

	// hostAccountKey is the key used when generating a module address for the host submodule
	hostAccountsKey = "icahost-accounts"
)

var (
	// ActiveChannelKeyPrefix defines the key prefix used to store active channels
	ActiveChannelKeyPrefix = "activeChannel"

	// OwnerKeyPrefix defines the key prefix used to store interchain accounts
	OwnerKeyPrefix = "owner"

	// PortKeyPrefix defines the key prefix used to store ports
	PortKeyPrefix = "port"

	// IsMiddlewareEnabledPrefix defines the key prefix used to store a flag for legacy API callback routing via ibc middleware
	IsMiddlewareEnabledPrefix = "isMiddlewareEnabled"

	// MiddlewareEnabled is the value used to signal that controller middleware is enabled
	MiddlewareEnabled = []byte{0x01}

	// MiddlewareDisabled is the value used to signal that controller midleware is disabled
	MiddlewareDisabled = []byte{0x02}
)

// KeyActiveChannel creates and returns a new key used for active channels store operations
func KeyActiveChannel(portID, connectionID string) []byte {
	return []byte(fmt.Sprintf("%s/%s/%s", ActiveChannelKeyPrefix, portID, connectionID))
}

// KeyOwnerAccount creates and returns a new key used for interchain account store operations
func KeyOwnerAccount(portID, connectionID string) []byte {
	return []byte(fmt.Sprintf("%s/%s/%s", OwnerKeyPrefix, portID, connectionID))
}

// KeyPort creates and returns a new key used for port store operations
func KeyPort(portID string) []byte {
	return []byte(fmt.Sprintf("%s/%s", PortKeyPrefix, portID))
}

// KeyIsMiddlewareEnabled creates and returns a new key used for signaling legacy API callback routing via ibc middleware
func KeyIsMiddlewareEnabled(portID, connectionID string) []byte {
	return []byte(fmt.Sprintf("%s/%s/%s", IsMiddlewareEnabledPrefix, portID, connectionID))
}
