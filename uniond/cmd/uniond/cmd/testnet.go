package cmd

import (
	"fmt"
	"io"
	"strings"

	"cosmossdk.io/log"
	"cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/libs/bytes"
	tmos "github.com/cometbft/cometbft/libs/os"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	dbm "github.com/cosmos/cosmos-db"
	"github.com/cosmos/cosmos-sdk/client/flags"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	"github.com/cosmos/cosmos-sdk/server"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	distrtypes "github.com/cosmos/cosmos-sdk/x/distribution/types"
	minttypes "github.com/cosmos/cosmos-sdk/x/mint/types"
	slashingtypes "github.com/cosmos/cosmos-sdk/x/slashing/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
	"github.com/spf13/cast"
	"github.com/spf13/cobra"

	"github.com/unionlabs/union/uniond/app"
)

const (
	valVotingPower int64 = 900000000000000
)

var (
	flagAccountsToFund = "accounts-to-fund"
)

type valArgs struct {
	newValAddr         bytes.HexBytes
	newOperatorAddress string
	newValPubKey       crypto.PubKey
	accountsToFund     []sdk.AccAddress
	upgradeToTrigger   string
	homeDir            string
}

func NewInPlaceTestnetCmd(addStartFlags servertypes.ModuleInitFlags) *cobra.Command {
	cmd := server.InPlaceTestnetCreator(newTestnetApp)
	addStartFlags(cmd)
	cmd.Short = "Updates chain's application and consensus state with provided validator info and starts the node"
	cmd.Long = `The test command modifies both application and consensus stores within a local mainnet node and starts the node,
with the aim of facilitating testing procedures. This command replaces existing validator data with updated information,
thereby removing the old validator set and introducing a new set suitable for local testing purposes. By altering the state extracted from the mainnet node,
it enables developers to configure their local environments to reflect mainnet conditions more accurately.`

	cmd.Example = fmt.Sprintf(`%sd in-place-testnet testing-1 cosmosvaloper1w7f3xx7e75p4l7qdym5msqem9rd4dyc4mq79dm --home $HOME/.%sd/validator1 --validator-privkey=6dq+/KHNvyiw2TToCgOpUpQKIzrLs69Rb8Az39xvmxPHNoPxY1Cil8FY+4DhT9YwD6s0tFABMlLcpaylzKKBOg== --accounts-to-fund="cosmos1f7twgcq4ypzg7y24wuywy06xmdet8pc4473tnq,cosmos1qvuhm5m644660nd8377d6l7yz9e9hhm9evmx3x"`, "github.com/unionlabs/union/uniond", "github.com/unionlabs/union/uniond")

	cmd.Flags().String(flagAccountsToFund, "", "Comma-separated list of account addresses that will be funded for testing purposes")
	return cmd
}

// newTestnetApp starts by running the normal newApp method. From there, the app interface returned is modified in order
// for a testnet to be created from the provided app.
func newTestnetApp(logger log.Logger, db dbm.DB, traceStore io.Writer, appOpts servertypes.AppOptions) servertypes.Application {
	// Create an app and type cast to an App
	newApp := newApp(logger, db, traceStore, appOpts)
	testApp, ok := newApp.(*app.App)
	if !ok {
		panic("app created from newApp is not of type App")
	}

	// Get command args
	args, err := getCommandArgs(appOpts)
	if err != nil {
		panic(err)
	}

	return initAppForTestnet(testApp, args)
}

