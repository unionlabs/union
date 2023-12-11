package types

import (
	"encoding/json"
	"fmt"
	"slices"

	wasmvmtypes "github.com/CosmWasm/wasmvm/types"

	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"

	abci "github.com/cometbft/cometbft/abci/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
)

/*
`queryHandler` is a contextual querier which references the global `ibcwasm.QueryPluginsI`
to handle queries. The global `ibcwasm.QueryPluginsI` points to a `types.QueryPlugins` which
contains two sub-queriers: `types.CustomQuerier` and `types.StargateQuerier`. These sub-queriers
can be replaced by the user through the options api in the keeper.

In addition, the `types.StargateQuerier` references a global `ibcwasm.QueryRouter` which points
to `baseapp.GRPCQueryRouter`.

This design is based on wasmd's (v0.50.0) querier plugin design.
*/

var (
	_ wasmvmtypes.Querier   = (*queryHandler)(nil)
	_ ibcwasm.QueryPluginsI = (*QueryPlugins)(nil)
)

// queryHandler is a wrapper around the sdk.Context and the CallerID that calls
// into the query plugins.
type queryHandler struct {
	Ctx      sdk.Context
	CallerID string
}

// newQueryHandler returns a default querier that can be used in the contract.
func newQueryHandler(ctx sdk.Context, callerID string) *queryHandler {
	return &queryHandler{
		Ctx:      ctx,
		CallerID: callerID,
	}
}

// GasConsumed implements the wasmvmtypes.Querier interface.
func (q *queryHandler) GasConsumed() uint64 {
	return VMGasRegister.ToWasmVMGas(q.Ctx.GasMeter().GasConsumed())
}

// Query implements the wasmvmtypes.Querier interface.
func (q *queryHandler) Query(request wasmvmtypes.QueryRequest, gasLimit uint64) ([]byte, error) {
	sdkGas := VMGasRegister.FromWasmVMGas(gasLimit)

	// discard all changes/events in subCtx by not committing the cached context
	subCtx, _ := q.Ctx.WithGasMeter(storetypes.NewGasMeter(sdkGas)).CacheContext()

	// make sure we charge the higher level context even on panic
	defer func() {
		q.Ctx.GasMeter().ConsumeGas(subCtx.GasMeter().GasConsumed(), "contract sub-query")
	}()

	res, err := ibcwasm.GetQueryPlugins().HandleQuery(subCtx, q.CallerID, request)
	if err == nil {
		return res, nil
	}

	Logger(q.Ctx).Debug("Redacting query error", "cause", err)
	return nil, redactError(err)
}

type (
	CustomQuerier   func(ctx sdk.Context, request json.RawMessage) ([]byte, error)
	StargateQuerier func(ctx sdk.Context, request *wasmvmtypes.StargateQuery) ([]byte, error)

	// QueryPlugins is a list of queriers that can be used to extend the default querier.
	QueryPlugins struct {
		Custom   CustomQuerier
		Stargate StargateQuerier
	}
)

// Merge merges the query plugin with a provided one.
func (e QueryPlugins) Merge(x *QueryPlugins) QueryPlugins {
	// only update if this is non-nil and then only set values
	if x == nil {
		return e
	}

	if x.Custom != nil {
		e.Custom = x.Custom
	}

	if x.Stargate != nil {
		e.Stargate = x.Stargate
	}

	return e
}

// HandleQuery implements the ibcwasm.QueryPluginsI interface.
func (e QueryPlugins) HandleQuery(ctx sdk.Context, caller string, request wasmvmtypes.QueryRequest) ([]byte, error) {
	if request.Stargate != nil {
		return e.Stargate(ctx, request.Stargate)
	}

	if request.Custom != nil {
		return e.Custom(ctx, request.Custom)
	}

	return nil, wasmvmtypes.UnsupportedRequest{Kind: "Unsupported query request"}
}

// NewDefaultQueryPlugins returns the default set of query plugins
func NewDefaultQueryPlugins() *QueryPlugins {
	return &QueryPlugins{
		Custom:   RejectCustomQuerier(),
		Stargate: AcceptListStargateQuerier([]string{}),
	}
}

// AcceptListStargateQuerier allows all queries that are in the provided accept list.
// This function returns protobuf encoded responses in bytes.
func AcceptListStargateQuerier(acceptedQueries []string) func(sdk.Context, *wasmvmtypes.StargateQuery) ([]byte, error) {
	return func(ctx sdk.Context, request *wasmvmtypes.StargateQuery) ([]byte, error) {
		// A default list of accepted queries can be added here.
		// accepted = append(defaultAcceptList, accepted...)

		isAccepted := slices.Contains(acceptedQueries, request.Path)
		if !isAccepted {
			return nil, wasmvmtypes.UnsupportedRequest{Kind: fmt.Sprintf("'%s' path is not allowed from the contract", request.Path)}
		}

		route := ibcwasm.GetQueryRouter().Route(request.Path)
		if route == nil {
			return nil, wasmvmtypes.UnsupportedRequest{Kind: fmt.Sprintf("No route to query '%s'", request.Path)}
		}

		res, err := route(ctx, &abci.RequestQuery{
			Data: request.Data,
			Path: request.Path,
		})
		if err != nil {
			return nil, err
		}
		if res == nil || res.Value == nil {
			return nil, wasmvmtypes.InvalidResponse{Err: "Query response is empty"}
		}

		return res.Value, nil
	}
}

// RejectCustomQuerier rejects all custom queries
func RejectCustomQuerier() func(sdk.Context, json.RawMessage) ([]byte, error) {
	return func(ctx sdk.Context, request json.RawMessage) ([]byte, error) {
		return nil, wasmvmtypes.UnsupportedRequest{Kind: "Custom queries are not allowed"}
	}
}

// Wasmd Issue [#759](https://github.com/CosmWasm/wasmd/issues/759)
// Don't return error string for worries of non-determinism
func redactError(err error) error {
	// Do not redact system errors
	// SystemErrors must be created in 08-wasm and we can ensure determinism
	if wasmvmtypes.ToSystemError(err) != nil {
		return err
	}

	codespace, code, _ := errorsmod.ABCIInfo(err, false)
	return fmt.Errorf("codespace: %s, code: %d", codespace, code)
}
