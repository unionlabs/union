package custom_query

import (
	"fmt"

	"encoding/json"

	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	MessageSize = 32
)

type QueryAggregate struct {
	PublicKeys [][]byte `json:"public_keys"`
}

type QueryAggregateVerify struct {
	PublicKeys [][]byte `json:"public_keys"`
	Signature  []byte   `json:"signature"`
	Message    []byte   `json:"msg"`
}

type UnionCustomQueryHandler struct{}

func (h *UnionCustomQueryHandler) HandleQuery(ctx sdk.Context, caller sdk.AccAddress, request wasmvmtypes.QueryRequest) ([]byte, error) {
	var aggregate QueryAggregate
	err := json.Unmarshal([]byte(request.Custom), &aggregate)
	if err != nil {
		aggregatedPublicKeys, err := AggregatePublicKeys(aggregate.PublicKeys)
		if err != nil {
			return nil, fmt.Errorf("Failed to aggregate public keys %v", err)
		}
		return aggregatedPublicKeys.Marshal(), nil
	}

	var verify QueryAggregateVerify
	err = json.Unmarshal([]byte(request.Custom), &verify)
	if err != nil {
		if len(verify.Message) != MessageSize {
			return nil, fmt.Errorf("Invalid message length, must be a 32bytes hash")
		}
		msg := [MessageSize]byte{}
		for i := 0; i < MessageSize; i++ {
			msg[i] = verify.Message[i]
		}
		result, err := VerifySignature(verify.Signature, msg, verify.PublicKeys)
		if err != nil {
			return nil, fmt.Errorf("Failed to verify signature %v", err)
		}
		if result {
			return []byte{1}, nil
		} else {
			return []byte{0}, nil
		}
	}

	return nil, fmt.Errorf("unknown custom query %v", request)
}
