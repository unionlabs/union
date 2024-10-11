package keeper

import (
	"context"
	"errors"
	"strconv"
	"strings"

	db "github.com/cosmos/cosmos-db"

	corestore "cosmossdk.io/core/store"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	coretypes "github.com/cosmos/ibc-go/v8/modules/core/types"
)

var _ porttypes.ICS4Wrapper = (*Keeper)(nil)

// Keeper defines the IBC channel keeper
type Keeper struct {
	// implements gRPC QueryServer interface
	types.QueryServer

	storeService     corestore.KVStoreService
	cdc              codec.BinaryCodec
	clientKeeper     types.ClientKeeper
	connectionKeeper types.ConnectionKeeper
	portKeeper       types.PortKeeper
	scopedKeeper     exported.ScopedKeeper
}

// NewKeeper creates a new IBC channel Keeper instance
func NewKeeper(
	cdc codec.BinaryCodec,
	storeService corestore.KVStoreService,
	clientKeeper types.ClientKeeper, connectionKeeper types.ConnectionKeeper,
	portKeeper types.PortKeeper, scopedKeeper exported.ScopedKeeper,
) Keeper {
	return Keeper{
		storeService:     storeService,
		cdc:              cdc,
		clientKeeper:     clientKeeper,
		connectionKeeper: connectionKeeper,
		portKeeper:       portKeeper,
		scopedKeeper:     scopedKeeper,
	}
}

// Logger returns a module-specific logger.
func (Keeper) Logger(ctx context.Context) log.Logger {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
	return sdkCtx.Logger().With("module", "x/"+exported.ModuleName+"/"+types.SubModuleName)
}

// GenerateChannelIdentifier returns the next channel identifier.
func (k Keeper) GenerateChannelIdentifier(ctx context.Context) string {
	nextChannelSeq := k.GetNextChannelSequence(ctx)
	channelID := types.FormatChannelIdentifier(nextChannelSeq)

	nextChannelSeq++
	k.SetNextChannelSequence(ctx, nextChannelSeq)
	return channelID
}

// HasChannel true if the channel with the given identifiers exists in state.
func (k Keeper) HasChannel(ctx context.Context, portID, channelID string) bool {
	store := k.storeService.OpenKVStore(ctx)
	has, err := store.Has(host.ChannelKey(portID, channelID))
	if err != nil {
		panic(err)
	}
	return has
}

// GetChannel returns a channel with a particular identifier binded to a specific port
func (k Keeper) GetChannel(ctx context.Context, portID, channelID string) (types.Channel, bool) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.ChannelKey(portID, channelID))
	if err != nil {
		panic(err)
	}
	if len(bz) == 0 {
		return types.Channel{}, false
	}

	var channel types.Channel
	k.cdc.MustUnmarshal(bz, &channel)
	return channel, true
}

// SetChannel sets a channel to the store
func (k Keeper) SetChannel(ctx context.Context, portID, channelID string, channel types.Channel) {
	store := k.storeService.OpenKVStore(ctx)
	bz := k.cdc.MustMarshal(&channel)
	if err := store.Set(host.ChannelKey(portID, channelID), bz); err != nil {
		panic(err)
	}
}

// GetAppVersion gets the version for the specified channel.
func (k Keeper) GetAppVersion(ctx context.Context, portID, channelID string) (string, bool) {
	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		return "", false
	}

	return channel.Version, true
}

// GetNextChannelSequence gets the next channel sequence from the store.
func (k Keeper) GetNextChannelSequence(ctx context.Context) uint64 {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get([]byte(types.KeyNextChannelSequence))
	if err != nil {
		panic(err)
	}
	if len(bz) == 0 {
		panic(errors.New("next channel sequence is nil"))
	}

	return sdk.BigEndianToUint64(bz)
}

// SetNextChannelSequence sets the next channel sequence to the store.
func (k Keeper) SetNextChannelSequence(ctx context.Context, sequence uint64) {
	store := k.storeService.OpenKVStore(ctx)
	bz := sdk.Uint64ToBigEndian(sequence)
	if err := store.Set([]byte(types.KeyNextChannelSequence), bz); err != nil {
		panic(err)
	}
}

