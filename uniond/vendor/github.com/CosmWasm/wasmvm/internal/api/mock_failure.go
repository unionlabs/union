package api

import (
	"fmt"

	"github.com/CosmWasm/wasmvm/types"
)

/***** Mock types.GoAPI ****/

func MockFailureCanonicalAddress(human string) ([]byte, uint64, error) {
	return nil, 0, fmt.Errorf("mock failure - canonical_address")
}

func MockFailureHumanAddress(canon []byte) (string, uint64, error) {
	return "", 0, fmt.Errorf("mock failure - human_address")
}

func NewMockFailureAPI() *types.GoAPI {
	return &types.GoAPI{
		HumanAddress:     MockFailureHumanAddress,
		CanonicalAddress: MockFailureCanonicalAddress,
	}
}
