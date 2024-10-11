---
sidebar_position: 1
---

# Application Testnets

Building an application is complicated and requires a lot of testing. The Cosmos SDK provides a way to test your application in a real-world environment: a testnet. 

We allow developers to take the state from their mainnet and run tests against the state. This is useful for testing upgrade migrations, or for testing the application in a real-world environment.

## Testnet Setup

We will be breaking down the steps to create a testnet from mainnet state. 

```go 
  // InitSimAppForTestnet is broken down into two sections:
  // Required Changes: Changes that, if not made, will cause the testnet to halt or panic
  // Optional Changes: Changes to customize the testnet to one's liking (lower vote times, fund accounts, etc)
  func InitSimAppForTestnet(app *SimApp, newValAddr bytes.HexBytes, newValPubKey crypto.PubKey, newOperatorAddress, upgradeToTrigger string) *SimApp {
  ...
  }
```

### Required Changes

#### Staking

When creating a testnet the important part is migrate the validator set from many validators to one or a few. This allows developers to spin up the chain without needing to replace validator keys. 

```go
	ctx := app.BaseApp.NewUncachedContext(true, tmproto.Header{})
	pubkey := &ed25519.PubKey{Key: newValPubKey.Bytes()}
	pubkeyAny, err := types.NewAnyWithValue(pubkey)
	if err != nil {
		tmos.Exit(err.Error())
	}

	// STAKING
	//

	// Create Validator struct for our new validator.
	_, bz, err := bech32.DecodeAndConvert(newOperatorAddress)
	if err != nil {
		tmos.Exit(err.Error())
	}
	bech32Addr, err := bech32.ConvertAndEncode("simvaloper", bz)
	if err != nil {
		tmos.Exit(err.Error())
	}
	newVal := stakingtypes.Validator{
		OperatorAddress: bech32Addr,
		ConsensusPubkey: pubkeyAny,
		Jailed:          false,
		Status:          stakingtypes.Bonded,
		Tokens:          sdk.NewInt(900000000000000),
		DelegatorShares: sdk.MustNewDecFromStr("10000000"),
		Description: stakingtypes.Description{
			Moniker: "Testnet Validator",
		},
		Commission: stakingtypes.Commission{
			CommissionRates: stakingtypes.CommissionRates{
				Rate:          sdk.MustNewDecFromStr("0.05"),
				MaxRate:       sdk.MustNewDecFromStr("0.1"),
				MaxChangeRate: sdk.MustNewDecFromStr("0.05"),
			},
		},
		MinSelfDelegation: sdk.OneInt(),
	}

	// Remove all validators from power store
	stakingKey := app.GetKey(stakingtypes.ModuleName)
	stakingStore := ctx.KVStore(stakingKey)
	iterator := app.StakingKeeper.ValidatorsPowerStoreIterator(ctx)
	for ; iterator.Valid(); iterator.Next() {
		stakingStore.Delete(iterator.Key())
	}
	iterator.Close()

	// Remove all validators from last validators store
	iterator = app.StakingKeeper.LastValidatorsIterator(ctx)
	for ; iterator.Valid(); iterator.Next() {
		app.StakingKeeper.LastValidatorPower.Delete(iterator.Key())
	}
	iterator.Close()

	// Add our validator to power and last validators store
	app.StakingKeeper.SetValidator(ctx, newVal)
	err = app.StakingKeeper.SetValidatorByConsAddr(ctx, newVal)
	if err != nil {
		panic(err)
	}
	app.StakingKeeper.SetValidatorByPowerIndex(ctx, newVal)
	app.StakingKeeper.SetLastValidatorPower(ctx, newVal.GetOperator(), 0)
	if err := app.StakingKeeper.Hooks().AfterValidatorCreated(ctx, newVal.GetOperator()); err != nil {
		panic(err)
	}
```

#### Distribution

Since the validator set has changed, we need to update the distribution records for the new validator.


```go
	// Initialize records for this validator across all distribution stores
	app.DistrKeeper.ValidatorHistoricalRewards.Set(ctx, newVal.GetOperator(), 0, distrtypes.NewValidatorHistoricalRewards(sdk.DecCoins{}, 1))
	app.DistrKeeper.ValidatorCurrentRewards.Set(ctx, newVal.GetOperator(), distrtypes.NewValidatorCurrentRewards(sdk.DecCoins{}, 1))
	app.DistrKeeper.ValidatorAccumulatedCommission.Set(ctx, newVal.GetOperator(), distrtypes.InitialValidatorAccumulatedCommission())
	app.DistrKeeper.ValidatorOutstandingRewards.Set(ctx, newVal.GetOperator(), distrtypes.ValidatorOutstandingRewards{Rewards: sdk.DecCoins{}})
```

#### Slashing

We also need to set the validator signing info for the new validator.

```go
  // SLASHING
	//

	// Set validator signing info for our new validator.
	newConsAddr := sdk.ConsAddress(newValAddr.Bytes())
	newValidatorSigningInfo := slashingtypes.ValidatorSigningInfo{
		Address:     newConsAddr.String(),
		StartHeight: app.LastBlockHeight() - 1,
		Tombstoned:  false,
	}
	app.SlashingKeeper.ValidatorSigningInfo.Set(ctx, newConsAddr, newValidatorSigningInfo)
```