// GetNextSequenceSend gets a channel's next send sequence from the store
func (k Keeper) GetNextSequenceSend(ctx context.Context, portID, channelID string) (uint64, bool) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.NextSequenceSendKey(portID, channelID))
	if err != nil {
		panic(err)
	}
	if len(bz) == 0 {
		return 0, false
	}

	return sdk.BigEndianToUint64(bz), true
}

// SetNextSequenceSend sets a channel's next send sequence to the store
func (k Keeper) SetNextSequenceSend(ctx context.Context, portID, channelID string, sequence uint64) {
	store := k.storeService.OpenKVStore(ctx)
	bz := sdk.Uint64ToBigEndian(sequence)
	if err := store.Set(host.NextSequenceSendKey(portID, channelID), bz); err != nil {
		panic(err)
	}
}

// GetNextSequenceRecv gets a channel's next receive sequence from the store
func (k Keeper) GetNextSequenceRecv(ctx context.Context, portID, channelID string) (uint64, bool) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.NextSequenceRecvKey(portID, channelID))
	if err != nil {
		panic(err)
	}
	if len(bz) == 0 {
		return 0, false
	}

	return sdk.BigEndianToUint64(bz), true
}

// SetNextSequenceRecv sets a channel's next receive sequence to the store
func (k Keeper) SetNextSequenceRecv(ctx context.Context, portID, channelID string, sequence uint64) {
	store := k.storeService.OpenKVStore(ctx)
	bz := sdk.Uint64ToBigEndian(sequence)
	if err := store.Set(host.NextSequenceRecvKey(portID, channelID), bz); err != nil {
		panic(err)
	}
}

// GetNextSequenceAck gets a channel's next ack sequence from the store
func (k Keeper) GetNextSequenceAck(ctx context.Context, portID, channelID string) (uint64, bool) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.NextSequenceAckKey(portID, channelID))
	if err != nil {
		panic(err)
	}

	if len(bz) == 0 {
		return 0, false
	}

	return sdk.BigEndianToUint64(bz), true
}

// SetNextSequenceAck sets a channel's next ack sequence to the store
func (k Keeper) SetNextSequenceAck(ctx context.Context, portID, channelID string, sequence uint64) {
	store := k.storeService.OpenKVStore(ctx)
	bz := sdk.Uint64ToBigEndian(sequence)
	if err := store.Set(host.NextSequenceAckKey(portID, channelID), bz); err != nil {
		panic(err)
	}
}

// GetPacketReceipt gets a packet receipt from the store
func (k Keeper) GetPacketReceipt(ctx context.Context, portID, channelID string, sequence uint64) (string, bool) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.PacketReceiptKey(portID, channelID, sequence))
	if err != nil {
		panic(err)
	}

	if len(bz) == 0 {
		return "", false
	}

	return string(bz), true
}

// SetPacketReceipt sets an empty packet receipt to the store
func (k Keeper) SetPacketReceipt(ctx context.Context, portID, channelID string, sequence uint64) {
	store := k.storeService.OpenKVStore(ctx)
	if err := store.Set(host.PacketReceiptKey(portID, channelID, sequence), []byte{byte(1)}); err != nil {
		panic(err)
	}
}

// deletePacketReceipt deletes a packet receipt from the store
func (k Keeper) deletePacketReceipt(ctx context.Context, portID, channelID string, sequence uint64) {
	store := k.storeService.OpenKVStore(ctx)
	if err := store.Delete(host.PacketReceiptKey(portID, channelID, sequence)); err != nil {
		panic(err)
	}
}

// GetPacketCommitment gets the packet commitment hash from the store
func (k Keeper) GetPacketCommitment(ctx context.Context, portID, channelID string, sequence uint64) []byte {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.PacketCommitmentKey(portID, channelID, sequence))
	if err != nil {
		panic(err)
	}

	return bz
}

