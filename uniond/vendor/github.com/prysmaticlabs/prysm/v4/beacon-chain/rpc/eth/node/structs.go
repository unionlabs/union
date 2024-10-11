package node

type SyncStatusResponse struct {
	Data *SyncStatusResponseData `json:"data"`
}

type SyncStatusResponseData struct {
	HeadSlot     string `json:"head_slot"`
	SyncDistance string `json:"sync_distance"`
	IsSyncing    bool   `json:"is_syncing"`
	IsOptimistic bool   `json:"is_optimistic"`
	ElOffline    bool   `json:"el_offline"`
}

type GetIdentityResponse struct {
	Data *Identity `json:"data"`
}

type Identity struct {
	PeerId             string    `json:"peer_id"`
	Enr                string    `json:"enr"`
	P2PAddresses       []string  `json:"p2p_addresses"`
	DiscoveryAddresses []string  `json:"discovery_addresses"`
	Metadata           *Metadata `json:"metadata"`
}

type Metadata struct {
	SeqNumber string `json:"seq_number"`
	Attnets   string `json:"attnets"`
}

type GetPeerResponse struct {
	Data *Peer `json:"data"`
}

type GetPeersResponse struct {
	Data []*Peer `json:"data"`
}

type Peer struct {
	PeerId             string `json:"peer_id"`
	Enr                string `json:"enr"`
	LastSeenP2PAddress string `json:"last_seen_p2p_address"`
	State              string `json:"state"`
	Direction          string `json:"direction"`
}

type GetPeerCountResponse struct {
	Data *PeerCount `json:"data"`
}

type PeerCount struct {
	Disconnected  string `json:"disconnected"`
	Connecting    string `json:"connecting"`
	Connected     string `json:"connected"`
	Disconnecting string `json:"disconnecting"`
}

type GetVersionResponse struct {
	Data *Version `json:"data"`
}

type Version struct {
	Version string `json:"version"`
}
