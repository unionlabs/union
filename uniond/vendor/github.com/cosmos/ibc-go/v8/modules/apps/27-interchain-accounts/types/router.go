package types

import (
	baseapp "github.com/cosmos/cosmos-sdk/baseapp"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// MessageRouter ADR 031 request type routing
// https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-031-msg-service.md
type MessageRouter interface {
	Handler(msg sdk.Msg) baseapp.MsgServiceHandler
}

// QueryRouter ADR 021 query type routing
// https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-021-protobuf-query-encoding.md
type QueryRouter interface {
	// Route returns the GRPCQueryHandler for a given query route path or nil
	// if not found
	Route(path string) baseapp.GRPCQueryHandler
}
