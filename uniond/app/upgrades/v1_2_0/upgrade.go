package v1_2_0

import (
	"context"
	"math/big"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math"

	upgradetypes "cosmossdk.io/x/upgrade/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	crisistypes "github.com/cosmos/cosmos-sdk/x/crisis/types"
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

// NOTE: must expand map with mainnet address
var unionFoundationSigMap = map[string]string{
	UNION_TESTNET: FOUNDATION_TESTNET_SIG,
	UNION_DEVNET:  DEVNET_SIG,
}

func CreateUpgradeHandler(mm *module.Manager, configurator module.Configurator, keepers *upgrades.AppKeepers) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, plan upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		migrations, err := mm.RunMigrations(ctx, configurator, vm)
		if err != nil {
			return nil, err
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

		sdkCtx.Logger().Info("total delegations", "count", len(delegations))

		for idx, delegation := range delegations {
			sdkCtx.Logger().Info(
				"delegation info",
				"idx", idx,
				"DelegatorAddress", delegation.DelegatorAddress,
				"ValidatorAddress", delegation.ValidatorAddress,
				"Shares", delegation.Shares,
			)

			delegatorAddr, err := sdk.AccAddressFromBech32(delegation.DelegatorAddress)
			if err != nil {
				return nil, err
			}
			valAddr, err := sdk.ValAddressFromBech32(delegation.ValidatorAddress)
			if err != nil {
				return nil, err
			}

			// Undelegate existing unbonding delegations
			// NOTE: Remove this first just in case it needs to read the state we manually delete after this
			unbondingDelegations, err := keepers.StakingKeeper.GetAllUnbondingDelegations(ctx, delegatorAddr)
			if err != nil {
				return nil, err
			}

			sdkCtx.Logger().Info("total unbonding delegations", "count", len(unbondingDelegations))

			for idx, unbondingDelegation := range unbondingDelegations {
				sdkCtx.Logger().Info(
					"unbonding delegation info",
					"idx", idx,
					"DelegatorAddress", unbondingDelegation.DelegatorAddress,
					"ValidatorAddress", unbondingDelegation.ValidatorAddress,
					"Entries", len(unbondingDelegation.Entries),
				)

				if err := keepers.StakingKeeper.RemoveUnbondingDelegation(ctx, unbondingDelegation); err != nil {
					return nil, err
				}
			}

			// delete all information relating to the existing validator delegations and rewards/commission
			_ = keepers.DistributionKeeper.DeleteValidatorOutstandingRewards(ctx, valAddr)
			_ = keepers.DistributionKeeper.DeleteDelegatorStartingInfo(ctx, valAddr, delegatorAddr)
			_ = keepers.DistributionKeeper.DeleteValidatorAccumulatedCommission(ctx, valAddr)
			_ = keepers.DistributionKeeper.DeleteDelegatorWithdrawAddr(
				ctx,
				delegatorAddr,
				// this is actually unused and shouldn't need to be provided lol
				// this is passed here just to make it compile
				delegatorAddr,
			)
			rewards, err := keepers.DistributionKeeper.GetValidatorCurrentRewards(ctx, valAddr)
			if err != nil {
				return nil, err
			}
			rewards.Rewards = sdk.DecCoins{}
			err = keepers.DistributionKeeper.SetValidatorCurrentRewards(ctx, valAddr, rewards)
			if err != nil {
				return nil, err
			}

			if err := keepers.StakingKeeper.RemoveDelegation(ctx, delegation); err != nil {
				return nil, err
			}
		}

		validators, err := keepers.StakingKeeper.GetAllValidators(ctx)
		if err != nil {
			return nil, err
		}

		for idx, validator := range validators {
			// the only delegation to validators is now via the foundation multisig
			validator.MinSelfDelegation = math.NewInt(0)
			// set tokens to zero as the delegate call at the end of the migration will set this
			// validator.Tokens = math.ZeroInt()
			validator, _ = validator.RemoveDelShares(validator.DelegatorShares)
			err = keepers.StakingKeeper.SetValidator(ctx, validator)
			if err != nil {
				return nil, err
			}

			if validator.IsJailed() {
				valAddr, err := sdk.ValAddressFromBech32(validator.OperatorAddress)
				if err != nil {
					return nil, err
				}
				sdkCtx.Logger().Info("validator is jailed, removing from set", "idx", idx, "addr", validator.OperatorAddress)
				err = keepers.StakingKeeper.RemoveValidator(ctx, valAddr)
				if err != nil {
					return nil, errorsmod.Wrapf(err, "unable to remove validator %s", validator.OperatorAddress)
				}
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
		// NOTE: *technically* this should be 0, but all commission will go to the community fund anyways since we set the community tax in x/distribution to 100%
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

		// Update x/distribution
		distrParams, err := keepers.DistributionKeeper.Params.Get(ctx)
		if err != nil {
			return nil, err
		}
		// set community tax to 100% to take all rewards
		distrParams.CommunityTax = math.LegacyMustNewDecFromStr("1")
		keepers.DistributionKeeper.Params.Set(ctx, distrParams)

		// Update x/feemarket
		feeMarketParams, err := keepers.FeeMarketKeeper.GetParams(sdkCtx)
		if err != nil {
			return nil, err
		}
		feeMarketParams.FeeDenom = U_BASE_DENOM
		// distribute fees rather than burning, the 100% community tax should intercept these before they're sent to the stakers (?)
		feeMarketParams.DistributeFees = true
		err = keepers.FeeMarketKeeper.SetParams(sdkCtx, feeMarketParams)
		if err != nil {
			return nil, err
		}

		// Redelegate to validators from Union foundation account
		// NOTE: This is the original delegations list, since we want to reconstruct the same validator delegations but with the foundation account being the owner of all delegations
		for idx, delegation := range delegations {
			sdkCtx.Logger().Info(
				"re-delegating delegation info",
				"idx", idx,
				"DelegatorAddress", delegation.DelegatorAddress,
				"ValidatorAddress", delegation.ValidatorAddress,
				"Shares", delegation.Shares,
			)

			valAddr, err := sdk.ValAddressFromBech32(delegation.ValidatorAddress)
			if err != nil {
				return nil, err
			}
			validator, err := keepers.StakingKeeper.GetValidator(ctx, valAddr)
			if err != nil {
				return nil, err
			}

			if validator.IsJailed() {
				sdkCtx.Logger().Info(
					"validator is jailed",
					"addr", valAddr,
				)
			} else {
				_, err = keepers.StakingKeeper.Delegate(
					ctx,
					unionFoundationMultiSig,
					delegation.Shares.RoundInt(),
					stakingtypes.Unbonded,
					validator,
					true,
				)
				if err != nil {
					return nil, err
				}
			}
		}

		// Burn old tokens
		burnToken(ctx, *keepers, "muno") // union-minimal-devnet-1 (local devnet) and union-testnet-10 gas token
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
