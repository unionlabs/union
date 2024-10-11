package types

import (
	"encoding/json"
	"fmt"
	"os"

	stakingtypes "cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// NewGenesisState creates a new GenesisState object
func NewGenesisState(genTxs []json.RawMessage) *GenesisState {
	// Ensure genTxs is never nil, https://github.com/cosmos/cosmos-sdk/issues/5086
	if len(genTxs) == 0 {
		genTxs = make([]json.RawMessage, 0)
	}
	return &GenesisState{
		GenTxs: genTxs,
	}
}

// DefaultGenesisState returns the genutil module's default genesis state.
func DefaultGenesisState() *GenesisState {
	return &GenesisState{
		GenTxs: []json.RawMessage{},
	}
}

// NewGenesisStateFromTx creates a new GenesisState object
// from auth transactions
func NewGenesisStateFromTx(txJSONEncoder sdk.TxEncoder, genTxs []sdk.Tx) *GenesisState {
	genTxsBz := make([]json.RawMessage, len(genTxs))
	for i, genTx := range genTxs {
		var err error
		genTxsBz[i], err = txJSONEncoder(genTx)
		if err != nil {
			panic(err)
		}
	}
	return NewGenesisState(genTxsBz)
}

// GetGenesisStateFromAppState gets the genutil genesis state from the expected app state
func GetGenesisStateFromAppState(cdc codec.JSONCodec, appState map[string]json.RawMessage) *GenesisState {
	var genesisState GenesisState
	if appState[ModuleName] != nil {
		cdc.MustUnmarshalJSON(appState[ModuleName], &genesisState)
	}
	return &genesisState
}

// SetGenesisStateInAppState sets the genutil genesis state within the expected app state
func SetGenesisStateInAppState(
	cdc codec.JSONCodec, appState map[string]json.RawMessage, genesisState *GenesisState,
) map[string]json.RawMessage {
	genesisStateBz := cdc.MustMarshalJSON(genesisState)
	appState[ModuleName] = genesisStateBz
	return appState
}

// GenesisStateFromAppGenesis creates the core parameters for genesis initialization
// for the application.
//
// NOTE: The pubkey input is this machines pubkey.
func GenesisStateFromAppGenesis(genesis *AppGenesis) (genesisState map[string]json.RawMessage, err error) {
	if err = json.Unmarshal(genesis.AppState, &genesisState); err != nil {
		return genesisState, err
	}
	return genesisState, nil
}

// GenesisStateFromGenFile creates the core parameters for genesis initialization
// for the application.
//
// NOTE: The pubkey input is this machines pubkey.
func GenesisStateFromGenFile(genFile string) (genesisState map[string]json.RawMessage, genesis *AppGenesis, err error) {
	if _, err := os.Stat(genFile); os.IsNotExist(err) {
		return genesisState, genesis, fmt.Errorf("%s does not exist, run `init` first", genFile)
	}

	genesis, err = AppGenesisFromFile(genFile)
	if err != nil {
		return genesisState, genesis, err
	}

	genesisState, err = GenesisStateFromAppGenesis(genesis)
	return genesisState, genesis, err
}

// ValidateGenesis validates GenTx transactions
func ValidateGenesis(genesisState *GenesisState, txJSONDecoder sdk.TxDecoder, validator MessageValidator) error {
	for _, genTx := range genesisState.GenTxs {
		_, err := ValidateAndGetGenTx(genTx, txJSONDecoder, validator)
		if err != nil {
			return err
		}
	}
	return nil
}

type MessageValidator func([]sdk.Msg) error

func DefaultMessageValidator(msgs []sdk.Msg) error {
	if len(msgs) != 1 {
		return fmt.Errorf("unexpected number of GenTx messages; got: %d, expected: 1", len(msgs))
	}
	if _, ok := msgs[0].(*stakingtypes.MsgCreateValidator); !ok {
		return fmt.Errorf("unexpected GenTx message type; expected: MsgCreateValidator, got: %T", msgs[0])
	}

	if m, ok := msgs[0].(sdk.HasValidateBasic); ok {
		if err := m.ValidateBasic(); err != nil {
			return fmt.Errorf("invalid GenTx '%s': %w", msgs[0], err)
		}
	}

	return nil
}

// ValidateAndGetGenTx validates the genesis transaction and returns GenTx if valid
// it cannot verify the signature as it is stateless validation
func ValidateAndGetGenTx(genTx json.RawMessage, txJSONDecoder sdk.TxDecoder, validator MessageValidator) (sdk.Tx, error) {
	tx, err := txJSONDecoder(genTx)
	if err != nil {
		return tx, fmt.Errorf("failed to decode gentx: %s, error: %w", genTx, err)
	}

	return tx, validator(tx.GetMsgs())
}
