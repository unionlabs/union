package config

import "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"

type GetDepositContractResponse struct {
	Data *DepositContractData `json:"data"`
}

type DepositContractData struct {
	ChainId string `json:"chain_id"`
	Address string `json:"address"`
}

type GetForkScheduleResponse struct {
	Data []*shared.Fork `json:"data"`
}

type GetSpecResponse struct {
	Data interface{} `json:"data"`
}
