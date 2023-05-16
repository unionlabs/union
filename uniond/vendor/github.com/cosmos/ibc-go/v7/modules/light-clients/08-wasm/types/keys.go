package types

import (
	"encoding/hex"
	"fmt"
)

const (
	// ModuleName for the wasm client
	ModuleName = "08-wasm"

	// StoreKey is the store key string for 08-wasm
	StoreKey = ModuleName

	KeyCodeIDPrefix = "codeId"
)

func CodeIDKey(codeID []byte) []byte {
	return []byte(fmt.Sprintf("%s/%s", KeyCodeIDPrefix, hex.EncodeToString(codeID)))
}
