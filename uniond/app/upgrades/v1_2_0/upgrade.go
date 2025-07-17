package v1_2_0

import (
	"context"
	"math/big"

	"cosmossdk.io/math"

	upgradetypes "cosmossdk.io/x/upgrade/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	crisistypes "github.com/cosmos/cosmos-sdk/x/crisis/types"
	distributiontypes "github.com/cosmos/cosmos-sdk/x/distribution/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
	minttypes "github.com/cosmos/cosmos-sdk/x/mint/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"

	"github.com/unionlabs/union/uniond/app/upgrades"
)

const U_BASE_DENOM = "au"

// One U in `au`
const ONE_U = 1_000_000_000_000_000_000

// Total supply of U (note, not in `au`)
const U_TOTAL_SUPPLY = 10_000_000_000

// Union foundation multisig address
const FOUNDATION_TESTNET_SIG = "union1cpz5fhesgjcv2q0640uxtyur5ju65av6r8fem0"
const DEVNET_SIG = "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"

const UNION_DEVNET = "union-minimal-devnet-1"
const UNION_TESTNET = "union-testnet-10"

func CreateUpgradeHandler(mm *module.Manager, configurator module.Configurator, keepers *upgrades.AppKeepers) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, plan upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		migrations, err := mm.RunMigrations(ctx, configurator, vm)
		if err != nil {
			return nil, err
		}

		// NOTE: must expand map with mainnet address
		unionFoundationSigMap := map[string]string{
			UNION_TESTNET: FOUNDATION_TESTNET_SIG,
			UNION_DEVNET:  DEVNET_SIG,
		}

		sdkCtx := sdk.UnwrapSDKContext(ctx)
		unionFoundationMultiSig, err := sdk.AccAddressFromBech32(unionFoundationSigMap[sdkCtx.ChainID()])
		if err != nil {
			return nil, err
		}

		// Undelegate existing delegations
		delegations, err := keepers.StakingKeeper.GetAllDelegations(ctx)
		if err != nil {
			return nil, err
		}

		for idx, delegation := range delegations {
			sdkCtx.Logger().Info(
				"delegation info",
				"idx", idx,
				"DelegatorAddress", delegation.DelegatorAddress,
				"ValidatorAddress", delegation.ValidatorAddress,
				"Shares", delegation.Shares,
			)

			accAddr, err := sdk.AccAddressFromBech32(delegation.DelegatorAddress)
			if err != nil {
				return nil, err
			}
			valAddr, err := sdk.ValAddressFromBech32(delegation.ValidatorAddress)
			if err != nil {
				return nil, err
			}

			_, _ = keepers.DistributionKeeper.WithdrawDelegationRewards(ctx, accAddr, valAddr)
			sdkCtx.Logger().Info("withdrew rewards")
			_, _ = keepers.DistributionKeeper.WithdrawValidatorCommission(ctx, valAddr)
			sdkCtx.Logger().Info("withdrew validator commission")

			validator, err := keepers.StakingKeeper.GetValidator(ctx, valAddr)
			if err != nil {
				return nil, err
			}
			validator.MinSelfDelegation = math.NewInt(0)
			keepers.StakingKeeper.SetValidator(ctx, validator)

			_, _, err = keepers.StakingKeeper.Undelegate(ctx, accAddr, valAddr, delegation.Shares)
			if err != nil {
				return nil, err
			}
		}

		// Mint U
		uTotalSupply := []sdk.Coin{getUFromU64(U_TOTAL_SUPPLY)}
		err = keepers.MintKeeper.MintCoins(ctx, uTotalSupply)
		if err != nil {
			return nil, err
		}
		err = keepers.BankKeeper.SendCoinsFromModuleToAccount(ctx, minttypes.ModuleName, unionFoundationMultiSig, uTotalSupply)
		if err != nil {
			return nil, err
		}

		// Update x/staking
		stakingParams, err := keepers.StakingKeeper.GetParams(ctx)
		if err != nil {
			return nil, err
		}
		stakingParams.BondDenom = U_BASE_DENOM
		stakingParams.MinCommissionRate = math.LegacyMustNewDecFromStr("0.05")
		err = keepers.StakingKeeper.SetParams(ctx, stakingParams)
		if err != nil {
			return nil, err
		}

		// Update x/mint
		// NOTE: Keeping inflation set to 0 until TGE
		mintparams, err := keepers.MintKeeper.Params.Get(ctx)
		if err != nil {
			return nil, err
		}
		mintparams.MintDenom = U_BASE_DENOM
		err = keepers.MintKeeper.Params.Set(ctx, mintparams)
		if err != nil {
			return nil, err
		}

		// Update x/gov
		govParams, err := keepers.GovKeeper.Params.Get(ctx)
		if err != nil {
			return nil, err
		}
		govParams.MinDeposit = []sdk.Coin{getUFromU64(10)}
		govParams.ExpeditedMinDeposit = []sdk.Coin{getUFromU64(50)}
		err = keepers.GovKeeper.Params.Set(ctx, govParams)
		if err != nil {
			return nil, err
		}

		// Update x/crisis
		_, err = keepers.CrisisKeeper.UpdateParams(ctx, &crisistypes.MsgUpdateParams{
			Authority:   keepers.GovKeeper.GetAuthority(),
			ConstantFee: getUFromU64(1),
		})
		if err != nil {
			return nil, err
		}

		sdkCtx = sdk.UnwrapSDKContext(ctx)

		// Update x/distribution
		distrParams, err := keepers.DistributionKeeper.Params.Get(ctx)
		if err != nil {
			return nil, err
		}
		distrParams.CommunityTax = math.LegacyMustNewDecFromStr("1")
		keepers.DistributionKeeper.Params.Set(ctx, distrParams)

		// Update x/feemarket
		feeMarketParams, err := keepers.FeeMarketKeeper.GetParams(sdkCtx)
		if err != nil {
			return nil, err
		}
		feeMarketParams.FeeDenom = U_BASE_DENOM
		feeMarketParams.DistributeFees = true
		err = keepers.FeeMarketKeeper.SetParams(sdkCtx, feeMarketParams)
		if err != nil {
			return nil, err
		}

		// Redelegate to validators from Union foundation account
		for _, delegation := range delegations {
			valAddr, err := sdk.ValAddressFromBech32(delegation.ValidatorAddress)
			if err != nil {
				return nil, err
			}
			validator, err := keepers.StakingKeeper.GetValidator(ctx, valAddr)
			if err != nil {
				return nil, err
			}

			keepers.DistributionKeeper.SetValidatorOutstandingRewards(ctx, valAddr, distributiontypes.ValidatorOutstandingRewards{})

			_, err = keepers.StakingKeeper.Delegate(ctx, unionFoundationMultiSig, delegation.Shares.RoundInt(), stakingtypes.Unbonded, validator, true)
			if err != nil {
				return nil, err
			}
		}

		// Burn old tokens
		burnToken(ctx, *keepers, "muno") // union-testnet-10 gas token
		burnToken(ctx, *keepers, "upoa") // union-1 and union-testnet-10 PoA token
		burnToken(ctx, *keepers, "ugas") // union-1 gas token

		return migrations, nil
	}
}

