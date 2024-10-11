package sims

import (
	"bufio"
	"bytes"
	"encoding/json"
	"errors"
	"io"
	"math/rand"
	"os"
	"path/filepath"
	"time"

	"github.com/cosmos/gogoproto/proto"

	"cosmossdk.io/core/address"
	"cosmossdk.io/math"
	banktypes "cosmossdk.io/x/bank/types"
	stakingtypes "cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
	simcli "github.com/cosmos/cosmos-sdk/x/simulation/client/cli"
)

// Simulation parameter constants
const (
	StakePerAccount           = "stake_per_account"
	InitiallyBondedValidators = "initially_bonded_validators"
)

// AppStateFn returns the initial application state using a genesis or the simulation parameters.
func AppStateFn(
	cdc codec.JSONCodec,
	addressCodec, validatorCodec address.Codec,
	modules []module.AppModuleSimulation,
	genesisState map[string]json.RawMessage,
) simtypes.AppStateFn {
	return func(
		r *rand.Rand,
		accs []simtypes.Account,
		config simtypes.Config,
	) (appState json.RawMessage, simAccs []simtypes.Account, chainID string, genesisTimestamp time.Time) {
		genesisTimestamp = time.Unix(config.GenesisTime, 0)
		chainID = config.ChainID

		switch {
		case config.ParamsFile != "" && config.GenesisFile != "":
			panic("cannot provide both a genesis file and a params file")

		case config.GenesisFile != "":
			// override the default chain-id from simapp to set it later to the config
			genesisDoc, accounts, err := AppStateFromGenesisFileFn(r, cdc, config.GenesisFile)
			if err != nil {
				panic(err)
			}

			if simcli.FlagGenesisTimeValue == 0 {
				// use genesis timestamp if no custom timestamp is provided (i.e no random timestamp)
				genesisTimestamp = genesisDoc.GenesisTime
			}

			appState = genesisDoc.AppState
			chainID = genesisDoc.ChainID
			simAccs = accounts

		case config.ParamsFile != "":
			appParams := make(simtypes.AppParams)
			bz, err := os.ReadFile(config.ParamsFile)
			if err != nil {
				panic(err)
			}

			err = json.Unmarshal(bz, &appParams)
			if err != nil {
				panic(err)
			}
			appState, simAccs = AppStateRandomizedFn(modules, r, cdc, accs, genesisTimestamp, appParams, genesisState, addressCodec, validatorCodec)

		default:
			appParams := make(simtypes.AppParams)
			appState, simAccs = AppStateRandomizedFn(modules, r, cdc, accs, genesisTimestamp, appParams, genesisState, addressCodec, validatorCodec)
		}

		rawState := make(map[string]json.RawMessage)
		err := json.Unmarshal(appState, &rawState)
		if err != nil {
			panic(err)
		}

		stakingStateBz, ok := rawState[stakingtypes.ModuleName]
		if !ok {
			panic("staking genesis state is missing")
		}

		stakingState := new(stakingtypes.GenesisState)
		if err = cdc.UnmarshalJSON(stakingStateBz, stakingState); err != nil {
			panic(err)
		}
		// compute not bonded balance
		notBondedTokens := math.ZeroInt()
		for _, val := range stakingState.Validators {
			if val.Status != stakingtypes.Unbonded {
				continue
			}
			notBondedTokens = notBondedTokens.Add(val.GetTokens())
		}
		notBondedCoins := sdk.NewCoin(stakingState.Params.BondDenom, notBondedTokens)
		// edit bank state to make it have the not bonded pool tokens
		bankStateBz, ok := rawState[testutil.BankModuleName]
		if !ok {
			panic("bank genesis state is missing")
		}
		bankState := new(banktypes.GenesisState)
		if err = cdc.UnmarshalJSON(bankStateBz, bankState); err != nil {
			panic(err)
		}

		stakingAddr := authtypes.NewModuleAddress(stakingtypes.NotBondedPoolName).String()
		var found bool
		for _, balance := range bankState.Balances {
			if balance.Address == stakingAddr {
				found = true
				break
			}
		}
		if !found {
			bankState.Balances = append(bankState.Balances, banktypes.Balance{
				Address: stakingAddr,
				Coins:   sdk.NewCoins(notBondedCoins),
			})
		}

		// change appState back
		for name, state := range map[string]proto.Message{
			stakingtypes.ModuleName: stakingState,
			testutil.BankModuleName: bankState,
		} {
			rawState[name] = cdc.MustMarshalJSON(state)
		}

		// replace appstate
		appState, err = json.Marshal(rawState)
		if err != nil {
			panic(err)
		}
		return appState, simAccs, chainID, genesisTimestamp
	}
}