#### Bank

It is useful to create new accounts for your testing purposes. This avoids the need to have the same key as you may have on mainnet. 

```go
  // BANK
	//

	defaultCoins := sdk.NewCoins(sdk.NewInt64Coin("ustake", 1000000000000))

	localSimAppAccounts := []sdk.AccAddress{
		sdk.MustAccAddressFromBech32("cosmos12smx2wdlyttvyzvzg54y2vnqwq2qjateuf7thj"),
		sdk.MustAccAddressFromBech32("cosmos1cyyzpxplxdzkeea7kwsydadg87357qnahakaks"),
		sdk.MustAccAddressFromBech32("cosmos18s5lynnmx37hq4wlrw9gdn68sg2uxp5rgk26vv"),
		sdk.MustAccAddressFromBech32("cosmos1qwexv7c6sm95lwhzn9027vyu2ccneaqad4w8ka"),
		sdk.MustAccAddressFromBech32("cosmos14hcxlnwlqtq75ttaxf674vk6mafspg8xwgnn53"),
		sdk.MustAccAddressFromBech32("cosmos12rr534cer5c0vj53eq4y32lcwguyy7nndt0u2t"),
		sdk.MustAccAddressFromBech32("cosmos1nt33cjd5auzh36syym6azgc8tve0jlvklnq7jq"),
		sdk.MustAccAddressFromBech32("cosmos10qfrpash5g2vk3hppvu45x0g860czur8ff5yx0"),
		sdk.MustAccAddressFromBech32("cosmos1f4tvsdukfwh6s9swrc24gkuz23tp8pd3e9r5fa"),
		sdk.MustAccAddressFromBech32("cosmos1myv43sqgnj5sm4zl98ftl45af9cfzk7nhjxjqh"),
		sdk.MustAccAddressFromBech32("cosmos14gs9zqh8m49yy9kscjqu9h72exyf295afg6kgk"),
		sdk.MustAccAddressFromBech32("cosmos1jllfytsz4dryxhz5tl7u73v29exsf80vz52ucc")}

  // Fund localSimApp accounts
	for _, account := range localSimAppAccounts {
		err := app.BankKeeper.MintCoins(ctx, minttypes.ModuleName, defaultCoins)
		if err != nil {
			tmos.Exit(err.Error())
		}
		err = app.BankKeeper.SendCoinsFromModuleToAccount(ctx, minttypes.ModuleName, account, defaultCoins)
		if err != nil {
			tmos.Exit(err.Error())
		}
	}
```

#### Upgrade

If you would like to schedule an upgrade the below can be used. 

```go
	// UPGRADE
	//

	if upgradeToTrigger != "" {
		upgradePlan := upgradetypes.Plan{
			Name:   upgradeToTrigger,
			Height: app.LastBlockHeight(),
		}
		err = app.UpgradeKeeper.ScheduleUpgrade(ctx, upgradePlan)
		if err != nil {
			panic(err)
		}
	}
```

### Optional Changes

If you have custom modules that rely on specific state from the above modules and/or you would like to test your custom module, you will need to update the state of your custom module to reflect your needs

## Running the Testnet

Before we can run the testnet we must plug everything together. 

in `root.go`, in the `initRootCmd` function we add:

```diff
server.AddCommands(rootCmd, simapp.DefaultNodeHome, newApp, createSimAppAndExport)
+server.AddTestnetCreatorCommand(rootCmd, simapp.DefaultNodeHome, newTestnetApp)
```

Next we will add a newTestnetApp helper function:

```diff
// newTestnetApp starts by running the normal newApp method. From there, the app interface returned is modified in order
// for a testnet to be created from the provided app.
func newTestnetApp(logger log.Logger, db cometbftdb.DB, traceStore io.Writer, appOpts servertypes.AppOptions) servertypes.Application {
	// Create an app and type cast to an SimApp
	app := newApp(logger, db, traceStore, appOpts)
	simApp, ok := app.(*simapp.SimApp)
	if !ok {
		panic("app created from newApp is not of type simApp")
	}

	newValAddr, ok := appOpts.Get(server.KeyNewValAddr).(bytes.HexBytes)
	if !ok {
		panic("newValAddr is not of type bytes.HexBytes")
	}
	newValPubKey, ok := appOpts.Get(server.KeyUserPubKey).(crypto.PubKey)
	if !ok {
		panic("newValPubKey is not of type crypto.PubKey")
	}
	newOperatorAddress, ok := appOpts.Get(server.KeyNewOpAddr).(string)
	if !ok {
		panic("newOperatorAddress is not of type string")
	}
	upgradeToTrigger, ok := appOpts.Get(server.KeyTriggerTestnetUpgrade).(string)
	if !ok {
		panic("upgradeToTrigger is not of type string")
	}

	// Make modifications to the normal SimApp required to run the network locally
	return simapp.InitSimAppForTestnet(simApp, newValAddr, newValPubKey, newOperatorAddress, upgradeToTrigger)
}
```
