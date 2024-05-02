package keeper

import (
	"context"
	"errors"
	"slices"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	abci "github.com/cometbft/cometbft/abci/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

var _ types.MsgServer = (*msgServer)(nil)

type msgServer struct {
	*Keeper
}

// NewMsgServerImpl returns an implementation of the ICS27 host MsgServer interface
// for the provided Keeper.
func NewMsgServerImpl(keeper *Keeper) types.MsgServer {
	if keeper.queryRouter == nil {
		panic("query router must not be nil")
	}
	return &msgServer{Keeper: keeper}
}

// ModuleQuerySafe routes the queries to the keeper's query router if they are module_query_safe.
// This handler doesn't use the signer.
func (m msgServer) ModuleQuerySafe(goCtx context.Context, msg *types.MsgModuleQuerySafe) (*types.MsgModuleQuerySafeResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	responses := make([][]byte, len(msg.Requests))
	for i, query := range msg.Requests {
		isModuleQuerySafe := slices.Contains(m.mqsAllowList, query.Path)
		if !isModuleQuerySafe {
			return nil, errorsmod.Wrapf(ibcerrors.ErrInvalidRequest, "not module query safe: %s", query.Path)
		}

		if m.queryRouter == nil {
			return nil, errors.New("query router must not be nil")
		}

		route := m.queryRouter.Route(query.Path)
		if route == nil {
			return nil, errorsmod.Wrapf(ibcerrors.ErrInvalidRequest, "no route to query: %s", query.Path)
		}

		res, err := route(ctx, &abci.RequestQuery{
			Path: query.Path,
			Data: query.Data,
		})
		if err != nil {
			m.Logger(ctx).Debug("query failed", "path", query.Path, "error", err)
			return nil, err
		}
		if res == nil || res.Value == nil {
			return nil, errorsmod.Wrapf(ibcerrors.ErrInvalidRequest, "no response for query: %s", query.Path)
		}

		responses[i] = res.Value
	}

	return &types.MsgModuleQuerySafeResponse{Responses: responses, Height: uint64(ctx.BlockHeight())}, nil
}

// UpdateParams updates the host submodule's params.
func (m msgServer) UpdateParams(goCtx context.Context, msg *types.MsgUpdateParams) (*types.MsgUpdateParamsResponse, error) {
	if m.GetAuthority() != msg.Signer {
		return nil, errorsmod.Wrapf(ibcerrors.ErrUnauthorized, "expected %s, got %s", m.GetAuthority(), msg.Signer)
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	m.SetParams(ctx, msg.Params)

	return &types.MsgUpdateParamsResponse{}, nil
}