func burnToken(ctx context.Context, keepers upgrades.AppKeepers, denom string) error {
	tokenOwners, err := keepers.BankKeeper.DenomOwners(ctx, &banktypes.QueryDenomOwnersRequest{
		Denom:      denom,
		Pagination: nil,
	})
	if err != nil {
		return err
	}
	tokenSum := math.ZeroInt()
	for _, tokenOwner := range tokenOwners.DenomOwners {
		accAddr, err := sdk.AccAddressFromBech32(tokenOwner.Address)
		if err != nil {
			return err
		}
		err = keepers.BankKeeper.SendCoinsFromAccountToModule(ctx, accAddr, govtypes.ModuleName, sdk.NewCoins(tokenOwner.Balance))

		if err != nil {
			return err
		}
		tokenSum = tokenSum.Add(tokenOwner.Balance.Amount)
	}
	if tokenSum.GT(math.ZeroInt()) {
		err = keepers.BankKeeper.BurnCoins(ctx, govtypes.ModuleName, sdk.NewCoins(sdk.Coin{
			Denom:  denom,
			Amount: tokenSum,
		}))
		if err != nil {
			return err
		}
	}
	return nil
}

func getUFromU64(amount int64) sdk.Coin {
	res := new(big.Int).Mul(big.NewInt(ONE_U), big.NewInt(amount))
	return sdk.Coin{
		Denom:  U_BASE_DENOM,
		Amount: math.NewIntFromBigInt(res),
	}
}