// HasPacketCommitment returns true if the packet commitment exists
func (k Keeper) HasPacketCommitment(ctx context.Context, portID, channelID string, sequence uint64) bool {
	store := k.storeService.OpenKVStore(ctx)
	has, err := store.Has(host.PacketCommitmentKey(portID, channelID, sequence))
	if err != nil {
		panic(err)
	}
	return has
}

// SetPacketCommitment sets the packet commitment hash to the store
func (k Keeper) SetPacketCommitment(ctx context.Context, portID, channelID string, sequence uint64, commitmentHash []byte) {
	store := k.storeService.OpenKVStore(ctx)
	if err := store.Set(host.PacketCommitmentKey(portID, channelID, sequence), commitmentHash); err != nil {
		panic(err)
	}
}

func (k Keeper) deletePacketCommitment(ctx context.Context, portID, channelID string, sequence uint64) {
	store := k.storeService.OpenKVStore(ctx)
	if err := store.Delete(host.PacketCommitmentKey(portID, channelID, sequence)); err != nil {
		panic(err)
	}
}

// SetPacketAcknowledgement sets the packet ack hash to the store
func (k Keeper) SetPacketAcknowledgement(ctx context.Context, portID, channelID string, sequence uint64, ackHash []byte) {
	store := k.storeService.OpenKVStore(ctx)
	if err := store.Set(host.PacketAcknowledgementKey(portID, channelID, sequence), ackHash); err != nil {
		panic(err)
	}
}

// GetPacketAcknowledgement gets the packet ack hash from the store
func (k Keeper) GetPacketAcknowledgement(ctx context.Context, portID, channelID string, sequence uint64) ([]byte, bool) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.PacketAcknowledgementKey(portID, channelID, sequence))
	if err != nil {
		panic(err)
	}

	if len(bz) == 0 {
		return nil, false
	}

	return bz, true
}

// HasPacketAcknowledgement check if the packet ack hash is already on the store
func (k Keeper) HasPacketAcknowledgement(ctx context.Context, portID, channelID string, sequence uint64) bool {
	store := k.storeService.OpenKVStore(ctx)
	has, err := store.Has(host.PacketAcknowledgementKey(portID, channelID, sequence))
	if err != nil {
		panic(err)
	}
	return has
}

// deletePacketAcknowledgement deletes the packet ack hash from the store
func (k Keeper) deletePacketAcknowledgement(ctx context.Context, portID, channelID string, sequence uint64) {
	store := k.storeService.OpenKVStore(ctx)
	if err := store.Delete(host.PacketAcknowledgementKey(portID, channelID, sequence)); err != nil {
		panic(err)
	}
}

// IteratePacketSequence provides an iterator over all send, receive or ack sequences.
// For each sequence, cb will be called. If the cb returns true, the iterator
// will close and stop.
func (k *Keeper) IteratePacketSequence(ctx context.Context, iterator db.Iterator, cb func(portID, channelID string, sequence uint64) bool) {
	defer coretypes.LogDeferred(k.Logger(ctx), func() error { return iterator.Close() })
	for ; iterator.Valid(); iterator.Next() {
		portID, channelID, err := host.ParseChannelPath(string(iterator.Key()))
		if err != nil {
			// return if the key is not a channel key
			return
		}

		sequence := sdk.BigEndianToUint64(iterator.Value())

		if cb(portID, channelID, sequence) {
			break
		}
	}
}

// GetAllPacketSendSeqs returns all stored next send sequences.
func (k *Keeper) GetAllPacketSendSeqs(ctx context.Context) (seqs []types.PacketSequence) {
	store := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	iterator := storetypes.KVStorePrefixIterator(store, []byte(host.KeyNextSeqSendPrefix))
	k.IteratePacketSequence(ctx, iterator, func(portID, channelID string, nextSendSeq uint64) bool {
		ps := types.NewPacketSequence(portID, channelID, nextSendSeq)
		seqs = append(seqs, ps)
		return false
	})
	return seqs
}

