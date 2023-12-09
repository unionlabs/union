package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
)

// InitGenesis initializes the fee middleware application state from a provided genesis state
func (k Keeper) InitGenesis(ctx sdk.Context, state types.GenesisState) {
	for _, identifiedFees := range state.IdentifiedFees {
		k.SetFeesInEscrow(ctx, identifiedFees.PacketId, types.NewPacketFees(identifiedFees.PacketFees))
	}

	for _, registeredPayee := range state.RegisteredPayees {
		k.SetPayeeAddress(ctx, registeredPayee.Relayer, registeredPayee.Payee, registeredPayee.ChannelId)
	}

	for _, registeredCounterpartyPayee := range state.RegisteredCounterpartyPayees {
		k.SetCounterpartyPayeeAddress(ctx, registeredCounterpartyPayee.Relayer, registeredCounterpartyPayee.CounterpartyPayee, registeredCounterpartyPayee.ChannelId)
	}

	for _, forwardAddr := range state.ForwardRelayers {
		k.SetRelayerAddressForAsyncAck(ctx, forwardAddr.PacketId, forwardAddr.Address)
	}

	for _, enabledChan := range state.FeeEnabledChannels {
		k.SetFeeEnabled(ctx, enabledChan.PortId, enabledChan.ChannelId)
	}
}

// ExportGenesis returns the fee middleware application exported genesis
func (k Keeper) ExportGenesis(ctx sdk.Context) *types.GenesisState {
	return &types.GenesisState{
		IdentifiedFees:               k.GetAllIdentifiedPacketFees(ctx),
		FeeEnabledChannels:           k.GetAllFeeEnabledChannels(ctx),
		RegisteredPayees:             k.GetAllPayees(ctx),
		RegisteredCounterpartyPayees: k.GetAllCounterpartyPayees(ctx),
		ForwardRelayers:              k.GetAllForwardRelayerAddresses(ctx),
	}
}
