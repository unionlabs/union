package types

import (
	"encoding/json"
)

// isJSONObjectWithTopLevelKey returns true if the given bytes are a valid JSON object
// with exactly one top-level key that is contained in the list of allowed keys.
func isJSONObjectWithTopLevelKey(jsonBytes RawContractMessage, allowedKeys []string) (bool, error) {
	if err := jsonBytes.ValidateBasic(); err != nil {
		return false, err
	}

	document := map[string]interface{}{}
	if err := json.Unmarshal(jsonBytes, &document); err != nil {
		return false, nil // not a map
	}

	if len(document) != 1 {
		return false, nil // unsupported type
	}

	// Loop is executed exactly once
	for topLevelKey := range document {
		for _, allowedKey := range allowedKeys {
			if allowedKey == topLevelKey {
				return true, nil
			}
		}
		return false, nil
	}

	panic("Reached unreachable code. This is a bug.")
}
