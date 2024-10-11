package host

import "fmt"

const (
	KeyPortPrefix = "ports"
)

// ICS05
// The following paths are the keys to the store as defined in https://github.com/cosmos/ibc/tree/master/spec/core/ics-005-port-allocation#store-paths

// PortPath defines the path under which ports paths are stored on the capability module
func PortPath(portID string) string {
	return fmt.Sprintf("%s/%s", KeyPortPrefix, portID)
}
