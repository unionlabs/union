package beacon

import "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"

type GetWeakSubjectivityResponse struct {
	Data *WeakSubjectivityData `json:"data"`
}

type WeakSubjectivityData struct {
	WsCheckpoint *shared.Checkpoint `json:"ws_checkpoint"`
	StateRoot    string             `json:"state_root"`
}
