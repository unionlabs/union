package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"

	"union/x/tokenfactory/types"
)

// ConvertToBaseToken converts a fee amount in a whitelisted fee token to the base fee token amount
func (k Keeper) CreateDenom(ctx sdk.Context, creatorAddr string, subdenom string) (newTokenDenom string, err error) {
	denom, err := k.validateCreateDenom(ctx, creatorAddr, subdenom)
	if err != nil {
		return "", err
	}

	err = k.chargeForCreateDenom(ctx, creatorAddr, subdenom)
	if err != nil {
		return "", err
	}

	err = k.createDenomAfterValidation(ctx, creatorAddr, denom)
	return denom, err
}

// Runs CreateDenom logic after the charge and all denom validation has been handled.
// Made into a second function for genesis initialization.
func (k Keeper) createDenomAfterValidation(ctx sdk.Context, creatorAddr string, denom string) (err error) {
	denomMetaData := banktypes.Metadata{
		DenomUnits: []*banktypes.DenomUnit{{
			Denom:    denom,
			Exponent: 0,
		}},
		Base: denom,
		// The following is necessary for x/bank denom validation
		Display: denom,
		Name:    denom,
		Symbol:  denom,
	}

	k.bankKeeper.SetDenomMetaData(ctx, denomMetaData)

	authorityMetadata := types.DenomAuthorityMetadata{
		Admin: creatorAddr,
	}
	err = k.setAuthorityMetadata(ctx, denom, authorityMetadata)
	if err != nil {
		return err
	}

	k.addDenomFromCreator(ctx, creatorAddr, denom)
	return nil
}

func (k Keeper) validateCreateDenom(ctx sdk.Context, creatorAddr string, subdenom string) (newTokenDenom string, err error) {
	// TODO: This was a nil key on Store issue. Removed as we are upgrading IBC versions now
	// Temporary check until IBC bug is sorted out
	// if k.bankKeeper.HasSupply(ctx, subdenom) {
	// 	return "", fmt.Errorf("temporary error until IBC bug is sorted out, " +
	// 		"can't create subdenoms that are the same as a native denom")
	// }

	denom, err := types.GetTokenDenom(creatorAddr, subdenom)
	if err != nil {
		return "", err
	}

	_, found := k.bankKeeper.GetDenomMetaData(ctx, denom)
	if found {
		return "", types.ErrDenomExists
	}

	return denom, nil
}

func (k Keeper) chargeForCreateDenom(ctx sdk.Context, creatorAddr string, _ string) (err error) {
	params := k.GetParams(ctx)

	// if DenomCreationFee is non-zero, transfer the tokens from the creator
	// account to community pool
	if params.DenomCreationFee != nil {
		accAddr, err := sdk.AccAddressFromBech32(creatorAddr)
		if err != nil {
			return err
		}

		if err := k.communityPoolKeeper.FundCommunityPool(ctx, params.DenomCreationFee, accAddr); err != nil {
			return err
		}
	}

	// if DenomCreationGasConsume is non-zero, consume the gas
	if params.DenomCreationGasConsume != 0 {
		ctx.GasMeter().ConsumeGas(params.DenomCreationGasConsume, "consume denom creation gas")
	}

	return nil
}
