package v2

import (
	"context"
	"encoding/json"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// AppExporter is a function that dumps all app state to
// JSON-serializable structure and returns the current validator set.
type AppExporter func(
	ctx context.Context,
	height int64,
	jailAllowedAddrs []string,
) (ExportedApp, error)

// ExportedApp represents an exported app state, along with
// validators, consensus params and latest app height.
type ExportedApp struct {
	// AppState is the application state as JSON.
	AppState json.RawMessage
	// Height is the app's latest block height.
	Height int64
	// Validators is the exported validator set.
	Validators []sdk.GenesisValidator
}
