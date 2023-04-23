package types

import "fmt"

// GetModuleOwner enforces that only IBC and the module bound to port can own the capability
// while future implementations may allow multiple modules to bind to a port, currently we
// only allow one module to be bound to a port at any given time
func GetModuleOwner(modules []string) string {
	if len(modules) != 2 {
		panic(fmt.Sprintf("capability should only be owned by port or channel owner and ibc module, multiple owners currently not supported, owners: %v", modules))
	}

	if modules[0] == "ibc" {
		return modules[1]
	}
	return modules[0]
}
