package params

import (
	"github.com/mohae/deepcopy"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

// NetworkConfig defines the spec based network parameters.
type NetworkConfig struct {
	// DiscoveryV5 Config
	ETH2Key                    string // ETH2Key is the ENR key of the Ethereum consensus object in an enr.
	AttSubnetKey               string // AttSubnetKey is the ENR key of the subnet bitfield in the enr.
	SyncCommsSubnetKey         string // SyncCommsSubnetKey is the ENR key of the sync committee subnet bitfield in the enr.
	MinimumPeersInSubnetSearch uint64 // PeersInSubnetSearch is the required amount of peers that we need to be able to lookup in a subnet search.

	// Chain Network Config
	ContractDeploymentBlock uint64   // ContractDeploymentBlock is the eth1 block in which the deposit contract is deployed.
	BootstrapNodes          []string // BootstrapNodes are the addresses of the bootnodes.
}

var networkConfig = mainnetNetworkConfig

// BeaconNetworkConfig returns the current network config for
// the beacon chain.
func BeaconNetworkConfig() *NetworkConfig {
	return networkConfig
}

// OverrideBeaconNetworkConfig will override the network
// config with the added argument.
func OverrideBeaconNetworkConfig(cfg *NetworkConfig) {
	networkConfig = cfg.Copy()
}

// Copy returns Copy of the config object.
func (c *NetworkConfig) Copy() *NetworkConfig {
	config, ok := deepcopy.Copy(*c).(NetworkConfig)
	if !ok {
		config = *networkConfig
	}
	return &config
}

// MaxRequestBlock determines the maximum number of blocks that can be requested in a single
// request for a given epoch. If the epoch is at or beyond config's `DenebForkEpoch`,
// a special limit defined for Deneb is used.
func MaxRequestBlock(e primitives.Epoch) uint64 {
	if e >= BeaconConfig().DenebForkEpoch {
		return BeaconConfig().MaxRequestBlocksDeneb
	}
	return BeaconConfig().MaxRequestBlocks
}
