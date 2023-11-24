package genutil

import (
	"encoding/json"
	"errors"
	"fmt"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	authvesting "github.com/cosmos/cosmos-sdk/x/auth/vesting/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
)

// AddGenesisAccount adds a genesis account to the genesis state.
// Where `cdc` is client codec, `genesisFileUrl` is the path/url of current genesis file,
// `accAddr` is the address to be added to the genesis state, `amountStr` is the list of initial coins
// to be added for the account, `appendAcct` updates the account if already exists.
// `vestingStart, vestingEnd and vestingAmtStr` respectively are the schedule start time, end time (unix epoch)
// `moduleName“ is the module name for which the account is being created
// and coins to be appended to the account already in the genesis.json file.
func AddGenesisAccount(
	cdc codec.Codec,
	accAddr sdk.AccAddress,
	appendAcct bool,
	genesisFileURL, amountStr, vestingAmtStr string,
	vestingStart, vestingEnd int64,
	moduleName string,
) error {
	coins, err := sdk.ParseCoinsNormalized(amountStr)
	if err != nil {
		return fmt.Errorf("failed to parse coins: %w", err)
	}

	vestingAmt, err := sdk.ParseCoinsNormalized(vestingAmtStr)
	if err != nil {
		return fmt.Errorf("failed to parse vesting amount: %w", err)
	}

	// create concrete account type based on input parameters
	var genAccount authtypes.GenesisAccount

	balances := banktypes.Balance{Address: accAddr.String(), Coins: coins.Sort()}
	baseAccount := authtypes.NewBaseAccount(accAddr, nil, 0, 0)

	if !vestingAmt.IsZero() {
		baseVestingAccount, err := authvesting.NewBaseVestingAccount(baseAccount, vestingAmt.Sort(), vestingEnd)
		if err != nil {
			return fmt.Errorf("failed to create base vesting account: %w", err)
		}

		if (balances.Coins.IsZero() && !baseVestingAccount.OriginalVesting.IsZero()) ||
			baseVestingAccount.OriginalVesting.IsAnyGT(balances.Coins) {
			return errors.New("vesting amount cannot be greater than total amount")
		}

		switch {
		case vestingStart != 0 && vestingEnd != 0:
			genAccount = authvesting.NewContinuousVestingAccountRaw(baseVestingAccount, vestingStart)

		case vestingEnd != 0:
			genAccount = authvesting.NewDelayedVestingAccountRaw(baseVestingAccount)

		default:
			return errors.New("invalid vesting parameters; must supply start and end time or end time")
		}
	} else if moduleName != "" {
		genAccount = authtypes.NewEmptyModuleAccount(moduleName, authtypes.Burner, authtypes.Minter)
	} else {
		genAccount = baseAccount
	}

	if err := genAccount.Validate(); err != nil {
		return fmt.Errorf("failed to validate new genesis account: %w", err)
	}

	appState, appGenesis, err := genutiltypes.GenesisStateFromGenFile(genesisFileURL)
	if err != nil {
		return fmt.Errorf("failed to unmarshal genesis state: %w", err)
	}

	authGenState := authtypes.GetGenesisStateFromAppState(cdc, appState)

	accs, err := authtypes.UnpackAccounts(authGenState.Accounts)
	if err != nil {
		return fmt.Errorf("failed to get accounts from any: %w", err)
	}

	bankGenState := banktypes.GetGenesisStateFromAppState(cdc, appState)
	if accs.Contains(accAddr) {
		if !appendAcct {
			return fmt.Errorf(" Account %s already exists\nUse `append` flag to append account at existing address", accAddr)
		}

		genesisB := banktypes.GetGenesisStateFromAppState(cdc, appState)
		for idx, acc := range genesisB.Balances {
			if acc.Address != accAddr.String() {
				continue
			}

			updatedCoins := acc.Coins.Add(coins...)
			bankGenState.Balances[idx] = banktypes.Balance{Address: accAddr.String(), Coins: updatedCoins.Sort()}
			break
		}
	} else {
		// Add the new account to the set of genesis accounts and sanitize the accounts afterwards.
		accs = append(accs, genAccount)
		accs = authtypes.SanitizeGenesisAccounts(accs)

		genAccs, err := authtypes.PackAccounts(accs)
		if err != nil {
			return fmt.Errorf("failed to convert accounts into any's: %w", err)
		}
		authGenState.Accounts = genAccs

		authGenStateBz, err := cdc.MarshalJSON(&authGenState)
		if err != nil {
			return fmt.Errorf("failed to marshal auth genesis state: %w", err)
		}
		appState[authtypes.ModuleName] = authGenStateBz

		bankGenState.Balances = append(bankGenState.Balances, balances)
	}

	bankGenState.Balances = banktypes.SanitizeGenesisBalances(bankGenState.Balances)

	bankGenState.Supply = bankGenState.Supply.Add(balances.Coins...)

	bankGenStateBz, err := cdc.MarshalJSON(bankGenState)
	if err != nil {
		return fmt.Errorf("failed to marshal bank genesis state: %w", err)
	}
	appState[banktypes.ModuleName] = bankGenStateBz

	appStateJSON, err := json.Marshal(appState)
	if err != nil {
		return fmt.Errorf("failed to marshal application genesis state: %w", err)
	}

	appGenesis.AppState = appStateJSON
	return ExportGenesisFile(appGenesis, genesisFileURL)
}
