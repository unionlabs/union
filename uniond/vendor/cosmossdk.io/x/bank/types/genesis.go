package types

import (
	"encoding/json"
	"errors"
	"fmt"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// Validate performs basic validation of supply genesis data returning an
// error for any failed validation criteria.
func (gs GenesisState) Validate() error {
	if len(gs.Params.SendEnabled) > 0 && len(gs.SendEnabled) > 0 {
		return errors.New("send_enabled defined in both the send_enabled field and in params (deprecated)")
	}

	if err := gs.Params.Validate(); err != nil {
		return err
	}

	seenSendEnabled := make(map[string]bool)
	seenBalances := make(map[string]bool)
	seenMetadatas := make(map[string]bool)

	totalSupply := sdk.Coins{}

	for _, p := range gs.GetAllSendEnabled() {
		if _, exists := seenSendEnabled[p.Denom]; exists {
			return fmt.Errorf("duplicate send enabled found: '%s'", p.Denom)
		}
		if err := p.Validate(); err != nil {
			return err
		}
		seenSendEnabled[p.Denom] = true
	}

	for _, balance := range gs.Balances {
		if seenBalances[balance.Address] {
			return fmt.Errorf("duplicate balance for address %s", balance.Address)
		}

		if err := balance.Validate(); err != nil {
			return err
		}

		seenBalances[balance.Address] = true

		totalSupply = totalSupply.Add(balance.Coins...)
	}

	for _, metadata := range gs.DenomMetadata {
		if seenMetadatas[metadata.Base] {
			return fmt.Errorf("duplicate client metadata for denom %s", metadata.Base)
		}

		if err := metadata.Validate(); err != nil {
			return err
		}

		seenMetadatas[metadata.Base] = true
	}

	if !gs.Supply.Empty() {
		// NOTE: this errors if supply for any given coin is zero
		err := gs.Supply.Validate()
		if err != nil {
			return err
		}

		if !gs.Supply.Equal(totalSupply) {
			return fmt.Errorf("genesis supply is incorrect, expected %v, got %v", gs.Supply, totalSupply)
		}
	}

	return nil
}

// NewGenesisState creates a new genesis state.
func NewGenesisState(params Params, balances []Balance, supply sdk.Coins, denomMetaData []Metadata, sendEnabled []SendEnabled) *GenesisState {
	rv := &GenesisState{
		Params:        params,
		Balances:      balances,
		Supply:        supply,
		DenomMetadata: denomMetaData,
		SendEnabled:   sendEnabled,
	}
	rv.MigrateSendEnabled()
	return rv
}

// DefaultGenesisState returns a default bank module genesis state.
func DefaultGenesisState() *GenesisState {
	return NewGenesisState(DefaultParams(), []Balance{}, sdk.Coins{}, []Metadata{}, []SendEnabled{})
}

// GetGenesisStateFromAppState returns x/bank GenesisState given raw application
// genesis state.
func GetGenesisStateFromAppState(cdc codec.JSONCodec, appState map[string]json.RawMessage) *GenesisState {
	var genesisState GenesisState

	if appState[ModuleName] != nil {
		cdc.MustUnmarshalJSON(appState[ModuleName], &genesisState)
	}

	return &genesisState
}

// MigrateSendEnabled moves the SendEnabled info from Params into the
// GenesisState.SendEnabled field and removes them from Params. If the
// Params.SendEnabled slice is empty, this is a noop.
//
// If the main SendEnabled slice already has entries, the Params.SendEnabled
// entries are added. In case of the same demon in both, preference is given to
// the existing (main GenesisState field) entry.
func (gs *GenesisState) MigrateSendEnabled() {
	gs.SendEnabled = gs.GetAllSendEnabled()
	gs.Params.SendEnabled = nil
}

// GetAllSendEnabled returns all the SendEnabled entries from both the SendEnabled
// field and the Params. If a denom has an entry in both, the entry in the
// SendEnabled field takes precedence over one in Params.
func (gs GenesisState) GetAllSendEnabled() []SendEnabled {
	if len(gs.Params.SendEnabled) == 0 {
		return gs.SendEnabled
	}

	rv := make([]SendEnabled, len(gs.SendEnabled))
	knownSendEnabled := map[string]bool{}
	for i, se := range gs.SendEnabled {
		rv[i] = se
		knownSendEnabled[se.Denom] = true
	}

	for _, se := range gs.Params.SendEnabled {
		if _, known := knownSendEnabled[se.Denom]; !known {
			rv = append(rv, *se)
		}
	}

	return rv
}
