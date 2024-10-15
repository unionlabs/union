package keeper

import (
	"context"
	"errors"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/x/upgrade/types"
)

var _ types.QueryServer = Keeper{}

// CurrentPlan implements the Query/CurrentPlan gRPC method
func (k Keeper) CurrentPlan(ctx context.Context, req *types.QueryCurrentPlanRequest) (*types.QueryCurrentPlanResponse, error) {
	plan, err := k.GetUpgradePlan(ctx)
	if err != nil {
		if errors.Is(err, types.ErrNoUpgradePlanFound) {
			return &types.QueryCurrentPlanResponse{}, nil
		}

		return nil, err
	}

	return &types.QueryCurrentPlanResponse{Plan: &plan}, nil
}

// AppliedPlan implements the Query/AppliedPlan gRPC method
func (k Keeper) AppliedPlan(ctx context.Context, req *types.QueryAppliedPlanRequest) (*types.QueryAppliedPlanResponse, error) {
	applied, err := k.GetDoneHeight(ctx, req.Name)

	return &types.QueryAppliedPlanResponse{Height: applied}, err
}

// UpgradedConsensusState implements the Query/UpgradedConsensusState gRPC method
func (k Keeper) UpgradedConsensusState(ctx context.Context, req *types.QueryUpgradedConsensusStateRequest) (*types.QueryUpgradedConsensusStateResponse, error) { //nolint:staticcheck // we're using a deprecated call for compatibility
	consState, err := k.GetUpgradedConsensusState(ctx, req.LastHeight)
	if err != nil {
		if errors.Is(err, types.ErrNoUpgradedConsensusStateFound) {
			return &types.QueryUpgradedConsensusStateResponse{}, nil //nolint:staticcheck // we're using a deprecated call for compatibility
		}

		return nil, err
	}

	return &types.QueryUpgradedConsensusStateResponse{ //nolint:staticcheck // we're using a deprecated call for compatibility
		UpgradedConsensusState: consState,
	}, nil
}

// ModuleVersions implements the Query/QueryModuleVersions gRPC method
func (k Keeper) ModuleVersions(ctx context.Context, req *types.QueryModuleVersionsRequest) (*types.QueryModuleVersionsResponse, error) {
	// check if a specific module was requested
	if len(req.ModuleName) > 0 {
		version, err := k.getModuleVersion(ctx, req.ModuleName)
		if err != nil {
			// module requested, but not found or error happened
			return nil, errorsmod.Wrapf(err, "x/upgrade: QueryModuleVersions module %s not found", req.ModuleName)
		}

		// return the requested module
		res := []*types.ModuleVersion{{Name: req.ModuleName, Version: version}}
		return &types.QueryModuleVersionsResponse{ModuleVersions: res}, nil
	}

	// if no module requested return all module versions from state
	mv, err := k.GetModuleVersions(ctx)
	if err != nil {
		return nil, err
	}

	return &types.QueryModuleVersionsResponse{
		ModuleVersions: mv,
	}, nil
}

// Authority implements the Query/Authority gRPC method, returning the account capable of performing upgrades
func (k Keeper) Authority(c context.Context, req *types.QueryAuthorityRequest) (*types.QueryAuthorityResponse, error) {
	return &types.QueryAuthorityResponse{Address: k.authority}, nil
}
