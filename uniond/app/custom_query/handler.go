package custom_query

import (
	"fmt"

	"encoding/json"

	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
)

const (
	MessageSize = 32
)

type CustomQuery struct {
	AggregateVerify *QueryAggregateVerify `json:"aggregate_verify,omitempty"`
	Aggregate       *QueryAggregate       `json:"aggregate,omitempty"`
}

type QueryAggregate struct {
	PublicKeys [][]byte `json:"public_keys"`
}

type QueryAggregateVerify struct {
	PublicKeys [][]byte `json:"public_keys"`
	Signature  []byte   `json:"signature"`
	Message    []byte   `json:"message"`
}

type UnionCustomQueryHandler struct{}

func (h *UnionCustomQueryHandler) GasConsumed() uint64 {
	return 0
}

// TODO: /!\ verify gasLimit <= the gas we wanna consume and update GasConsumed()
func (h *UnionCustomQueryHandler) Query(request wasmvmtypes.QueryRequest, gasLimit uint64) ([]byte, error) {
	var customQuery CustomQuery
	err := json.Unmarshal([]byte(request.Custom), &customQuery)

	if err != nil {
		return nil, fmt.Errorf("Failed to parse custom query %v", err)
	}

	if customQuery.Aggregate != nil {
		aggregatedPublicKeys, err := AggregatePublicKeys(customQuery.Aggregate.PublicKeys)
		if err != nil {
			return nil, fmt.Errorf("Failed to aggregate public keys %v", err)
		}
		return json.Marshal(aggregatedPublicKeys.Marshal())
	} else if customQuery.AggregateVerify != nil {
		if len(customQuery.AggregateVerify.Message) != MessageSize {
			return nil, fmt.Errorf("Invalid message length, must be a 32bytes hash", customQuery.AggregateVerify.Message)
		}
		msg := [MessageSize]byte{}
		for i := 0; i < MessageSize; i++ {
			msg[i] = customQuery.AggregateVerify.Message[i]
		}
		result, err := VerifySignature(customQuery.AggregateVerify.Signature, msg, customQuery.AggregateVerify.PublicKeys)
		if err != nil {
			return nil, fmt.Errorf("Failed to verify signature %v", err)
		}
		if result {
			return json.Marshal(true)
		} else {
			return json.Marshal(false)
		}
	} else {
		return nil, fmt.Errorf("unknown custom query %v", request)
	}
}
