package keeper

import (
	"context"
	"fmt"
	"time"

	"cosmossdk.io/core/comet"
	"cosmossdk.io/x/evidence/types"

	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// BeginBlocker iterates through and handles any newly discovered evidence of
// misbehavior submitted by CometBFT. Currently, only equivocation is handled.
func (k Keeper) BeginBlocker(ctx context.Context) error {
	defer telemetry.ModuleMeasureSince(types.ModuleName, time.Now(), telemetry.MetricKeyBeginBlocker)

	bi := k.cometInfo.GetCometBlockInfo(ctx)
	if bi == nil {
		// If we don't have block info, we don't have any evidence to process.  Block info may be nil during
		// genesis calls or in tests.
		return nil
	}

	evidences := bi.GetEvidence()
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	for i := 0; i < evidences.Len(); i++ {
		switch evidences.Get(i).Type() {
		// It's still ongoing discussion how should we treat and slash attacks with
		// premeditation. So for now we agree to treat them in the same way.
		case comet.LightClientAttack, comet.DuplicateVote:
			evidence := types.FromABCIEvidence(evidences.Get(i), k.stakingKeeper.ConsensusAddressCodec())
			err := k.handleEquivocationEvidence(ctx, evidence)
			if err != nil {
				return err
			}
		default:
			k.Logger(sdkCtx).Error(fmt.Sprintf("ignored unknown evidence type: %x", evidences.Get(i).Type()))
		}
	}
	return nil
}