// GetAllPacketRecvSeqs returns all stored next recv sequences.
func (k *Keeper) GetAllPacketRecvSeqs(ctx context.Context) (seqs []types.PacketSequence) {
	store := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	iterator := storetypes.KVStorePrefixIterator(store, []byte(host.KeyNextSeqRecvPrefix))
	k.IteratePacketSequence(ctx, iterator, func(portID, channelID string, nextRecvSeq uint64) bool {
		ps := types.NewPacketSequence(portID, channelID, nextRecvSeq)
		seqs = append(seqs, ps)
		return false
	})
	return seqs
}

// GetAllPacketAckSeqs returns all stored next acknowledgements sequences.
func (k *Keeper) GetAllPacketAckSeqs(ctx context.Context) (seqs []types.PacketSequence) {
	store := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	iterator := storetypes.KVStorePrefixIterator(store, []byte(host.KeyNextSeqAckPrefix))
	k.IteratePacketSequence(ctx, iterator, func(portID, channelID string, nextAckSeq uint64) bool {
		ps := types.NewPacketSequence(portID, channelID, nextAckSeq)
		seqs = append(seqs, ps)
		return false
	})
	return seqs
}

// IteratePacketCommitment provides an iterator over all PacketCommitment objects. For each
// packet commitment, cb will be called. If the cb returns true, the iterator will close
// and stop.
func (k Keeper) IteratePacketCommitment(ctx context.Context, cb func(portID, channelID string, sequence uint64, hash []byte) bool) {
	store := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	iterator := storetypes.KVStorePrefixIterator(store, []byte(host.KeyPacketCommitmentPrefix))
	k.iterateHashes(ctx, iterator, cb)
}

// GetAllPacketCommitments returns all stored PacketCommitments objects.
func (k Keeper) GetAllPacketCommitments(ctx context.Context) (commitments []types.PacketState) {
	k.IteratePacketCommitment(ctx, func(portID, channelID string, sequence uint64, hash []byte) bool {
		pc := types.NewPacketState(portID, channelID, sequence, hash)
		commitments = append(commitments, pc)
		return false
	})
	return commitments
}

// IteratePacketCommitmentAtChannel provides an iterator over all PacketCommmitment objects
// at a specified channel. For each packet commitment, cb will be called. If the cb returns
// true, the iterator will close and stop.
func (k Keeper) IteratePacketCommitmentAtChannel(ctx context.Context, portID, channelID string, cb func(_, _ string, sequence uint64, hash []byte) bool) {
	store := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	iterator := storetypes.KVStorePrefixIterator(store, []byte(host.PacketCommitmentPrefixPath(portID, channelID)))
	k.iterateHashes(ctx, iterator, cb)
}

// GetAllPacketCommitmentsAtChannel returns all stored PacketCommitments objects for a specified
// port ID and channel ID.
func (k Keeper) GetAllPacketCommitmentsAtChannel(ctx context.Context, portID, channelID string) (commitments []types.PacketState) {
	k.IteratePacketCommitmentAtChannel(ctx, portID, channelID, func(_, _ string, sequence uint64, hash []byte) bool {
		pc := types.NewPacketState(portID, channelID, sequence, hash)
		commitments = append(commitments, pc)
		return false
	})
	return commitments
}

// IteratePacketReceipt provides an iterator over all PacketReceipt objects. For each
// receipt, cb will be called. If the cb returns true, the iterator will close
// and stop.
func (k Keeper) IteratePacketReceipt(ctx context.Context, cb func(portID, channelID string, sequence uint64, receipt []byte) bool) {
	store := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	iterator := storetypes.KVStorePrefixIterator(store, []byte(host.KeyPacketReceiptPrefix))
	k.iterateHashes(ctx, iterator, cb)
}

// GetAllPacketReceipts returns all stored PacketReceipt objects.
func (k Keeper) GetAllPacketReceipts(ctx context.Context) (receipts []types.PacketState) {
	k.IteratePacketReceipt(ctx, func(portID, channelID string, sequence uint64, receipt []byte) bool {
		packetReceipt := types.NewPacketState(portID, channelID, sequence, receipt)
		receipts = append(receipts, packetReceipt)
		return false
	})
	return receipts
}