func initAppForTestnet(app *app.App, args valArgs) *app.App {
	// Required Changes:
	//
	ctx := app.BaseApp.NewUncachedContext(true, tmproto.Header{})

	pubkey := &ed25519.PubKey{Key: args.newValPubKey.Bytes()}
	pubkeyAny, err := codectypes.NewAnyWithValue(pubkey)
	if err != nil {
		tmos.Exit(err.Error())
	}

	// STAKING
	//

	// Create Validator struct for our new validator.
	newVal := stakingtypes.Validator{
		OperatorAddress: args.newOperatorAddress,
		ConsensusPubkey: pubkeyAny,
		Jailed:          false,
		Status:          stakingtypes.Bonded,
		Tokens:          math.NewInt(valVotingPower),
		DelegatorShares: math.LegacyMustNewDecFromStr("10000000"),
		Description: stakingtypes.Description{
			Moniker: "Testnet Validator",
		},
		Commission: stakingtypes.Commission{
			CommissionRates: stakingtypes.CommissionRates{
				Rate:          math.LegacyMustNewDecFromStr("0.05"),
				MaxRate:       math.LegacyMustNewDecFromStr("0.1"),
				MaxChangeRate: math.LegacyMustNewDecFromStr("0.05"),
			},
		},
		MinSelfDelegation: math.OneInt(),
	}

	validator, err := app.StakingKeeper.ValidatorAddressCodec().StringToBytes(newVal.GetOperator())
	if err != nil {
		tmos.Exit(err.Error())
	}

	// Remove all validators from power store
	stakingKey := app.GetKey(stakingtypes.ModuleName)
	stakingStore := ctx.KVStore(stakingKey)
	iterator, err := app.StakingKeeper.ValidatorsPowerStoreIterator(ctx)
	if err != nil {
		tmos.Exit(err.Error())
	}
	for ; iterator.Valid(); iterator.Next() {
		stakingStore.Delete(iterator.Key())
	}
	iterator.Close()

	// Remove all valdiators from last validators store
	iterator, err = app.StakingKeeper.LastValidatorsIterator(ctx)
	if err != nil {
		tmos.Exit(err.Error())
	}
	for ; iterator.Valid(); iterator.Next() {
		stakingStore.Delete(iterator.Key())
	}
	iterator.Close()

	// Remove all validators from validators store
	iterator = stakingStore.Iterator(stakingtypes.ValidatorsKey, storetypes.PrefixEndBytes(stakingtypes.ValidatorsKey))
	for ; iterator.Valid(); iterator.Next() {
		stakingStore.Delete(iterator.Key())
	}
	iterator.Close()

	// Remove all validators from unbonding queue
	iterator = stakingStore.Iterator(stakingtypes.ValidatorQueueKey, storetypes.PrefixEndBytes(stakingtypes.ValidatorQueueKey))
	for ; iterator.Valid(); iterator.Next() {
		stakingStore.Delete(iterator.Key())
	}
	iterator.Close()

	// Add our validator to power and last validators store
	app.StakingKeeper.SetValidator(ctx, newVal)
	err = app.StakingKeeper.SetValidatorByConsAddr(ctx, newVal)
	if err != nil {
		tmos.Exit(err.Error())
	}
	app.StakingKeeper.SetValidatorByPowerIndex(ctx, newVal)
	app.StakingKeeper.SetLastValidatorPower(ctx, validator, 0)
	if err := app.StakingKeeper.Hooks().AfterValidatorCreated(ctx, validator); err != nil {
		tmos.Exit(err.Error())
	}

	// DISTRIBUTION
	//

	// Initialize records for this validator across all distribution stores
	app.DistrKeeper.SetValidatorHistoricalRewards(ctx, validator, 0, distrtypes.NewValidatorHistoricalRewards(sdk.DecCoins{}, 1))
	app.DistrKeeper.SetValidatorCurrentRewards(ctx, validator, distrtypes.NewValidatorCurrentRewards(sdk.DecCoins{}, 1))
	app.DistrKeeper.SetValidatorAccumulatedCommission(ctx, validator, distrtypes.InitialValidatorAccumulatedCommission())
	app.DistrKeeper.SetValidatorOutstandingRewards(ctx, validator, distrtypes.ValidatorOutstandingRewards{Rewards: sdk.DecCoins{}})

	// SLASHING
	//

	// Set validator signing info for our new validator.
	newConsAddr := sdk.ConsAddress(args.newValAddr.Bytes())
	newValidatorSigningInfo := slashingtypes.ValidatorSigningInfo{
		Address:     newConsAddr.String(),
		StartHeight: app.LastBlockHeight() - 1,
		Tombstoned:  false,
	}
	app.SlashingKeeper.SetValidatorSigningInfo(ctx, newConsAddr, newValidatorSigningInfo)

	// BANK
	//
	bondDenom, err := app.StakingKeeper.BondDenom(ctx)
	if err != nil {
		tmos.Exit(err.Error())
	}

	defaultCoins := sdk.NewCoins(sdk.NewInt64Coin(bondDenom, 1000000000))

	// Fund local accounts
	for _, account := range args.accountsToFund {
		err := app.BankKeeper.MintCoins(ctx, minttypes.ModuleName, defaultCoins)
		if err != nil {
			tmos.Exit(err.Error())
		}
		err = app.BankKeeper.SendCoinsFromModuleToAccount(ctx, minttypes.ModuleName, account, defaultCoins)
		if err != nil {
			tmos.Exit(err.Error())
		}
	}

	return app
}

// parse the input flags and returns valArgs
func getCommandArgs(appOpts servertypes.AppOptions) (valArgs, error) {
	args := valArgs{}

	newValAddr, ok := appOpts.Get(server.KeyNewValAddr).(bytes.HexBytes)
	if !ok {
		panic("newValAddr is not of type bytes.HexBytes")
	}
	args.newValAddr = newValAddr
	newValPubKey, ok := appOpts.Get(server.KeyUserPubKey).(crypto.PubKey)
	if !ok {
		panic("newValPubKey is not of type crypto.PubKey")
	}
	args.newValPubKey = newValPubKey
	newOperatorAddress, ok := appOpts.Get(server.KeyNewOpAddr).(string)
	if !ok {
		panic("newOperatorAddress is not of type string")
	}
	args.newOperatorAddress = newOperatorAddress
	upgradeToTrigger, ok := appOpts.Get(server.KeyTriggerTestnetUpgrade).(string)
	if !ok {
		panic("upgradeToTrigger is not of type string")
	}
	args.upgradeToTrigger = upgradeToTrigger

	// validate  and set accounts to fund
	accountsString := cast.ToString(appOpts.Get(flagAccountsToFund))

	for _, account := range strings.Split(accountsString, ",") {
		if account != "" {
			addr, err := sdk.AccAddressFromBech32(account)
			if err != nil {
				return args, fmt.Errorf("invalid bech32 address format %w", err)
			}
			args.accountsToFund = append(args.accountsToFund, addr)
		}
	}

	// home dir
	homeDir := cast.ToString(appOpts.Get(flags.FlagHome))
	if homeDir == "" {
		return args, fmt.Errorf("invalid home dir")
	}
	args.homeDir = homeDir

	return args, nil
}
