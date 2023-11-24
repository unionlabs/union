package keeper

import (
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// Middleware must implement types.ChannelKeeper and types.PortKeeper expected interfaces
// so that it can wrap IBC channel and port logic for underlying application.
var (
	_ types.ChannelKeeper = (*Keeper)(nil)
	_ types.PortKeeper    = (*Keeper)(nil)
)

// Keeper defines the IBC fungible transfer keeper
type Keeper struct {
	storeKey storetypes.StoreKey
	cdc      codec.BinaryCodec

	authKeeper    types.AccountKeeper
	ics4Wrapper   porttypes.ICS4Wrapper
	channelKeeper types.ChannelKeeper
	portKeeper    types.PortKeeper
	bankKeeper    types.BankKeeper
}

// NewKeeper creates a new 29-fee Keeper instance
func NewKeeper(
	cdc codec.BinaryCodec, key storetypes.StoreKey,
	ics4Wrapper porttypes.ICS4Wrapper, channelKeeper types.ChannelKeeper,
	portKeeper types.PortKeeper, authKeeper types.AccountKeeper, bankKeeper types.BankKeeper,
) Keeper {
	return Keeper{
		cdc:           cdc,
		storeKey:      key,
		ics4Wrapper:   ics4Wrapper,
		channelKeeper: channelKeeper,
		portKeeper:    portKeeper,
		authKeeper:    authKeeper,
		bankKeeper:    bankKeeper,
	}
}

// WithICS4Wrapper sets the ICS4Wrapper. This function may be used after
// the keepers creation to set the middleware which is above this module
// in the IBC application stack.
func (k *Keeper) WithICS4Wrapper(wrapper porttypes.ICS4Wrapper) {
	k.ics4Wrapper = wrapper
}

// Logger returns a module-specific logger.
func (Keeper) Logger(ctx sdk.Context) log.Logger {
	return ctx.Logger().With("module", "x/"+ibcexported.ModuleName+"-"+types.ModuleName)
}

// BindPort defines a wrapper function for the port Keeper's function in
// order to expose it to module's InitGenesis function
func (k Keeper) BindPort(ctx sdk.Context, portID string) *capabilitytypes.Capability {
	return k.portKeeper.BindPort(ctx, portID)
}

// GetChannel wraps IBC ChannelKeeper's GetChannel function
func (k Keeper) GetChannel(ctx sdk.Context, portID, channelID string) (channeltypes.Channel, bool) {
	return k.channelKeeper.GetChannel(ctx, portID, channelID)
}

// GetPacketCommitment wraps IBC ChannelKeeper's GetPacketCommitment function
func (k Keeper) GetPacketCommitment(ctx sdk.Context, portID, channelID string, sequence uint64) []byte {
	return k.channelKeeper.GetPacketCommitment(ctx, portID, channelID, sequence)
}

// GetNextSequenceSend wraps IBC ChannelKeeper's GetNextSequenceSend function
func (k Keeper) GetNextSequenceSend(ctx sdk.Context, portID, channelID string) (uint64, bool) {
	return k.channelKeeper.GetNextSequenceSend(ctx, portID, channelID)
}

// GetFeeModuleAddress returns the ICS29 Fee ModuleAccount address
func (k Keeper) GetFeeModuleAddress() sdk.AccAddress {
	return k.authKeeper.GetModuleAddress(types.ModuleName)
}

// EscrowAccountHasBalance verifies if the escrow account has the provided fee.
func (k Keeper) EscrowAccountHasBalance(ctx sdk.Context, coins sdk.Coins) bool {
	for _, coin := range coins {
		if !k.bankKeeper.HasBalance(ctx, k.GetFeeModuleAddress(), coin) {
			return false
		}
	}

	return true
}

// lockFeeModule sets a flag to determine if fee handling logic should run for the given channel
// identified by channel and port identifiers.
// Please see ADR 004 for more information.
func (k Keeper) lockFeeModule(ctx sdk.Context) {
	store := ctx.KVStore(k.storeKey)
	store.Set(types.KeyLocked(), []byte{1})
}

// IsLocked indicates if the fee module is locked
// Please see ADR 004 for more information.
func (k Keeper) IsLocked(ctx sdk.Context) bool {
	store := ctx.KVStore(k.storeKey)
	return store.Has(types.KeyLocked())
}

// SetFeeEnabled sets a flag to determine if fee handling logic should run for the given channel
// identified by channel and port identifiers.
func (k Keeper) SetFeeEnabled(ctx sdk.Context, portID, channelID string) {
	store := ctx.KVStore(k.storeKey)
	store.Set(types.KeyFeeEnabled(portID, channelID), []byte{1})
}

// DeleteFeeEnabled deletes the fee enabled flag for a given portID and channelID
func (k Keeper) DeleteFeeEnabled(ctx sdk.Context, portID, channelID string) {
	store := ctx.KVStore(k.storeKey)
	store.Delete(types.KeyFeeEnabled(portID, channelID))
}

// IsFeeEnabled returns whether fee handling logic should be run for the given port. It will check the
// fee enabled flag for the given port and channel identifiers
func (k Keeper) IsFeeEnabled(ctx sdk.Context, portID, channelID string) bool {
	store := ctx.KVStore(k.storeKey)
	return store.Has(types.KeyFeeEnabled(portID, channelID))
}

// GetAllFeeEnabledChannels returns a list of all ics29 enabled channels containing portID & channelID that are stored in state
func (k Keeper) GetAllFeeEnabledChannels(ctx sdk.Context) []types.FeeEnabledChannel {
	store := ctx.KVStore(k.storeKey)
	iterator := storetypes.KVStorePrefixIterator(store, []byte(types.FeeEnabledKeyPrefix))
	defer sdk.LogDeferred(ctx.Logger(), func() error { return iterator.Close() })

	var enabledChArr []types.FeeEnabledChannel
	for ; iterator.Valid(); iterator.Next() {
		portID, channelID, err := types.ParseKeyFeeEnabled(string(iterator.Key()))
		if err != nil {
			panic(err)
		}
		ch := types.FeeEnabledChannel{
			PortId:    portID,
			ChannelId: channelID,
		}

		enabledChArr = append(enabledChArr, ch)
	}

	return enabledChArr
}

// GetPayeeAddress retrieves the fee payee address stored in state given the provided channel identifier and relayer address
func (k Keeper) GetPayeeAddress(ctx sdk.Context, relayerAddr, channelID string) (string, bool) {
	store := ctx.KVStore(k.storeKey)
	key := types.KeyPayee(relayerAddr, channelID)

	if !store.Has(key) {
		return "", false
	}

	return string(store.Get(key)), true
}

// SetPayeeAddress stores the fee payee address in state keyed by the provided channel identifier and relayer address
func (k Keeper) SetPayeeAddress(ctx sdk.Context, relayerAddr, payeeAddr, channelID string) {
	store := ctx.KVStore(k.storeKey)
	store.Set(types.KeyPayee(relayerAddr, channelID), []byte(payeeAddr))
}

// GetAllPayees returns all registered payees addresses
func (k Keeper) GetAllPayees(ctx sdk.Context) []types.RegisteredPayee {
	store := ctx.KVStore(k.storeKey)
	iterator := storetypes.KVStorePrefixIterator(store, []byte(types.PayeeKeyPrefix))
	defer sdk.LogDeferred(ctx.Logger(), func() error { return iterator.Close() })

	var registeredPayees []types.RegisteredPayee
	for ; iterator.Valid(); iterator.Next() {
		relayerAddr, channelID, err := types.ParseKeyPayeeAddress(string(iterator.Key()))
		if err != nil {
			panic(err)
		}

		payee := types.RegisteredPayee{
			Relayer:   relayerAddr,
			Payee:     string(iterator.Value()),
			ChannelId: channelID,
		}

		registeredPayees = append(registeredPayees, payee)
	}

	return registeredPayees
}

// SetCounterpartyPayeeAddress maps the destination chain counterparty payee address to the source relayer address
// The receiving chain must store the mapping from: address -> counterpartyPayeeAddress for the given channel
func (k Keeper) SetCounterpartyPayeeAddress(ctx sdk.Context, address, counterpartyAddress, channelID string) {
	store := ctx.KVStore(k.storeKey)
	store.Set(types.KeyCounterpartyPayee(address, channelID), []byte(counterpartyAddress))
}

// GetCounterpartyPayeeAddress gets the counterparty payee address given a destination relayer address
func (k Keeper) GetCounterpartyPayeeAddress(ctx sdk.Context, address, channelID string) (string, bool) {
	store := ctx.KVStore(k.storeKey)
	key := types.KeyCounterpartyPayee(address, channelID)

	if !store.Has(key) {
		return "", false
	}

	addr := string(store.Get(key))
	return addr, true
}

// GetAllCounterpartyPayees returns all registered counterparty payee addresses
func (k Keeper) GetAllCounterpartyPayees(ctx sdk.Context) []types.RegisteredCounterpartyPayee {
	store := ctx.KVStore(k.storeKey)
	iterator := storetypes.KVStorePrefixIterator(store, []byte(types.CounterpartyPayeeKeyPrefix))
	defer sdk.LogDeferred(ctx.Logger(), func() error { return iterator.Close() })

	var registeredCounterpartyPayees []types.RegisteredCounterpartyPayee
	for ; iterator.Valid(); iterator.Next() {
		relayerAddr, channelID, err := types.ParseKeyCounterpartyPayee(string(iterator.Key()))
		if err != nil {
			panic(err)
		}

		counterpartyPayee := types.RegisteredCounterpartyPayee{
			Relayer:           relayerAddr,
			CounterpartyPayee: string(iterator.Value()),
			ChannelId:         channelID,
		}

		registeredCounterpartyPayees = append(registeredCounterpartyPayees, counterpartyPayee)
	}

	return registeredCounterpartyPayees
}

// SetRelayerAddressForAsyncAck sets the forward relayer address during OnRecvPacket in case of async acknowledgement
func (k Keeper) SetRelayerAddressForAsyncAck(ctx sdk.Context, packetID channeltypes.PacketId, address string) {
	store := ctx.KVStore(k.storeKey)
	store.Set(types.KeyRelayerAddressForAsyncAck(packetID), []byte(address))
}

// GetRelayerAddressForAsyncAck gets forward relayer address for a particular packet
func (k Keeper) GetRelayerAddressForAsyncAck(ctx sdk.Context, packetID channeltypes.PacketId) (string, bool) {
	store := ctx.KVStore(k.storeKey)
	key := types.KeyRelayerAddressForAsyncAck(packetID)
	if !store.Has(key) {
		return "", false
	}

	addr := string(store.Get(key))
	return addr, true
}

// GetAllForwardRelayerAddresses returns all forward relayer addresses stored for async acknowledgements
func (k Keeper) GetAllForwardRelayerAddresses(ctx sdk.Context) []types.ForwardRelayerAddress {
	store := ctx.KVStore(k.storeKey)
	iterator := storetypes.KVStorePrefixIterator(store, []byte(types.ForwardRelayerPrefix))
	defer sdk.LogDeferred(ctx.Logger(), func() error { return iterator.Close() })

	var forwardRelayerAddr []types.ForwardRelayerAddress
	for ; iterator.Valid(); iterator.Next() {
		packetID, err := types.ParseKeyRelayerAddressForAsyncAck(string(iterator.Key()))
		if err != nil {
			panic(err)
		}

		addr := types.ForwardRelayerAddress{
			Address:  string(iterator.Value()),
			PacketId: packetID,
		}

		forwardRelayerAddr = append(forwardRelayerAddr, addr)
	}

	return forwardRelayerAddr
}

// Deletes the forwardRelayerAddr associated with the packetID
func (k Keeper) DeleteForwardRelayerAddress(ctx sdk.Context, packetID channeltypes.PacketId) {
	store := ctx.KVStore(k.storeKey)
	key := types.KeyRelayerAddressForAsyncAck(packetID)
	store.Delete(key)
}

// GetFeesInEscrow returns all escrowed packet fees for a given packetID
func (k Keeper) GetFeesInEscrow(ctx sdk.Context, packetID channeltypes.PacketId) (types.PacketFees, bool) {
	store := ctx.KVStore(k.storeKey)
	key := types.KeyFeesInEscrow(packetID)
	bz := store.Get(key)
	if len(bz) == 0 {
		return types.PacketFees{}, false
	}

	return k.MustUnmarshalFees(bz), true
}

// HasFeesInEscrow returns true if packet fees exist for the provided packetID
func (k Keeper) HasFeesInEscrow(ctx sdk.Context, packetID channeltypes.PacketId) bool {
	store := ctx.KVStore(k.storeKey)
	key := types.KeyFeesInEscrow(packetID)

	return store.Has(key)
}

// SetFeesInEscrow sets the given packet fees in escrow keyed by the packetID
func (k Keeper) SetFeesInEscrow(ctx sdk.Context, packetID channeltypes.PacketId, fees types.PacketFees) {
	store := ctx.KVStore(k.storeKey)
	bz := k.MustMarshalFees(fees)
	store.Set(types.KeyFeesInEscrow(packetID), bz)
}

// DeleteFeesInEscrow deletes the fee associated with the given packetID
func (k Keeper) DeleteFeesInEscrow(ctx sdk.Context, packetID channeltypes.PacketId) {
	store := ctx.KVStore(k.storeKey)
	key := types.KeyFeesInEscrow(packetID)
	store.Delete(key)
}

// GetIdentifiedPacketFeesForChannel returns all the currently escrowed fees on a given channel.
func (k Keeper) GetIdentifiedPacketFeesForChannel(ctx sdk.Context, portID, channelID string) []types.IdentifiedPacketFees {
	var identifiedPacketFees []types.IdentifiedPacketFees

	store := ctx.KVStore(k.storeKey)
	iterator := storetypes.KVStorePrefixIterator(store, types.KeyFeesInEscrowChannelPrefix(portID, channelID))

	defer sdk.LogDeferred(ctx.Logger(), func() error { return iterator.Close() })
	for ; iterator.Valid(); iterator.Next() {
		packetID, err := types.ParseKeyFeesInEscrow(string(iterator.Key()))
		if err != nil {
			panic(err)
		}

		packetFees := k.MustUnmarshalFees(iterator.Value())

		identifiedFee := types.NewIdentifiedPacketFees(packetID, packetFees.PacketFees)
		identifiedPacketFees = append(identifiedPacketFees, identifiedFee)
	}

	return identifiedPacketFees
}

// GetAllIdentifiedPacketFees returns a list of all IdentifiedPacketFees that are stored in state
func (k Keeper) GetAllIdentifiedPacketFees(ctx sdk.Context) []types.IdentifiedPacketFees {
	store := ctx.KVStore(k.storeKey)
	iterator := storetypes.KVStorePrefixIterator(store, []byte(types.FeesInEscrowPrefix))
	defer sdk.LogDeferred(ctx.Logger(), func() error { return iterator.Close() })

	var identifiedFees []types.IdentifiedPacketFees
	for ; iterator.Valid(); iterator.Next() {
		packetID, err := types.ParseKeyFeesInEscrow(string(iterator.Key()))
		if err != nil {
			panic(err)
		}

		feesInEscrow := k.MustUnmarshalFees(iterator.Value())

		identifiedFee := types.IdentifiedPacketFees{
			PacketId:   packetID,
			PacketFees: feesInEscrow.PacketFees,
		}

		identifiedFees = append(identifiedFees, identifiedFee)
	}

	return identifiedFees
}

// MustMarshalFees attempts to encode a Fee object and returns the
// raw encoded bytes. It panics on error.
func (k Keeper) MustMarshalFees(fees types.PacketFees) []byte {
	return k.cdc.MustMarshal(&fees)
}

// MustUnmarshalFees attempts to decode and return a Fee object from
// raw encoded bytes. It panics on error.
func (k Keeper) MustUnmarshalFees(bz []byte) types.PacketFees {
	var fees types.PacketFees
	k.cdc.MustUnmarshal(bz, &fees)
	return fees
}
