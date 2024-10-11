package keeper

import (
	"context"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/event"
	"cosmossdk.io/math"
	"cosmossdk.io/x/mint/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// Keeper of the mint store
type Keeper struct {
	appmodule.Environment

	cdc              codec.BinaryCodec
	stakingKeeper    types.StakingKeeper
	bankKeeper       types.BankKeeper
	feeCollectorName string
	// the address capable of executing a MsgUpdateParams message. Typically, this
	// should be the x/gov module account.
	authority string

	Schema collections.Schema
	Params collections.Item[types.Params]
	Minter collections.Item[types.Minter]
}

// NewKeeper creates a new mint Keeper instance
func NewKeeper(
	cdc codec.BinaryCodec,
	env appmodule.Environment,
	sk types.StakingKeeper,
	ak types.AccountKeeper,
	bk types.BankKeeper,
	feeCollectorName string,
	authority string,
) Keeper {
	// ensure mint module account is set
	if addr := ak.GetModuleAddress(types.ModuleName); addr == nil {
		panic(fmt.Sprintf("the x/%s module account has not been set", types.ModuleName))
	}

	sb := collections.NewSchemaBuilder(env.KVStoreService)
	k := Keeper{
		Environment:      env,
		cdc:              cdc,
		stakingKeeper:    sk,
		bankKeeper:       bk,
		feeCollectorName: feeCollectorName,
		authority:        authority,
		Params:           collections.NewItem(sb, types.ParamsKey, "params", codec.CollValue[types.Params](cdc)),
		Minter:           collections.NewItem(sb, types.MinterKey, "minter", codec.CollValue[types.Minter](cdc)),
	}

	schema, err := sb.Build()
	if err != nil {
		panic(err)
	}
	k.Schema = schema
	return k
}

// GetAuthority returns the x/mint module's authority.
func (k Keeper) GetAuthority() string {
	return k.authority
}

// StakingTokenSupply implements an alias call to the underlying staking keeper's
// StakingTokenSupply to be used in BeginBlocker.
func (k Keeper) StakingTokenSupply(ctx context.Context) (math.Int, error) {
	return k.stakingKeeper.StakingTokenSupply(ctx)
}

// BondedRatio implements an alias call to the underlying staking keeper's
// BondedRatio to be used in BeginBlocker.
func (k Keeper) BondedRatio(ctx context.Context) (math.LegacyDec, error) {
	return k.stakingKeeper.BondedRatio(ctx)
}

// MintCoins implements an alias call to the underlying supply keeper's
// MintCoins to be used in BeginBlocker.
func (k Keeper) MintCoins(ctx context.Context, newCoins sdk.Coins) error {
	if newCoins.Empty() {
		// skip as no coins need to be minted
		return nil
	}

	return k.bankKeeper.MintCoins(ctx, types.ModuleName, newCoins)
}

// AddCollectedFees implements an alias call to the underlying supply keeper's
// AddCollectedFees to be used in BeginBlocker.
func (k Keeper) AddCollectedFees(ctx context.Context, fees sdk.Coins) error {
	return k.bankKeeper.SendCoinsFromModuleToModule(ctx, types.ModuleName, k.feeCollectorName, fees)
}

func (k Keeper) DefaultMintFn(ic types.InflationCalculationFn) types.MintFn {
	return func(ctx context.Context, env appmodule.Environment, minter *types.Minter, epochId string, epochNumber int64) error {
		// the default mint function is called every block, so we only check if epochId is "block" which is
		// a special value to indicate that this is not an epoch minting, but a regular block minting.
		if epochId != "block" {
			return nil
		}

		stakingTokenSupply, err := k.StakingTokenSupply(ctx)
		if err != nil {
			return err
		}

		bondedRatio, err := k.BondedRatio(ctx)
		if err != nil {
			return err
		}

		params, err := k.Params.Get(ctx)
		if err != nil {
			return err
		}

		minter.Inflation = ic(ctx, *minter, params, bondedRatio)
		minter.AnnualProvisions = minter.NextAnnualProvisions(params, stakingTokenSupply)

		mintedCoin := minter.BlockProvision(params)
		mintedCoins := sdk.NewCoins(mintedCoin)
		maxSupply := params.MaxSupply
		totalSupply := stakingTokenSupply

		// if maxSupply is not infinite, check against max_supply parameter
		if !maxSupply.IsZero() {
			if totalSupply.Add(mintedCoins.AmountOf(params.MintDenom)).GT(maxSupply) {
				// calculate the difference between maxSupply and totalSupply
				diff := maxSupply.Sub(totalSupply)
				if diff.LTE(math.ZeroInt()) {
					k.Environment.Logger.Info("max supply reached, no new tokens will be minted")
					return nil
				}

				// mint the difference
				diffCoin := sdk.NewCoin(params.MintDenom, diff)
				diffCoins := sdk.NewCoins(diffCoin)

				// mint coins
				if err := k.MintCoins(ctx, diffCoins); err != nil {
					return err
				}
				mintedCoins = diffCoins
			}
		}

		// mint coins if maxSupply is infinite or total staking supply is less than maxSupply
		if maxSupply.IsZero() || totalSupply.Add(mintedCoins.AmountOf(params.MintDenom)).LT(maxSupply) {
			// mint coins
			if err := k.MintCoins(ctx, mintedCoins); err != nil {
				return err
			}
		}

		// send the minted coins to the fee collector account
		// TODO: figure out a better way to do this
		err = k.AddCollectedFees(ctx, mintedCoins)
		if err != nil {
			return err
		}

		if mintedCoin.Amount.IsInt64() {
			defer telemetry.ModuleSetGauge(types.ModuleName, float32(mintedCoin.Amount.Int64()), "minted_tokens")
		}

		return env.EventService.EventManager(ctx).EmitKV(
			types.EventTypeMint,
			event.NewAttribute(types.AttributeKeyBondedRatio, bondedRatio.String()),
			event.NewAttribute(types.AttributeKeyInflation, minter.Inflation.String()),
			event.NewAttribute(types.AttributeKeyAnnualProvisions, minter.AnnualProvisions.String()),
			event.NewAttribute(sdk.AttributeKeyAmount, mintedCoin.Amount.String()),
		)
	}
}
