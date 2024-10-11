package clientstats

const (
	ClientName            = "prysm"
	BeaconNodeProcessName = "beaconnode"
	ValidatorProcessName  = "validator"
	APIVersion            = 1
)

// APIMessage are common to all requests to the client-stats API
// Note that there is a "system" type that we do not currently
// support -- if we did APIMessage would be present on the system
// messages as well as validator and beaconnode, whereas
// CommonStats would only be part of beaconnode and validator.
type APIMessage struct {
	APIVersion  int    `json:"version"`
	Timestamp   int64  `json:"timestamp"` // unix timestamp in milliseconds
	ProcessName string `json:"process"`   // validator, beaconnode, system
}

// CommonStats represent generic metrics that are expected on both
// beaconnode and validator metric types. This type is used for
// marshaling metrics to the POST body sent to the metrics collcetor.
// Note that some metrics are labeled NA because they are expected
// to be present with their zero-value when not supported by a client.
type CommonStats struct {
	CPUProcessSecondsTotal int64  `json:"cpu_process_seconds_total"`
	MemoryProcessBytes     int64  `json:"memory_process_bytes"`
	ClientName             string `json:"client_name"`
	ClientVersion          string `json:"client_version"`
	ClientBuild            int64  `json:"client_build"`
	// TODO(#8849): parse the grpc connection string to determine
	// if multiple addresses are present
	SyncEth2FallbackConfigured bool `json:"sync_eth2_fallback_configured"`
	// N/A -- when multiple addresses are provided to grpc, requests are
	// load-balanced between the provided endpoints.
	// This is different from a "fallback" configuration where
	// the second address is treated as a failover.
	SyncEth2FallbackConnected bool `json:"sync_eth2_fallback_connected"`
	APIMessage                `json:",inline"`
}

// BeaconNodeStats embeds CommonStats and represents metrics specific to
// the beacon-node process. This type is used to marshal metrics data
// to the POST body sent to the metrics collcetor. To make the connection
// to client-stats clear, BeaconNodeStats is also used by prometheus
// collection code introduced to support client-stats.
// Note that some metrics are labeled NA because they are expected
// to be present with their zero-value when not supported by a client.
type BeaconNodeStats struct {
	// TODO(#8850): add support for this after slasher refactor is merged
	SlasherActive             bool  `json:"slasher_active"`
	SyncEth1Connected         bool  `json:"sync_eth1_connected"`
	SyncEth2Synced            bool  `json:"sync_eth2_synced"`
	DiskBeaconchainBytesTotal int64 `json:"disk_beaconchain_bytes_total"`
	// N/A -- would require significant network code changes at this time
	NetworkLibp2pBytesTotalReceive int64 `json:"network_libp2p_bytes_total_receive"`
	// N/A -- would require significant network code changes at this time
	NetworkLibp2pBytesTotalTransmit int64 `json:"network_libp2p_bytes_total_transmit"`
	// p2p_peer_count where label "state" == "Connected"
	NetworkPeersConnected int64 `json:"network_peers_connected"`
	// beacon_head_slot
	SyncBeaconHeadSlot int64 `json:"sync_beacon_head_slot"`
	CommonStats        `json:",inline"`
}

// ValidatorStats embeds CommonStats and represents metrics specific to
// the validator process. This type is used to marshal metrics data
// to the POST body sent to the metrics collcetor.
// Note that some metrics are labeled NA because they are expected
// to be present with their zero-value when not supported by a client.
type ValidatorStats struct {
	// N/A -- TODO(#8848): verify whether we can obtain this metric from the validator process
	ValidatorTotal int64 `json:"validator_total"`
	// N/A -- TODO(#8848): verify whether we can obtain this metric from the validator process
	ValidatorActive int64 `json:"validator_active"`
	CommonStats     `json:",inline"`
}
