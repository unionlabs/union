package keeper

import (
	"context"
	"errors"
	"fmt"

	corestore "cosmossdk.io/core/store"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	coretypes "github.com/cosmos/ibc-go/v8/modules/core/types"
)

// Keeper defines the IBC connection keeper
type Keeper struct {
	// implements gRPC QueryServer interface
	types.QueryServer

	storeService   corestore.KVStoreService
	legacySubspace types.ParamSubspace
	cdc            codec.BinaryCodec
	clientKeeper   types.ClientKeeper
	consensusHost  clienttypes.ConsensusHost
}

// NewKeeper creates a new IBC connection Keeper instance
func NewKeeper(cdc codec.BinaryCodec, storeService corestore.KVStoreService, legacySubspace types.ParamSubspace, ck types.ClientKeeper) Keeper {
	return Keeper{
		storeService:   storeService,
		cdc:            cdc,
		legacySubspace: legacySubspace,
		clientKeeper:   ck,
	}
}

// Logger returns a module-specific logger.
func (Keeper) Logger(ctx context.Context) log.Logger {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
	return sdkCtx.Logger().With("module", "x/"+exported.ModuleName+"/"+types.SubModuleName)
}

// SetConsensusHost sets a custom ConsensusHost for self client state and consensus state validation.
func (k *Keeper) SetConsensusHost(consensusHost clienttypes.ConsensusHost) {
	if consensusHost == nil {
		panic(fmt.Errorf("cannot set a nil self consensus host"))
	}

	k.consensusHost = consensusHost
}

// GetCommitmentPrefix returns the IBC connection store prefix as a commitment
// Prefix
func (k Keeper) GetCommitmentPrefix() exported.Prefix {
	return commitmenttypes.NewMerklePrefix([]byte(exported.StoreKey))
}

// GenerateConnectionIdentifier returns the next connection identifier.
func (k Keeper) GenerateConnectionIdentifier(ctx context.Context) string {
	nextConnSeq := k.GetNextConnectionSequence(ctx)
	connectionID := types.FormatConnectionIdentifier(nextConnSeq)

	nextConnSeq++
	k.SetNextConnectionSequence(ctx, nextConnSeq)
	return connectionID
}

// GetConnection returns a connection with a particular identifier
func (k Keeper) GetConnection(ctx context.Context, connectionID string) (types.ConnectionEnd, bool) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.ConnectionKey(connectionID))
	if err != nil {
		panic(err)
	}

	if len(bz) == 0 {
		return types.ConnectionEnd{}, false
	}

	var connection types.ConnectionEnd
	k.cdc.MustUnmarshal(bz, &connection)

	return connection, true
}

// HasConnection returns a true if the connection with the given identifier
// exists in the store.
func (k Keeper) HasConnection(ctx context.Context, connectionID string) bool {
	store := k.storeService.OpenKVStore(ctx)
	has, err := store.Has(host.ConnectionKey(connectionID))
	if err != nil {
		return false
	}
	return has
}

// SetConnection sets a connection to the store
func (k Keeper) SetConnection(ctx context.Context, connectionID string, connection types.ConnectionEnd) {
	store := k.storeService.OpenKVStore(ctx)
	bz := k.cdc.MustMarshal(&connection)
	if err := store.Set(host.ConnectionKey(connectionID), bz); err != nil {
		panic(err)
	}
}

// GetTimestampAtHeight returns the timestamp in nanoseconds of the consensus state at the
// given height.
func (k Keeper) GetTimestampAtHeight(ctx context.Context, connection types.ConnectionEnd, height exported.Height) (uint64, error) {
	clientState, found := k.clientKeeper.GetClientState(ctx, connection.GetClientID())
	if !found {
		return 0, errorsmod.Wrapf(
			clienttypes.ErrClientNotFound, "clientID (%s)", connection.GetClientID(),
		)
	}

	timestamp, err := clientState.GetTimestampAtHeight(ctx, k.clientKeeper.ClientStore(ctx, connection.GetClientID()), k.cdc, height)
	if err != nil {
		return 0, err
	}

	return timestamp, nil
}

// GetClientConnectionPaths returns all the connection paths stored under a
// particular client
func (k Keeper) GetClientConnectionPaths(ctx context.Context, clientID string) ([]string, bool) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(host.ClientConnectionsKey(clientID))
	if err != nil {
		panic(err)
	}

	if len(bz) == 0 {
		return nil, false
	}

	var clientPaths types.ClientPaths
	k.cdc.MustUnmarshal(bz, &clientPaths)
	return clientPaths.Paths, true
}

// SetClientConnectionPaths sets the connections paths for client
func (k Keeper) SetClientConnectionPaths(ctx context.Context, clientID string, paths []string) {
	store := k.storeService.OpenKVStore(ctx)
	clientPaths := types.ClientPaths{Paths: paths}
	bz := k.cdc.MustMarshal(&clientPaths)
	if err := store.Set(host.ClientConnectionsKey(clientID), bz); err != nil {
		panic(err)
	}
}