// IteratePacketAcknowledgement provides an iterator over all PacketAcknowledgement objects. For each
// aknowledgement, cb will be called. If the cb returns true, the iterator will close
// and stop.
func (k Keeper) IteratePacketAcknowledgement(ctx context.Context, cb func(portID, channelID string, sequence uint64, hash []byte) bool) {
	store := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	iterator := storetypes.KVStorePrefixIterator(store, []byte(host.KeyPacketAckPrefix))
	k.iterateHashes(ctx, iterator, cb)
}

// GetAllPacketAcks returns all stored PacketAcknowledgements objects.
func (k Keeper) GetAllPacketAcks(ctx context.Context) (acks []types.PacketState) {
	k.IteratePacketAcknowledgement(ctx, func(portID, channelID string, sequence uint64, ack []byte) bool {
		packetAck := types.NewPacketState(portID, channelID, sequence, ack)
		acks = append(acks, packetAck)
		return false
	})
	return acks
}

// IterateChannels provides an iterator over all Channel objects. For each
// Channel, cb will be called. If the cb returns true, the iterator will close
// and stop.
func (k Keeper) IterateChannels(ctx context.Context, cb func(types.IdentifiedChannel) bool) {
	store := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	iterator := storetypes.KVStorePrefixIterator(store, []byte(host.KeyChannelEndPrefix))

	defer coretypes.LogDeferred(k.Logger(ctx), func() error { return iterator.Close() })
	for ; iterator.Valid(); iterator.Next() {
		var channel types.Channel
		k.cdc.MustUnmarshal(iterator.Value(), &channel)

		portID, channelID := host.MustParseChannelPath(string(iterator.Key()))
		identifiedChannel := types.NewIdentifiedChannel(portID, channelID, channel)
		if cb(identifiedChannel) {
			break
		}
	}
}

// GetAllChannelsWithPortPrefix returns all channels with the specified port prefix. If an empty prefix is provided
// all channels will be returned.
func (k Keeper) GetAllChannelsWithPortPrefix(ctx context.Context, portPrefix string) []types.IdentifiedChannel {
	if strings.TrimSpace(portPrefix) == "" {
		return k.GetAllChannels(ctx)
	}
	store := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	iterator := storetypes.KVStorePrefixIterator(store, types.FilteredPortPrefix(portPrefix))
	defer coretypes.LogDeferred(k.Logger(ctx), func() error { return iterator.Close() })

	var filteredChannels []types.IdentifiedChannel
	for ; iterator.Valid(); iterator.Next() {
		var channel types.Channel
		k.cdc.MustUnmarshal(iterator.Value(), &channel)

		portID, channelID := host.MustParseChannelPath(string(iterator.Key()))
		identifiedChannel := types.NewIdentifiedChannel(portID, channelID, channel)
		filteredChannels = append(filteredChannels, identifiedChannel)
	}
	return filteredChannels
}

// GetAllChannels returns all stored Channel objects.
func (k Keeper) GetAllChannels(ctx context.Context) (channels []types.IdentifiedChannel) {
	k.IterateChannels(ctx, func(channel types.IdentifiedChannel) bool {
		channels = append(channels, channel)
		return false
	})
	return channels
}

// GetChannelClientState returns the associated client state with its ID, from a port and channel identifier.
func (k Keeper) GetChannelClientState(ctx context.Context, portID, channelID string) (string, exported.ClientState, error) {
	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		return "", nil, errorsmod.Wrapf(types.ErrChannelNotFound, "port-id: %s, channel-id: %s", portID, channelID)
	}

	connection, found := k.connectionKeeper.GetConnection(ctx, channel.ConnectionHops[0])
	if !found {
		return "", nil, errorsmod.Wrapf(connectiontypes.ErrConnectionNotFound, "connection-id: %s", channel.ConnectionHops[0])
	}

	clientState, found := k.clientKeeper.GetClientState(ctx, connection.ClientId)
	if !found {
		return "", nil, errorsmod.Wrapf(clienttypes.ErrClientNotFound, "client-id: %s", connection.ClientId)
	}

	return connection.ClientId, clientState, nil
}

