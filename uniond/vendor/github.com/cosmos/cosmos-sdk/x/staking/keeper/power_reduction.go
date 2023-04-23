package keeper

import (
	"cosmossdk.io/math"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// TokensToConsensusPower - convert input tokens to potential consensus-engine power
func (k Keeper) TokensToConsensusPower(ctx sdk.Context, tokens math.Int) int64 {
	return sdk.TokensToConsensusPower(tokens, k.PowerReduction(ctx))
}

// TokensFromConsensusPower - convert input power to tokens
func (k Keeper) TokensFromConsensusPower(ctx sdk.Context, power int64) math.Int {
	return sdk.TokensFromConsensusPower(power, k.PowerReduction(ctx))
}