// AppStateRandomizedFn creates calls each module's GenesisState generator function
// and creates the simulation params
func AppStateRandomizedFn(
	modules []module.AppModuleSimulation,
	r *rand.Rand,
	cdc codec.JSONCodec,
	accs []simtypes.Account,
	genesisTimestamp time.Time,
	appParams simtypes.AppParams,
	genesisState map[string]json.RawMessage,
	addressCodec, validatorCodec address.Codec,
) (json.RawMessage, []simtypes.Account) {
	numAccs := int64(len(accs))
	// generate a random amount of initial stake coins and a random initial
	// number of bonded accounts
	var (
		numInitiallyBonded int64
		initialStake       math.Int
	)
	appParams.GetOrGenerate(
		StakePerAccount, &initialStake, r,
		func(r *rand.Rand) { initialStake = sdk.DefaultPowerReduction.AddRaw(r.Int63n(1e12)) },
	)
	appParams.GetOrGenerate(
		InitiallyBondedValidators, &numInitiallyBonded, r,
		func(r *rand.Rand) { numInitiallyBonded = int64(r.Intn(299) + 1) },
	)

	if numInitiallyBonded > numAccs {
		numInitiallyBonded = numAccs
	}

	simState := &module.SimulationState{
		AppParams:      appParams,
		Cdc:            cdc,
		AddressCodec:   addressCodec,
		ValidatorCodec: validatorCodec,
		Rand:           r,
		GenState:       genesisState,
		Accounts:       accs,
		InitialStake:   initialStake,
		NumBonded:      numInitiallyBonded,
		BondDenom:      sdk.DefaultBondDenom,
		GenTimestamp:   genesisTimestamp,
	}
	generateGenesisStates(modules, simState)

	appState, err := json.Marshal(genesisState)
	if err != nil {
		panic(err)
	}

	return appState, accs
}

// AppStateFromGenesisFileFn util function to generate the genesis AppState
// from a genesis.json file.
func AppStateFromGenesisFileFn(_ io.Reader, cdc codec.JSONCodec, genesisFile string) (genutiltypes.AppGenesis, []simtypes.Account, error) {
	file, err := os.Open(filepath.Clean(genesisFile))
	if err != nil {
		panic(err)
	}
	defer file.Close()

	genesis, err := genutiltypes.AppGenesisFromReader(bufio.NewReader(file))
	if err != nil {
		return genutiltypes.AppGenesis{}, nil, err
	}

	appStateJSON := genesis.AppState
	newAccs, err := AccountsFromAppState(cdc, appStateJSON)
	if err != nil {
		panic(err)
	}

	return *genesis, newAccs, nil
}

func AccountsFromAppState(cdc codec.JSONCodec, appStateJSON json.RawMessage) ([]simtypes.Account, error) {
	var appState map[string]json.RawMessage
	if err := json.Unmarshal(appStateJSON, &appState); err != nil {
		return nil, err
	}

	var authGenesis authtypes.GenesisState
	if appState[testutil.AuthModuleName] != nil {
		cdc.MustUnmarshalJSON(appState[testutil.AuthModuleName], &authGenesis)
	}
	r := bufio.NewReader(bytes.NewReader(appStateJSON)) // any deterministic source
	newAccs := make([]simtypes.Account, len(authGenesis.Accounts))
	for i, acc := range authGenesis.Accounts {
		// Pick a random private key, since we don't know the actual key
		// This should be fine as it's only used for mock CometBFT validators
		// and these keys are never actually used to sign by mock CometBFT.
		privkeySeed := make([]byte, 15)
		if _, err := r.Read(privkeySeed); err != nil {
			return nil, err
		}

		privKey := secp256k1.GenPrivKeyFromSecret(privkeySeed)

		a, ok := acc.GetCachedValue().(sdk.AccountI)
		if !ok {
			return nil, errors.New("expected account")
		}

		// create simulator accounts
		simAcc := simtypes.Account{PrivKey: privKey, PubKey: privKey.PubKey(), Address: a.GetAddress(), ConsKey: ed25519.GenPrivKeyFromSecret(privkeySeed)}
		newAccs[i] = simAcc
	}
	return newAccs, nil
}

func generateGenesisStates(modules []module.AppModuleSimulation, simState *module.SimulationState) {
	for _, m := range modules {
		m.GenerateGenesisState(simState)
	}
}
