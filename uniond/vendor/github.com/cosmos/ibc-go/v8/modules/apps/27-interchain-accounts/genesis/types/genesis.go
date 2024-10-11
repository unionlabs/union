package types

import (
	controllertypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	hosttypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
)

// DefaultGenesis creates and returns the interchain accounts GenesisState
func DefaultGenesis() *GenesisState {
	return &GenesisState{
		ControllerGenesisState: DefaultControllerGenesis(),
		HostGenesisState:       DefaultHostGenesis(),
	}
}

// NewGenesisState creates and returns a new GenesisState instance from the provided controller and host genesis state types
func NewGenesisState(controllerGenesisState ControllerGenesisState, hostGenesisState HostGenesisState) *GenesisState {
	return &GenesisState{
		ControllerGenesisState: controllerGenesisState,
		HostGenesisState:       hostGenesisState,
	}
}

// Validate performs basic validation of the interchain accounts GenesisState
func (gs GenesisState) Validate() error {
	if err := gs.ControllerGenesisState.Validate(); err != nil {
		return err
	}

	return gs.HostGenesisState.Validate()
}

// DefaultControllerGenesis creates and returns the default interchain accounts ControllerGenesisState
func DefaultControllerGenesis() ControllerGenesisState {
	return ControllerGenesisState{
		Params: controllertypes.DefaultParams(),
	}
}

// NewControllerGenesisState creates a returns a new ControllerGenesisState instance
func NewControllerGenesisState(channels []ActiveChannel, accounts []RegisteredInterchainAccount, ports []string, controllerParams controllertypes.Params) ControllerGenesisState {
	return ControllerGenesisState{
		ActiveChannels:     channels,
		InterchainAccounts: accounts,
		Ports:              ports,
		Params:             controllerParams,
	}
}

// Validate performs basic validation of the ControllerGenesisState
func (gs ControllerGenesisState) Validate() error {
	for _, ch := range gs.ActiveChannels {
		if err := host.ChannelIdentifierValidator(ch.ChannelId); err != nil {
			return err
		}

		if err := host.PortIdentifierValidator(ch.PortId); err != nil {
			return err
		}
	}

	for _, acc := range gs.InterchainAccounts {
		if err := host.PortIdentifierValidator(acc.PortId); err != nil {
			return err
		}

		if err := icatypes.ValidateAccountAddress(acc.AccountAddress); err != nil {
			return err
		}
	}

	for _, port := range gs.Ports {
		if err := host.PortIdentifierValidator(port); err != nil {
			return err
		}
	}

	return nil
}

// DefaultHostGenesis creates and returns the default interchain accounts HostGenesisState
func DefaultHostGenesis() HostGenesisState {
	return HostGenesisState{
		Port:   icatypes.HostPortID,
		Params: hosttypes.DefaultParams(),
	}
}

// NewHostGenesisState creates a returns a new HostGenesisState instance
func NewHostGenesisState(channels []ActiveChannel, accounts []RegisteredInterchainAccount, port string, hostParams hosttypes.Params) HostGenesisState {
	return HostGenesisState{
		ActiveChannels:     channels,
		InterchainAccounts: accounts,
		Port:               port,
		Params:             hostParams,
	}
}

// Validate performs basic validation of the HostGenesisState
func (gs HostGenesisState) Validate() error {
	for _, ch := range gs.ActiveChannels {
		if err := host.ChannelIdentifierValidator(ch.ChannelId); err != nil {
			return err
		}

		if err := host.PortIdentifierValidator(ch.PortId); err != nil {
			return err
		}
	}

	for _, acc := range gs.InterchainAccounts {
		if err := host.PortIdentifierValidator(acc.PortId); err != nil {
			return err
		}

		if err := icatypes.ValidateAccountAddress(acc.AccountAddress); err != nil {
			return err
		}
	}

	if err := host.PortIdentifierValidator(gs.Port); err != nil {
		return err
	}

	return gs.Params.Validate()
}