// GetNextConnectionSequence gets the next connection sequence from the store.
func (k Keeper) GetNextConnectionSequence(ctx context.Context) uint64 {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get([]byte(types.KeyNextConnectionSequence))
	if err != nil {
		panic(err)
	}

	if len(bz) == 0 {
		panic(errors.New("next connection sequence is nil"))
	}

	return sdk.BigEndianToUint64(bz)
}

// SetNextConnectionSequence sets the next connection sequence to the store.
func (k Keeper) SetNextConnectionSequence(ctx context.Context, sequence uint64) {
	store := k.storeService.OpenKVStore(ctx)
	bz := sdk.Uint64ToBigEndian(sequence)
	if err := store.Set([]byte(types.KeyNextConnectionSequence), bz); err != nil {
		panic(err)
	}
}

// GetAllClientConnectionPaths returns all stored clients connection id paths. It
// will ignore the clients that haven't initialized a connection handshake since
// no paths are stored.
func (k Keeper) GetAllClientConnectionPaths(ctx context.Context) []types.ConnectionPaths {
	var allConnectionPaths []types.ConnectionPaths
	k.clientKeeper.IterateClientStates(ctx, nil, func(clientID string, cs exported.ClientState) bool {
		paths, found := k.GetClientConnectionPaths(ctx, clientID)
		if !found {
			// continue when connection handshake is not initialized
			return false
		}
		connPaths := types.NewConnectionPaths(clientID, paths)
		allConnectionPaths = append(allConnectionPaths, connPaths)
		return false
	})

	return allConnectionPaths
}

// IterateConnections provides an iterator over all ConnectionEnd objects.
// For each ConnectionEnd, cb will be called. If the cb returns true, the
// iterator will close and stop.
func (k Keeper) IterateConnections(ctx context.Context, cb func(types.IdentifiedConnection) bool) {
	store := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))

	iterator := storetypes.KVStorePrefixIterator(store, []byte(host.KeyConnectionPrefix))

	defer coretypes.LogDeferred(k.Logger(ctx), func() error { return iterator.Close() })
	for ; iterator.Valid(); iterator.Next() {
		var connection types.ConnectionEnd
		k.cdc.MustUnmarshal(iterator.Value(), &connection)

		connectionID := host.MustParseConnectionPath(string(iterator.Key()))
		identifiedConnection := types.NewIdentifiedConnection(connectionID, connection)
		if cb(identifiedConnection) {
			break
		}
	}
}

// GetAllConnections returns all stored ConnectionEnd objects.
func (k Keeper) GetAllConnections(ctx context.Context) (connections []types.IdentifiedConnection) {
	k.IterateConnections(ctx, func(connection types.IdentifiedConnection) bool {
		connections = append(connections, connection)
		return false
	})
	return connections
}

// CreateSentinelLocalhostConnection creates and sets the sentinel localhost connection end in the IBC store.
func (k Keeper) CreateSentinelLocalhostConnection(ctx context.Context) {
	counterparty := types.NewCounterparty(exported.LocalhostClientID, exported.LocalhostConnectionID, commitmenttypes.NewMerklePrefix(k.GetCommitmentPrefix().Bytes()))
	connectionEnd := types.NewConnectionEnd(types.OPEN, exported.LocalhostClientID, counterparty, types.GetCompatibleVersions(), 0)

	k.SetConnection(ctx, exported.LocalhostConnectionID, connectionEnd)
}

// addConnectionToClient is used to add a connection identifier to the set of
// connections associated with a client.
func (k Keeper) addConnectionToClient(ctx context.Context, clientID, connectionID string) error {
	_, found := k.clientKeeper.GetClientState(ctx, clientID)
	if !found {
		return errorsmod.Wrap(clienttypes.ErrClientNotFound, clientID)
	}

	conns, found := k.GetClientConnectionPaths(ctx, clientID)
	if !found {
		conns = []string{}
	}

	conns = append(conns, connectionID)
	k.SetClientConnectionPaths(ctx, clientID, conns)
	return nil
}

// GetParams returns the total set of ibc-connection parameters.
func (k Keeper) GetParams(ctx context.Context) types.Params {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get([]byte(types.ParamsKey))
	if err != nil {
		panic(err)
	}

	if bz == nil { // only panic on unset params and not on empty params
		panic(errors.New("connection params are not set in store"))
	}

	var params types.Params
	k.cdc.MustUnmarshal(bz, &params)
	return params
}

// SetParams sets the total set of ibc-connection parameters.
func (k Keeper) SetParams(ctx context.Context, params types.Params) {
	store := k.storeService.OpenKVStore(ctx)
	bz := k.cdc.MustMarshal(&params)
	if err := store.Set([]byte(types.ParamsKey), bz); err != nil {
		panic(err)
	}
}
