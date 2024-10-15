package appmodulev2

import (
	"context"
	"encoding/json"
)

// HasGenesis defines a custom genesis handling API implementation.
// WARNING: this API is meant as a short-term solution to allow for the
// migration of existing modules to the new app module API.
// It is intended to be replaced by an automatic genesis with collections/orm.
type HasGenesis interface {
	AppModule

	DefaultGenesis() json.RawMessage
	ValidateGenesis(data json.RawMessage) error
	InitGenesis(ctx context.Context, data json.RawMessage) error
	ExportGenesis(ctx context.Context) (json.RawMessage, error)
}

// HasABCIGenesis defines a custom genesis handling API implementation for ABCI.
// (stateful genesis methods which returns validator updates)
// Most modules should not implement this interface.
type HasABCIGenesis interface {
	AppModule

	DefaultGenesis() json.RawMessage
	ValidateGenesis(data json.RawMessage) error
	InitGenesis(ctx context.Context, data json.RawMessage) ([]ValidatorUpdate, error)
	ExportGenesis(ctx context.Context) (json.RawMessage, error)
}

// GenesisDecoder is an alternative to the InitGenesis method.
// It is implemented by the genutil module to decode genTxs.
type GenesisDecoder interface {
	DecodeGenesisJSON(data json.RawMessage) ([]json.RawMessage, error)
}