// GetConnection wraps the connection keeper's GetConnection function.
func (k Keeper) GetConnection(ctx context.Context, connectionID string) (connectiontypes.ConnectionEnd, error) {
	connection, found := k.connectionKeeper.GetConnection(ctx, connectionID)
	if !found {
		return connectiontypes.ConnectionEnd{}, errorsmod.Wrapf(connectiontypes.ErrConnectionNotFound, "connection-id: %s", connectionID)
	}

	return connection, nil
}

// GetChannelConnection returns the connection ID and state associated with the given port and channel identifier.
func (k Keeper) GetChannelConnection(ctx context.Context, portID, channelID string) (string, exported.ConnectionI, error) {
	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		return "", nil, errorsmod.Wrapf(types.ErrChannelNotFound, "port-id: %s, channel-id: %s", portID, channelID)
	}

	connectionID := channel.ConnectionHops[0]

	connection, found := k.connectionKeeper.GetConnection(ctx, connectionID)
	if !found {
		return "", nil, errorsmod.Wrapf(connectiontypes.ErrConnectionNotFound, "connection-id: %s", connectionID)
	}

	return connectionID, connection, nil
}

// LookupModuleByChannel will return the IBCModule along with the capability associated with a given channel defined by its portID and channelID
func (k Keeper) LookupModuleByChannel(ctx context.Context, portID, channelID string) (string, *capabilitytypes.Capability, error) {
	modules, capability, err := k.scopedKeeper.LookupModules(ctx, host.ChannelCapabilityPath(portID, channelID))
	if err != nil {
		return "", nil, err
	}

	return porttypes.GetModuleOwner(modules), capability, nil
}

// GetUpgradeErrorReceipt returns the upgrade error receipt for the provided port and channel identifiers.
func (k Keeper) GetUpgradeErrorReceipt(ctx context.Context, portID, channelID string) (types.ErrorReceipt, bool) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.ChannelUpgradeErrorKey(portID, channelID))
	if err != nil {
		panic(err)
	}

	if len(bz) == 0 {
		return types.ErrorReceipt{}, false
	}

	var errorReceipt types.ErrorReceipt
	k.cdc.MustUnmarshal(bz, &errorReceipt)

	return errorReceipt, true
}

// setUpgradeErrorReceipt sets the provided error receipt in store using the port and channel identifiers.
func (k Keeper) setUpgradeErrorReceipt(ctx context.Context, portID, channelID string, errorReceipt types.ErrorReceipt) {
	store := k.storeService.OpenKVStore(ctx)
	bz := k.cdc.MustMarshal(&errorReceipt)
	if err := store.Set(host.ChannelUpgradeErrorKey(portID, channelID), bz); err != nil {
		panic(err)
	}
}

// hasUpgrade returns true if a proposed upgrade exists in store
func (k Keeper) hasUpgrade(ctx context.Context, portID, channelID string) bool {
	store := k.storeService.OpenKVStore(ctx)
	has, err := store.Has(host.ChannelUpgradeKey(portID, channelID))
	if err != nil {
		panic(err)
	}
	return has
}

// GetUpgrade returns the proposed upgrade for the provided port and channel identifiers.
func (k Keeper) GetUpgrade(ctx context.Context, portID, channelID string) (types.Upgrade, bool) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.ChannelUpgradeKey(portID, channelID))
	if err != nil {
		panic(err)
	}

	if len(bz) == 0 {
		return types.Upgrade{}, false
	}

	var upgrade types.Upgrade
	k.cdc.MustUnmarshal(bz, &upgrade)

	return upgrade, true
}

// SetUpgrade sets the proposed upgrade using the provided port and channel identifiers.
func (k Keeper) SetUpgrade(ctx context.Context, portID, channelID string, upgrade types.Upgrade) {
	store := k.storeService.OpenKVStore(ctx)
	bz := k.cdc.MustMarshal(&upgrade)
	if err := store.Set(host.ChannelUpgradeKey(portID, channelID), bz); err != nil {
		panic(err)
	}
}

// deleteUpgrade deletes the upgrade for the provided port and channel identifiers.
func (k Keeper) deleteUpgrade(ctx context.Context, portID, channelID string) {
	store := k.storeService.OpenKVStore(ctx)
	if err := store.Delete(host.ChannelUpgradeKey(portID, channelID)); err != nil {
		panic(err)
	}
}

