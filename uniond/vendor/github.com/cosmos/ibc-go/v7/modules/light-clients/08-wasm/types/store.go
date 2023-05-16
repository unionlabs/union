package types

import (
	"bytes"
	"io"

	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/store/cachekv"
	"github.com/cosmos/cosmos-sdk/store/listenkv"
	"github.com/cosmos/cosmos-sdk/store/tracekv"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	clienttypes "github.com/cosmos/ibc-go/v7/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v7/modules/core/24-host"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"
)

// wrappedStore combines two KVStores into one while transparently routing the calls based on key prefix
type wrappedStore struct {
	first  sdk.KVStore
	second sdk.KVStore

	firstPrefix  []byte
	secondPrefix []byte
}

func newWrappedStore(first, second sdk.KVStore, firstPrefix, secondPrefix []byte) wrappedStore {
	return wrappedStore{
		first:        first,
		second:       second,
		firstPrefix:  firstPrefix,
		secondPrefix: secondPrefix,
	}
}

func (ws wrappedStore) Get(key []byte) []byte {
	return ws.getStore(key).Get(ws.trimPrefix(key))
}

func (ws wrappedStore) Has(key []byte) bool {
	return ws.getStore(key).Has(ws.trimPrefix(key))
}

func (ws wrappedStore) Set(key, value []byte) {
	ws.getStore(key).Set(ws.trimPrefix(key), value)
}

func (ws wrappedStore) Delete(key []byte) {
	ws.getStore(key).Delete(ws.trimPrefix(key))
}

func (ws wrappedStore) GetStoreType() storetypes.StoreType {
	return ws.first.GetStoreType()
}

func (ws wrappedStore) Iterator(start, end []byte) sdk.Iterator {
	return ws.getStore(start).Iterator(ws.trimPrefix(start), ws.trimPrefix(end))
}

func (ws wrappedStore) ReverseIterator(start, end []byte) sdk.Iterator {
	return ws.getStore(start).ReverseIterator(ws.trimPrefix(start), ws.trimPrefix(end))
}

func (ws wrappedStore) CacheWrap() storetypes.CacheWrap {
	return cachekv.NewStore(ws)
}

func (ws wrappedStore) CacheWrapWithTrace(w io.Writer, tc storetypes.TraceContext) storetypes.CacheWrap {
	return cachekv.NewStore(tracekv.NewStore(ws, w, tc))
}

func (ws wrappedStore) CacheWrapWithListeners(storeKey storetypes.StoreKey, listeners []storetypes.WriteListener) storetypes.CacheWrap {
	return cachekv.NewStore(listenkv.NewStore(ws, storeKey, listeners))
}

func (ws wrappedStore) trimPrefix(key []byte) []byte {
	if bytes.HasPrefix(key, ws.firstPrefix) {
		key = bytes.TrimPrefix(key, ws.firstPrefix)
	} else {
		key = bytes.TrimPrefix(key, ws.secondPrefix)
	}

	return key
}

func (ws wrappedStore) getStore(key []byte) sdk.KVStore {
	if bytes.HasPrefix(key, ws.firstPrefix) {
		return ws.first
	}

	return ws.second
}

// setClientState stores the client state.
func setClientState(clientStore sdk.KVStore, cdc codec.BinaryCodec, clientState *ClientState) {
	key := host.ClientStateKey()
	val := clienttypes.MustMarshalClientState(cdc, clientState)
	clientStore.Set(key, val)
}

// setConsensusState stores the consensus state at the given height.
func setConsensusState(clientStore sdk.KVStore, cdc codec.BinaryCodec, consensusState *ConsensusState, height exported.Height) {
	key := host.ConsensusStateKey(height)
	val := clienttypes.MustMarshalConsensusState(cdc, consensusState)
	clientStore.Set(key, val)
}

// getConsensusState retrieves the consensus state from the client prefixed
// store. An error is returned if the consensus state does not exist or it cannot be unmarshalled.
func GetConsensusState(clientStore sdk.KVStore, cdc codec.BinaryCodec, height exported.Height) (*ConsensusState, error) {
	bz := clientStore.Get(host.ConsensusStateKey(height))
	if len(bz) == 0 {
		return nil, sdkerrors.Wrapf(clienttypes.ErrConsensusStateNotFound, "consensus state does not exist for height %s", height)
	}

	consensusStateI, err := clienttypes.UnmarshalConsensusState(cdc, bz)
	if err != nil {
		return nil, sdkerrors.Wrapf(clienttypes.ErrInvalidConsensus, "unmarshal error: %v", err)
	}

	consensusState, ok := consensusStateI.(*ConsensusState)
	if !ok {
		return nil, sdkerrors.Wrapf(clienttypes.ErrInvalidConsensus, "invalid consensus type. expected %T, got %T", &ConsensusState{}, consensusState)
	}

	return consensusState, nil
}

var _ wasmvmtypes.KVStore = &storeAdapter{}

// storeAdapter adapter to bridge SDK store impl to wasmvm
type storeAdapter struct {
	parent sdk.KVStore
}

// newStoreAdapter constructor
func newStoreAdapter(s sdk.KVStore) *storeAdapter {
	if s == nil {
		panic("store must not be nil")
	}
	return &storeAdapter{parent: s}
}

func (s storeAdapter) Get(key []byte) []byte {
	return s.parent.Get(key)
}

func (s storeAdapter) Set(key, value []byte) {
	s.parent.Set(key, value)
}

func (s storeAdapter) Delete(key []byte) {
	s.parent.Delete(key)
}

func (s storeAdapter) Iterator(start, end []byte) wasmvmtypes.Iterator {
	return s.parent.Iterator(start, end)
}

func (s storeAdapter) ReverseIterator(start, end []byte) wasmvmtypes.Iterator {
	return s.parent.ReverseIterator(start, end)
}
