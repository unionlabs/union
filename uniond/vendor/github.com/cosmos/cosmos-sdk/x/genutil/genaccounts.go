package genutil

import (
	"encoding/json"
	"errors"
	"fmt"

	"cosmossdk.io/core/address"
	banktypes "cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	authvesting "github.com/cosmos/cosmos-sdk/x/auth/vesting/types"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
)

type GenesisAccount struct {
	// Base
	Address string    `json:"address"`
	Coins   sdk.Coins `json:"coins"`

	// Vesting
	VestingAmt   sdk.Coins `json:"vesting_amt,omitempty"`
	VestingStart int64     `json:"vesting_start,omitempty"`
	VestingEnd   int64     `json:"vesting_end,omitempty"`

	// Module
	ModuleName string `json:"module_name,omitempty"`
}

// AddGenesisAccounts adds genesis accounts to the genesis state.
// Where `cdc` is the client codec, `addressCodec` is the address codec, `accounts` are the genesis accounts to add,
// `appendAcct` updates the account if already exists, and `genesisFileURL` is the path/url of the current genesis file.
func AddGenesisAccounts(
	cdc codec.Codec,
	addressCodec address.Codec,
	accounts []GenesisAccount,
	appendAcct bool,
	genesisFileURL string,
) error {
	appState, appGenesis, err := genutiltypes.GenesisStateFromGenFile(genesisFileURL)
	if err != nil {
		return fmt.Errorf("failed to unmarshal genesis state: %w", err)
	}

	authGenState := authtypes.GetGenesisStateFromAppState(cdc, appState)
	bankGenState := banktypes.GetGenesisStateFromAppState(cdc, appState)

	accs, err := authtypes.UnpackAccounts(authGenState.Accounts)
	if err != nil {
		return fmt.Errorf("failed to get accounts from any: %w", err)
	}

	newSupplyCoinsCache := sdk.NewCoins()
	balanceCache := make(map[string]banktypes.Balance)
	for _, acc := range accs {
		for _, balance := range bankGenState.GetBalances() {
			if balance.Address == acc.GetAddress().String() {
				balanceCache[acc.GetAddress().String()] = balance
			}
		}
	}

	for _, acc := range accounts {
		addr := acc.Address
		coins := acc.Coins

		accAddr, err := addressCodec.StringToBytes(addr)
		if err != nil {
			return fmt.Errorf("failed to parse account address %s: %w", addr, err)
		}

		// create concrete account type based on input parameters
		var genAccount authtypes.GenesisAccount

		balances := banktypes.Balance{Address: addr, Coins: coins.Sort()}
		baseAccount := authtypes.NewBaseAccount(accAddr, nil, 0, 0)

		vestingAmt := acc.VestingAmt
		if !vestingAmt.IsZero() {
			vestingStart := acc.VestingStart
			vestingEnd := acc.VestingEnd

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
		} else if acc.ModuleName != "" {
			genAccount = authtypes.NewEmptyModuleAccount(acc.ModuleName, authtypes.Burner, authtypes.Minter)
		} else {
			genAccount = baseAccount
		}

		if err := genAccount.Validate(); err != nil {
			return fmt.Errorf("failed to validate new genesis account: %w", err)
		}

		if _, ok := balanceCache[addr]; ok {
			if !appendAcct {
				return fmt.Errorf(" Account %s already exists\nUse `append` flag to append account at existing address", accAddr)
			}

			for idx, acc := range bankGenState.Balances {
				if acc.Address != addr {
					continue
				}

				updatedCoins := acc.Coins.Add(coins...)
				bankGenState.Balances[idx] = banktypes.Balance{Address: addr, Coins: updatedCoins.Sort()}
				break
			}
		} else {
			accs = append(accs, genAccount)
			bankGenState.Balances = append(bankGenState.Balances, balances)
		}

		newSupplyCoinsCache = newSupplyCoinsCache.Add(coins...)
	}

	accs = authtypes.SanitizeGenesisAccounts(accs)

	authGenState.Accounts, err = authtypes.PackAccounts(accs)
	if err != nil {
		return fmt.Errorf("failed to convert accounts into any's: %w", err)
	}

	appState[authtypes.ModuleName], err = cdc.MarshalJSON(&authGenState)
	if err != nil {
		return fmt.Errorf("failed to marshal auth genesis state: %w", err)
	}

	bankGenState.Balances, err = banktypes.SanitizeGenesisBalances(bankGenState.Balances, addressCodec)
	if err != nil {
		return fmt.Errorf("failed to sanitize genesis bank Balances: %w", err)
	}

	bankGenState.Supply = bankGenState.Supply.Add(newSupplyCoinsCache...)

	appState[banktypes.ModuleName], err = cdc.MarshalJSON(bankGenState)
	if err != nil {
		return fmt.Errorf("failed to marshal bank genesis state: %w", err)
	}

	appStateJSON, err := json.Marshal(appState)
	if err != nil {
		return fmt.Errorf("failed to marshal application genesis state: %w", err)
	}

	appGenesis.AppState = appStateJSON
	return ExportGenesisFile(appGenesis, genesisFileURL)
}