// hasCounterpartyUpgrade returns true if a counterparty upgrade exists in store
func (k Keeper) hasCounterpartyUpgrade(ctx context.Context, portID, channelID string) bool {
	store := k.storeService.OpenKVStore(ctx)
	has, err := store.Has(host.ChannelCounterpartyUpgradeKey(portID, channelID))
	if err != nil {
		panic(err)
	}
	return has
}

// GetCounterpartyUpgrade gets the counterparty upgrade from the store.
func (k Keeper) GetCounterpartyUpgrade(ctx context.Context, portID, channelID string) (types.Upgrade, bool) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.ChannelCounterpartyUpgradeKey(portID, channelID))
	if err != nil {
		panic(err)
	}

	if len(bz) == 0 {
		return types.Upgrade{}, false
	}

	var upgrade types.Upgrade
	k.cdc.MustUnmarshal(bz, &upgrade)

	return upgrade, true
}

// SetCounterpartyUpgrade sets the counterparty upgrade in the store.
func (k Keeper) SetCounterpartyUpgrade(ctx context.Context, portID, channelID string, upgrade types.Upgrade) {
	store := k.storeService.OpenKVStore(ctx)
	bz := k.cdc.MustMarshal(&upgrade)
	if err := store.Set(host.ChannelCounterpartyUpgradeKey(portID, channelID), bz); err != nil {
		panic(err)
	}
}

// deleteCounterpartyUpgrade deletes the counterparty upgrade in the store.
func (k Keeper) deleteCounterpartyUpgrade(ctx context.Context, portID, channelID string) {
	store := k.storeService.OpenKVStore(ctx)
	if err := store.Delete(host.ChannelCounterpartyUpgradeKey(portID, channelID)); err != nil {
		panic(err)
	}
}

// deleteUpgradeInfo deletes all auxiliary upgrade information.
func (k Keeper) deleteUpgradeInfo(ctx context.Context, portID, channelID string) {
	k.deleteUpgrade(ctx, portID, channelID)
	k.deleteCounterpartyUpgrade(ctx, portID, channelID)
}

// SetParams sets the channel parameters.
func (k Keeper) SetParams(ctx context.Context, params types.Params) {
	store := k.storeService.OpenKVStore(ctx)
	bz := k.cdc.MustMarshal(&params)
	if err := store.Set([]byte(types.ParamsKey), bz); err != nil {
		panic(err)
	}
}

// GetParams returns the total set of the channel parameters.
func (k Keeper) GetParams(ctx context.Context) types.Params {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get([]byte(types.ParamsKey))
	if err != nil {
		panic(err)
	}

	if bz == nil { // only panic on unset params and not on empty params
		panic(errors.New("channel params are not set in store"))
	}

	var params types.Params
	k.cdc.MustUnmarshal(bz, &params)
	return params
}

// common functionality for IteratePacketCommitment and IteratePacketAcknowledgement
func (k *Keeper) iterateHashes(ctx context.Context, iterator db.Iterator, cb func(portID, channelID string, sequence uint64, hash []byte) bool) {
	defer coretypes.LogDeferred(k.Logger(ctx), func() error { return iterator.Close() })

	for ; iterator.Valid(); iterator.Next() {
		keySplit := strings.Split(string(iterator.Key()), "/")
		portID := keySplit[2]
		channelID := keySplit[4]

		sequence, err := strconv.ParseUint(keySplit[len(keySplit)-1], 10, 64)
		if err != nil {
			panic(err)
		}

		if cb(portID, channelID, sequence, iterator.Value()) {
			break
		}
	}
}

// HasInflightPackets returns true if there are packet commitments stored at the specified
// port and channel, and false otherwise.
func (k Keeper) HasInflightPackets(ctx context.Context, portID, channelID string) bool {
	store := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	iterator := storetypes.KVStorePrefixIterator(store, []byte(host.PacketCommitmentPrefixPath(portID, channelID)))
	defer coretypes.LogDeferred(k.Logger(ctx), func() error { return iterator.Close() })

	return iterator.Valid()
}

