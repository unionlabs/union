package slashing

import (
	"context"

	"cosmossdk.io/core/comet"
	"cosmossdk.io/x/slashing/keeper"
	"cosmossdk.io/x/slashing/types"

	"github.com/cosmos/cosmos-sdk/telemetry"
)

// BeginBlocker check for infraction evidence or downtime of validators
// on every begin block
func BeginBlocker(ctx context.Context, k keeper.Keeper, cometService comet.Service) error {
	start := telemetry.Now()
	defer telemetry.ModuleMeasureSince(types.ModuleName, start, telemetry.MetricKeyBeginBlocker)

	// Retrieve CometBFT info, then iterate through all validator votes
	// from the last commit. For each vote, handle the validator's signature, potentially
	// slashing or unbonding validators who have missed too many blocks.
	params, err := k.Params.Get(ctx)
	if err != nil {
		return err
	}
	ci := cometService.CometInfo(ctx)
	for _, vote := range ci.LastCommit.Votes {
		err := k.HandleValidatorSignatureWithParams(ctx, params, vote.Validator.Address, vote.Validator.Power, vote.BlockIDFlag)
		if err != nil {
			return err
		}
	}
	return nil
}
