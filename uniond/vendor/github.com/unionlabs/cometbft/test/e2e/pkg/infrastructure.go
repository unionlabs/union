package e2e

import (
	"encoding/json"
	"fmt"
	"net"
	"os"
	"sort"
)

const (
	dockerIPv4CIDR = "10.186.73.0/24"
	dockerIPv6CIDR = "fd80:b10c::/48"

	globalIPv4CIDR = "0.0.0.0/0"
)

// InfrastructureData contains the relevant information for a set of existing
// infrastructure that is to be used for running a testnet.
type InfrastructureData struct {
	Path string `json:"path"`

	// Provider is the name of infrastructure provider backing the testnet.
	// For example, 'docker' if it is running locally in a docker network or
	// 'digital-ocean', 'aws', 'google', etc. if it is from a cloud provider.
	Provider string `json:"provider"`

	// Instances is a map of all of the machine instances on which to run
	// processes for a testnet.
	// The key of the map is the name of the instance, which each must correspond
	// to the names of one of the testnet nodes defined in the testnet manifest.
	Instances map[string]InstanceData `json:"instances"`

	// Network is the CIDR notation range of IP addresses that all of the instances'
	// IP addresses are expected to be within.
	Network string `json:"network"`
}

// InstanceData contains the relevant information for a machine instance backing
// one of the nodes in the testnet.
type InstanceData struct {
	IPAddress          net.IP `json:"ip_address"`
	ExtIPAddress       net.IP `json:"ext_ip_address"`
	RPCPort            uint32 `json:"rpc_port"`
	GRPCPort           uint32 `json:"grpc_port"`
	PrivilegedGRPCPort uint32 `json:"privileged_grpc_port"`
}

func sortNodeNames(m Manifest) []string {
	// Set up nodes, in alphabetical order (IPs and ports get same order).
	nodeNames := []string{}
	for name := range m.Nodes {
		nodeNames = append(nodeNames, name)
	}
	sort.Strings(nodeNames)
	return nodeNames
}

func NewDockerInfrastructureData(m Manifest) (InfrastructureData, error) {
	netAddress := dockerIPv4CIDR
	if m.IPv6 {
		netAddress = dockerIPv6CIDR
	}
	_, ipNet, err := net.ParseCIDR(netAddress)
	if err != nil {
		return InfrastructureData{}, fmt.Errorf("invalid IP network address %q: %w", netAddress, err)
	}

	portGen := newPortGenerator(proxyPortFirst)
	ipGen := newIPGenerator(ipNet)
	ifd := InfrastructureData{
		Provider:  "docker",
		Instances: make(map[string]InstanceData),
		Network:   netAddress,
	}
	localHostIP := net.ParseIP("127.0.0.1")
	for _, name := range sortNodeNames(m) {
		ifd.Instances[name] = InstanceData{
			IPAddress:          ipGen.Next(),
			ExtIPAddress:       localHostIP,
			RPCPort:            portGen.Next(),
			GRPCPort:           portGen.Next(),
			PrivilegedGRPCPort: portGen.Next(),
		}
	}
	return ifd, nil
}

func InfrastructureDataFromFile(p string) (InfrastructureData, error) {
	ifd := InfrastructureData{}
	b, err := os.ReadFile(p)
	if err != nil {
		return InfrastructureData{}, err
	}
	err = json.Unmarshal(b, &ifd)
	if err != nil {
		return InfrastructureData{}, err
	}
	if ifd.Network == "" {
		ifd.Network = globalIPv4CIDR
	}
	ifd.Path = p
	return ifd, nil
}
