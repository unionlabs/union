package types

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"strings"

	"github.com/cometbft/cometbft/types"
)

const ChainIDFieldName = "chain_id"

// ParseChainIDFromGenesis parses the `chain_id` from a genesis JSON file, aborting early after finding the `chain_id`.
// For efficiency, it's recommended to place the `chain_id` field before any large entries in the JSON file.
// Returns an error if the `chain_id` field is not found.
func ParseChainIDFromGenesis(r io.Reader) (string, error) {
	dec := json.NewDecoder(r)

	t, err := dec.Token()
	if err != nil {
		return "", err
	}
	if t != json.Delim('{') {
		return "", fmt.Errorf("expected {, got %s", t)
	}

	for dec.More() {
		t, err = dec.Token()
		if err != nil {
			return "", err
		}
		key, ok := t.(string)
		if !ok {
			return "", fmt.Errorf("expected string for the key type, got %s", t)
		}

		if key == ChainIDFieldName {
			var chainId string
			if err := dec.Decode(&chainId); err != nil {
				return "", err
			}
			if err := validateChainID(chainId); err != nil {
				return "", err
			}
			return chainId, nil
		}

		// skip the value
		var value json.RawMessage
		if err := dec.Decode(&value); err != nil {
			return "", err
		}
	}

	return "", errors.New("missing chain-id in genesis file")
}

func validateChainID(chainID string) error {
	if strings.TrimSpace(chainID) == "" {
		return errors.New("genesis doc must include non-empty chain_id")
	}
	if len(chainID) > types.MaxChainIDLen {
		return fmt.Errorf("chain_id in genesis doc is too long (max: %d)", types.MaxChainIDLen)
	}

	return nil
}
