package custom_query

import (
	"fmt"

	"encoding/json"

	sdk "github.com/cosmos/cosmos-sdk/types"

	clientkeeper "github.com/cosmos/ibc-go/v8/modules/core/02-client/keeper"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
)

const (
	MessageSize = 32
)

type CustomQuery struct {
	AggregateVerify *QueryAggregateVerify `json:"aggregate_verify,omitempty"`
	Aggregate       *QueryAggregate       `json:"aggregate,omitempty"`
	ConsensusState  *QueryConsensusState  `json:"consensus_state,omitempty"`
	ClientState     *QueryClientState     `json:"client_state,omitempty"`
}

type QueryAggregate struct {
	PublicKeys [][]byte `json:"public_keys"`
}

type QueryAggregateVerify struct {
	PublicKeys [][]byte `json:"public_keys"`
	Signature  []byte   `json:"signature"`
	Message    []byte   `json:"message"`
}

type QueryConsensusState struct {
	ClientID string             `json:"client_id"`
	Height   clienttypes.Height `json:"height"`
}

type QueryClientState struct {
	ClientID string `json:"client_id"`
}

func CustomQuerier(clientKeeper *clientkeeper.Keeper) func(sdk.Context, json.RawMessage) ([]byte, error) {
	return func(ctx sdk.Context, request json.RawMessage) ([]byte, error) {
		var customQuery CustomQuery
		err := json.Unmarshal([]byte(request), &customQuery)
		if err != nil {
			return nil, fmt.Errorf("failed to parse custom query %v", err)
		}
		if customQuery.Aggregate != nil {
			aggregatedPublicKeys, err := AggregatePublicKeys(customQuery.Aggregate.PublicKeys)
			if err != nil {
				return nil, fmt.Errorf("failed to aggregate public keys %v", err)
			}
			return json.Marshal(aggregatedPublicKeys.Marshal())
		} else if customQuery.AggregateVerify != nil {
			if len(customQuery.AggregateVerify.Message) != MessageSize {
				return nil, fmt.Errorf("invalid message length, must be a 32bytes hash: %x", customQuery.AggregateVerify.Message)
			}
			msg := [MessageSize]byte{}
			for i := 0; i < MessageSize; i++ {
				msg[i] = customQuery.AggregateVerify.Message[i]
			}
			result, err := VerifySignature(customQuery.AggregateVerify.Signature, msg, customQuery.AggregateVerify.PublicKeys)
			if err != nil {
				return nil, fmt.Errorf("failed to verify signature %v", err)
			}
			if result {
				return json.Marshal(true)
			} else {
				return json.Marshal(false)
			}
		} else if customQuery.ConsensusState != nil {
			consensusState, ok := clientKeeper.GetClientConsensusState(ctx, customQuery.ConsensusState.ClientID, customQuery.ConsensusState.Height)
			if !ok {
				return nil, fmt.Errorf("failed to query consensus state")
			}
			return json.Marshal(clientKeeper.MustMarshalConsensusState(consensusState))
		} else if customQuery.ClientState != nil {
			clientState, ok := clientKeeper.GetClientState(ctx, customQuery.ClientState.ClientID)
			if !ok {
				return nil, fmt.Errorf("failed to query client state")
			}
			return json.Marshal(clientKeeper.MustMarshalClientState(clientState))
		} else {
			return nil, fmt.Errorf("unknown custom query %v", request)
		}
	}
}
