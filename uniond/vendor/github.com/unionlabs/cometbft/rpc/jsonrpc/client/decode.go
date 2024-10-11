package client

import (
	"encoding/json"
	"errors"
	"fmt"

	cmtjson "github.com/cometbft/cometbft/libs/json"
	"github.com/cometbft/cometbft/rpc/jsonrpc/types"
)

func unmarshalResponseBytes(
	responseBytes []byte,
	expectedID types.JSONRPCIntID,
	result any,
) (any, error) {
	// Read response.  If rpc/core/types is imported, the result will unmarshal
	// into the correct type.
	response := &types.RPCResponse{}
	if err := json.Unmarshal(responseBytes, response); err != nil {
		return nil, fmt.Errorf("error unmarshalling: %w", err)
	}

	if response.Error != nil {
		return nil, response.Error
	}

	if err := validateAndVerifyID(response, expectedID); err != nil {
		return nil, fmt.Errorf("wrong ID: %w", err)
	}

	// Unmarshal the RawMessage into the result.
	if err := cmtjson.Unmarshal(response.Result, result); err != nil {
		return nil, fmt.Errorf("error unmarshalling result: %w", err)
	}

	return result, nil
}

// Separate the unmarshalling actions using different functions to improve readability and maintainability.
func unmarshalIndividualResponse(responseBytes []byte) (types.RPCResponse, error) {
	var singleResponse types.RPCResponse
	err := json.Unmarshal(responseBytes, &singleResponse)
	return singleResponse, err
}

func unmarshalMultipleResponses(responseBytes []byte) ([]types.RPCResponse, error) {
	var responses []types.RPCResponse
	err := json.Unmarshal(responseBytes, &responses)
	return responses, err
}

func unmarshalResponseBytesArray(
	responseBytes []byte,
	expectedIDs []types.JSONRPCIntID,
	results []any,
) ([]any, error) {
	var responses []types.RPCResponse

	// Try to unmarshal as multiple responses
	responses, err := unmarshalMultipleResponses(responseBytes)
	// if err == nil it could unmarshal in multiple responses
	if err == nil {
		// No response error checking here as there may be a mixture of successful
		// and unsuccessful responses.

		if len(results) != len(responses) {
			return nil, fmt.Errorf(
				"expected %d result objects into which to inject responses, but got %d",
				len(responses),
				len(results),
			)
		}

		// Intersect IDs from responses with expectedIDs.
		ids := make([]types.JSONRPCIntID, len(responses))
		var ok bool
		for i, resp := range responses {
			ids[i], ok = resp.ID.(types.JSONRPCIntID)
			if !ok {
				return nil, fmt.Errorf("expected JSONRPCIntID, got %T", resp.ID)
			}
		}
		if err := validateResponseIDs(ids, expectedIDs); err != nil {
			return nil, fmt.Errorf("wrong IDs: %w", err)
		}

		for i := 0; i < len(responses); i++ {
			if err := cmtjson.Unmarshal(responses[i].Result, results[i]); err != nil {
				return nil, fmt.Errorf("error unmarshalling #%d result: %w", i, err)
			}
		}

		return results, nil
	}
	// check if it's a single response that should be an error
	singleResponse, err := unmarshalIndividualResponse(responseBytes)
	if err != nil {
		// Here, an error means that even single response unmarshalling failed,
		// so return the error.
		return nil, fmt.Errorf("error unmarshalling: %w", err)
	}
	singleResult := make([]any, 0)
	if singleResponse.Error != nil {
		singleResult = append(singleResult, singleResponse.Error)
	} else {
		singleResult = append(singleResult, singleResponse.Result)
	}
	return singleResult, nil
}

func validateResponseIDs(ids, expectedIDs []types.JSONRPCIntID) error {
	m := make(map[types.JSONRPCIntID]bool, len(expectedIDs))
	for _, expectedID := range expectedIDs {
		m[expectedID] = true
	}

	for i, id := range ids {
		if !m[id] {
			return fmt.Errorf("unsolicited ID #%d: %v", i, id)
		}
		delete(m, id)
	}

	return nil
}

// From the JSON-RPC 2.0 spec:
// id: It MUST be the same as the value of the id member in the Request Object.
func validateAndVerifyID(res *types.RPCResponse, expectedID types.JSONRPCIntID) error {
	if err := validateResponseID(res.ID); err != nil {
		return err
	}
	if expectedID != res.ID.(types.JSONRPCIntID) { // validateResponseID ensured res.ID has the right type
		return fmt.Errorf("response ID (%d) does not match request ID (%d)", res.ID, expectedID)
	}
	return nil
}

func validateResponseID(id any) error {
	if id == nil {
		return errors.New("no ID")
	}
	_, ok := id.(types.JSONRPCIntID)
	if !ok {
		return fmt.Errorf("expected JSONRPCIntID, but got: %T", id)
	}
	return nil
}
