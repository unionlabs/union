package v1_3_0

import (
	"bytes"
	"context"
	sdkmath "cosmossdk.io/math"
	"math"

	storetypes "cosmossdk.io/store/types"
	upgradetypes "cosmossdk.io/x/upgrade/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	query "github.com/cosmos/cosmos-sdk/types/query"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"

	"github.com/unionlabs/union/uniond/app/upgrades"
)

const UNION_DEVNET = "union-minimal-devnet-1"
const UNION_TESTNET = "union-testnet-10"
const UNION_MAINNET = "union-1"

var valsToUpdateByHex = map[string][]string{
	UNION_DEVNET: {
		"006BC15F2B541A5D8E15D5B3B8668D59678156D1",
		"6C6A76D96D2CC4F26BA915E3D15251283B7C1638",
		"EC0D051689E13D20DCE13E40BC2867119086E14F",
		"4D97DFF94555E3ABF91F4A69968285605A99FA8F",
	},
	UNION_TESTNET: {},
	UNION_MAINNET: {
		"0BF7B6DA19C3E6B29756289BCAD156EAA3587D1A",
		"61D76C2E7169F9412D8458B8CE25B578663276C5",
		"690E5C028AE228DB98E1FF8D1BB6DEE81754EDD0",
		"ECD081172FAC91590F0A218CFC0F62ACAF9D8E07",
		"D8585432D1C6016188BEEE80B383211FED75C6B9",
		"D64DDAAB6529FEF5EA662815387172B6B2D9BF1C",
		"21FBB9A9E0A1FB36B10D04D08D64E5C031D52E3C",
		"19A76B6253EB01ADF71C6A13CF5ECAC6BA45260E",
		"68581C99A8788A81170B3D3536854E3B77049699",
		"436486B86541B2BD1BC0B29AE4C60619320755F7",
		"344C49AFDFD83ED66B346CF01A0E3B835BB61AFF",
		"FBC4E0698E3A5F8F46E38C0FEDC68FB05D222A45",
		"2572C13DE20E0978F2EADDC0C356ECA52AAD035D",
		"935B20F1DF3B3FAC36AD855BA480C229991B428C",
		"2EC777A79E10A02FF01D3C34F621BCC52E67C048",
		"0DCFCE6736DE6F23F0D8C5CD7AE82CABDA6A004B",
		"770A98B6F021EC71AFF1BF13036C781EEB6A99A3",
		"FA1827E156C268DA238DBF4F906E77EE60DD933A",
		"7F10B83A1F652DF1798C6D6F374B7909F2CB13A5",
		"3DF2F50E9F26E9657F10B787076D4A4A12142052",
		"BB5B9E1B00971854D1D7F9D17F5C8AFF0A8F52E3",
		"29B9F86E18528780A4B85D1C57C6F2FB909A369C",
		"F61504BDCD37FD3A5CFE1752CCB938F7A33D349A",
		"CD65B5013B5AE901A0121952FCE19E7DBF18A088",
	},
}

func CreateUpgradeHandler(mm *module.Manager, configurator module.Configurator, keepers *upgrades.AppKeepers, getKey upgrades.GetKeyFunc) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, plan upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		migrations, err := mm.RunMigrations(ctx, configurator, vm)
		if err != nil {
			return nil, err
		}

		sdkCtx := sdk.UnwrapSDKContext(ctx)

		validators, err := keepers.StakingKeeper.GetAllValidators(ctx)
		if err != nil {
			return nil, err
		}

		if sdkCtx.ChainID() == UNION_TESTNET {
			// adjust power reduction of all validators
			// Adapted from https://github.com/DoraFactory/doravota/blob/b735125f0dfa2a0f50afac0685aa73698cf2227f/app/app.go#L1079-L1114
			sdkCtx.Logger().Info("resetting validator state")
			for _, validator := range validators {

				// Direct access to the staking store
				// Staking interfaces were not sufficient for removing duplicates in the set
				store := sdkCtx.KVStore(getKey(stakingtypes.StoreKey))

				deleted := false

				iterator := storetypes.KVStorePrefixIterator(store, stakingtypes.ValidatorsByPowerIndexKey)
				defer iterator.Close()

				for ; iterator.Valid(); iterator.Next() {
					valAddr := stakingtypes.ParseValidatorPowerRankKey(iterator.Key())

					bz, err := keepers.StakingKeeper.ValidatorAddressCodec().StringToBytes(validator.GetOperator())
					if err != nil {
						return nil, err
					}

					// Delete all entries of a validator including duplicates
					// Duplicates seem to exists due to PoA interacting with the new power reduction
					if bytes.Equal(valAddr, bz) {
						if deleted {
							sdkCtx.Logger().Info("deleting duplicate validator")
						} else {
							deleted = true
							sdkCtx.Logger().Info("deleting validator first record")
						}

						store.Delete(iterator.Key())
						sdkCtx.Logger().Info("deleted the key")
					}
				}

				// Set validator again after deleting, recreating the validator with the correct power reduction
				keepers.StakingKeeper.SetValidatorByPowerIndex(ctx, validator)
				sdkCtx.Logger().Info("reset validator")
				_, err := keepers.StakingKeeper.ApplyAndReturnValidatorSetUpdates(ctx)
				sdkCtx.Logger().Info("update valset")
				if err != nil {
					return nil, err
				}

				sdkCtx.Logger().Info("done with validator")
			}
		}

		if sdkCtx.ChainID() == UNION_TESTNET {
			// Burn old tokens
			burnToken(ctx, *keepers, "muno") // union-minimal-devnet-1 (local devnet) and union-testnet-10 gas token
			burnToken(ctx, *keepers, "upoa") // union-1 and union-testnet-10 PoA token
			burnToken(ctx, *keepers, "ugas") // union-1 gas token
		}

		// Update max `commission` values
		valHexes := valsToUpdateByHex[sdkCtx.ChainID()]
		for _, valHex := range valHexes {
			valAddr, err := sdk.ValAddressFromHex(valHex)
			if err != nil {
				sdkCtx.Logger().Error(
					"Unable to get validator from hex",
					"valHex", valHex,
				)
				return nil, err
			}
			val, err := keepers.StakingKeeper.GetValidator(ctx, valAddr)
			if err != nil {
				sdkCtx.Logger().Error(
					"Unable to get validator from valAddr",
					"valAddr", valAddr,
				)
				return nil, err
			}
			val.Commission.MaxRate = sdkmath.LegacyMustNewDecFromStr("1")
			err = keepers.StakingKeeper.SetValidator(ctx, val)
			if err != nil {
				sdkCtx.Logger().Error(
					"Unable to update validator commission",
					"valAddr", valAddr,
				)
				return nil, err
			}
		}

		return migrations, nil
	}
}

func burnToken(ctx context.Context, keepers upgrades.AppKeepers, denom string) error {
	var tokenOwners *banktypes.QueryDenomOwnersResponse
	var err error

	tokenOwners, err = keepers.BankKeeper.DenomOwners(ctx, &banktypes.QueryDenomOwnersRequest{
		Denom:      denom,
		Pagination: nil,
	})
	tokenOwners, err = keepers.BankKeeper.DenomOwners(ctx, &banktypes.QueryDenomOwnersRequest{
		Denom: denom,
		Pagination: &query.PageRequest{
			Key:        []byte{},
			Offset:     0,
			Limit:      math.MaxUint64,
			CountTotal: false,
			Reverse:    false,
		},
	})
	if err != nil {
		return err
	}

	tokenSum := sdkmath.ZeroInt()
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
	if tokenSum.GT(sdkmath.ZeroInt()) {
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
