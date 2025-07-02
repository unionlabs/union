package v1_2_0

import (
	"context"
	"fmt"
	"math/big"

	"cosmossdk.io/math"

	upgradetypes "cosmossdk.io/x/upgrade/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	// crisistypes "github.com/cosmos/cosmos-sdk/x/crisis/types"
	// minttypes "github.com/cosmos/cosmos-sdk/x/mint/types"

	"github.com/unionlabs/union/uniond/app/upgrades"
)

const U_BASE_DENOM = "au"

// One U in `au`
const ONE_U = 1000000000000000000

// Total supply of U (note, not in `au`)
const U_TOTAL_SUPPLY = 10000000000000
const UNION_FOUNDATION_MUTLI_SIG = "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"

func CreateUpgradeHandler(mm *module.Manager, configurator module.Configurator, keepers *upgrades.AppKeepers) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, plan upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		migrations, err := mm.RunMigrations(ctx, configurator, vm)
		if err != nil {
			return nil, err
		}

		// // Mint U
		// unionFoundationMultiSig, err := sdk.AccAddressFromBech32(UNION_FOUNDATION_MUTLI_SIG)
		// uTotalSupply := []sdk.Coin{getUFromU64(U_TOTAL_SUPPLY)}
		// keepers.MintKeeper.MintCoins(ctx, uTotalSupply)
		// keepers.BankKeeper.SendCoinsFromModuleToAccount(ctx, minttypes.ModuleName, unionFoundationMultiSig, uTotalSupply)

		// // Update x/staking
		// stakingParams, err := keepers.StakingKeeper.GetParams(ctx)
		// if err != nil {
		// 	return nil, err
		// }
		// stakingParams.BondDenom = U_BASE_DENOM
		// stakingParams.MinCommissionRate = math.LegacyMustNewDecFromStr("0.05")
		// err = keepers.StakingKeeper.SetParams(ctx, stakingParams)
		// if err != nil {
		// 	return nil, err
		// }

		// // Update x/mint
		// mintParams, err := keepers.MintKeeper.Params.Get(ctx)
		// if err != nil {
		// 	return nil, err
		// }
		// mintParams.MintDenom = U_BASE_DENOM
		// // TODO: Update mint params for U tokenomics
		// err = keepers.MintKeeper.Params.Set(ctx, mintParams)
		// if err != nil {
		// 	return nil, err
		// }

		// // Update x/gov
		// govParams, err := keepers.GovKeeper.Params.Get(ctx)
		// if err != nil {
		// 	return nil, err
		// }
		// govParams.MinDeposit = []sdk.Coin{getUFromU64(10)}
		// govParams.ExpeditedMinDeposit = []sdk.Coin{getUFromU64(50)}
		// err = keepers.GovKeeper.Params.Set(ctx, govParams)
		// if err != nil {
		// 	return nil, err
		// }

		// // Update x/crisis
		// _, err = keepers.CrisisKeeper.UpdateParams(ctx, &crisistypes.MsgUpdateParams{
		// 	Authority:   keepers.GovKeeper.GetAuthority(),
		// 	ConstantFee: getBaseUFromString("1000"),
		// })
		// if err != nil {
		// 	return nil, err
		// }

		return migrations, nil
	}
}

func getUFromU64(amount int64) sdk.Coin {
	res := new(big.Int).Mul(big.NewInt(ONE_U), big.NewInt(amount))
	return sdk.Coin{
		Denom:  U_BASE_DENOM,
		Amount: math.NewIntFromBigInt(res),
	}
}

func getBaseUFromString(amount string) sdk.Coin {
	res, ok := math.NewIntFromString(amount)
	if !ok {
		panic(fmt.Sprintf("Failed to create Int from amount: %s", amount))
	}
	return sdk.Coin{
		Denom:  U_BASE_DENOM,
		Amount: res,
	}
}