// setRecvStartSequence sets the channel's recv start sequence to the store.
func (k Keeper) setRecvStartSequence(ctx context.Context, portID, channelID string, sequence uint64) {
	store := k.storeService.OpenKVStore(ctx)
	bz := sdk.Uint64ToBigEndian(sequence)
	if err := store.Set(host.RecvStartSequenceKey(portID, channelID), bz); err != nil {
		panic(err)
	}
}

// GetRecvStartSequence gets a channel's recv start sequence from the store.
// The recv start sequence will be set to the counterparty's next sequence send
// upon a successful channel upgrade. It will be used for replay protection of
// historical packets and as the upper bound for pruning stale packet receives.
func (k Keeper) GetRecvStartSequence(ctx context.Context, portID, channelID string) (uint64, bool) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.RecvStartSequenceKey(portID, channelID))
	if err != nil {
		panic(err)
	}

	if len(bz) == 0 {
		return 0, false
	}

	return sdk.BigEndianToUint64(bz), true
}

// SetPruningSequenceStart sets a channel's pruning sequence start to the store.
func (k Keeper) SetPruningSequenceStart(ctx context.Context, portID, channelID string, sequence uint64) {
	store := k.storeService.OpenKVStore(ctx)
	bz := sdk.Uint64ToBigEndian(sequence)
	if err := store.Set(host.PruningSequenceStartKey(portID, channelID), bz); err != nil {
		panic(err)
	}
}

// GetPruningSequenceStart gets a channel's pruning sequence start from the store.
func (k Keeper) GetPruningSequenceStart(ctx context.Context, portID, channelID string) (uint64, bool) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.PruningSequenceStartKey(portID, channelID))
	if err != nil {
		panic(err)
	}

	if len(bz) == 0 {
		return 0, false
	}

	return sdk.BigEndianToUint64(bz), true
}

// HasPruningSequenceStart returns true if the pruning sequence start is set for the specified channel.
func (k Keeper) HasPruningSequenceStart(ctx context.Context, portID, channelID string) bool {
	store := k.storeService.OpenKVStore(ctx)
	has, err := store.Has(host.PruningSequenceStartKey(portID, channelID))
	if err != nil {
		panic(err)
	}
	return has
}

// PruneAcknowledgements prunes packet acknowledgements and receipts that have a sequence number less than pruning sequence end.
// The number of packet acks/receipts pruned is bounded by the limit. Pruning can only occur after a channel has been upgraded.
//
// Pruning sequence start keeps track of the packet ack/receipt that can be pruned next. When it reaches pruningSequenceEnd,
// pruning is complete.
func (k Keeper) PruneAcknowledgements(ctx context.Context, portID, channelID string, limit uint64) (uint64, uint64, error) {
	pruningSequenceStart, found := k.GetPruningSequenceStart(ctx, portID, channelID)
	if !found {
		return 0, 0, errorsmod.Wrapf(types.ErrPruningSequenceStartNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}
	pruningSequenceEnd, found := k.GetRecvStartSequence(ctx, portID, channelID)
	if !found {
		return 0, 0, errorsmod.Wrapf(types.ErrRecvStartSequenceNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	start := pruningSequenceStart
	end := pruningSequenceStart + limit
	for ; start < end; start++ {
		// stop pruning if pruningSequenceStart has reached pruningSequenceEnd, pruningSequenceEnd is
		// set to be equal to the _next_ sequence to be sent by the counterparty.
		if start >= pruningSequenceEnd {
			break
		}

		k.deletePacketAcknowledgement(ctx, portID, channelID, start)

		// NOTE: packet receipts are only relevant for unordered channels.
		k.deletePacketReceipt(ctx, portID, channelID, start)
	}

	// set pruning sequence start to the updated value
	k.SetPruningSequenceStart(ctx, portID, channelID, start)

	totalPruned := start - pruningSequenceStart
	totalRemaining := pruningSequenceEnd - start

	return totalPruned, totalRemaining, nil
}
