package keeper

import (
	"context"
	"errors"
	"fmt"
	"strings"

	corestore "cosmossdk.io/core/store"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/log"
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/modules/capability/types"
)

type (
	// Keeper defines the capability module's keeper. It is responsible for provisioning,
	// tracking, and authenticating capabilities at runtime. During application
	// initialization, the keeper can be hooked up to modules through unique function
	// references so that it can identify the calling module when later invoked.
	//
	// When the initial state is loaded from disk, the keeper allows the ability to
	// create new capability keys for all previously allocated capability identifiers
	// (allocated during execution of past transactions and assigned to particular modes),
	// and keep them in a memory-only store while the chain is running.
	//
	// The keeper allows the ability to create scoped sub-keepers which are tied to
	// a single specific module.
	Keeper struct {
		cdc           codec.BinaryCodec
		storeService  corestore.KVStoreService
		memKey        storetypes.StoreKey
		capMap        map[uint64]*types.Capability
		scopedModules map[string]struct{}
		sealed        bool
	}

	// ScopedKeeper defines a scoped sub-keeper which is tied to a single specific
	// module provisioned by the capability keeper. Scoped keepers must be created
	// at application initialization and passed to modules, which can then use them
	// to claim capabilities they receive and retrieve capabilities which they own
	// by name, in addition to creating new capabilities & authenticating capabilities
	// passed by other modules.
	ScopedKeeper struct {
		cdc          codec.BinaryCodec
		storeService corestore.KVStoreService
		memKey       storetypes.StoreKey
		capMap       map[uint64]*types.Capability
		module       string
	}
)

// NewKeeper constructs a new CapabilityKeeper instance and initializes maps
// for capability map and scopedModules map.
func NewKeeper(cdc codec.BinaryCodec, storeService corestore.KVStoreService, memKey storetypes.StoreKey) *Keeper {
	return &Keeper{
		cdc:           cdc,
		storeService:  storeService,
		memKey:        memKey,
		capMap:        make(map[uint64]*types.Capability),
		scopedModules: make(map[string]struct{}),
		sealed:        false,
	}
}

// ScopeToModule attempts to create and return a ScopedKeeper for a given module
// by name. It will panic if the keeper is already sealed or if the module name
// already has a ScopedKeeper.
func (k *Keeper) ScopeToModule(moduleName string) ScopedKeeper {
	if k.sealed {
		panic(errors.New("cannot scope to module via a sealed capability keeper"))
	}
	if strings.TrimSpace(moduleName) == "" {
		panic(errors.New("cannot scope to an empty module name"))
	}

	if _, ok := k.scopedModules[moduleName]; ok {
		panic(fmt.Errorf("cannot create multiple scoped keepers for the same module name: %s", moduleName))
	}

	k.scopedModules[moduleName] = struct{}{}

	return ScopedKeeper{
		cdc:          k.cdc,
		storeService: k.storeService,
		memKey:       k.memKey,
		capMap:       k.capMap,
		module:       moduleName,
	}
}

// Seal seals the keeper to prevent further modules from creating a scoped keeper.
// Seal may be called during app initialization for applications that do not wish to create scoped keepers dynamically.
func (k *Keeper) Seal() {
	if k.sealed {
		panic(errors.New("cannot initialize and seal an already sealed capability keeper"))
	}

	k.sealed = true
}

// IsSealed returns if the keeper is sealed.
func (k *Keeper) IsSealed() bool {
	return k.sealed
}

// InitMemStore will assure that the module store is a memory store (it will panic if it's not)
// and willl initialize it. The function is safe to be called multiple times.
// InitMemStore must be called every time the app starts before the keeper is used (so
// `BeginBlock` or `InitChain` - whichever is first). We need access to the store so we
// can't initialize it in a constructor.
func (k *Keeper) InitMemStore(ctx context.Context) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
	memStore := sdkCtx.KVStore(k.memKey)
	memStoreType := memStore.GetStoreType()

	if memStoreType != storetypes.StoreTypeMemory {
		panic(fmt.Errorf("invalid memory store type; got %s, expected: %s", memStoreType, storetypes.StoreTypeMemory))
	}

	// create context with no block gas meter to ensure we do not consume gas during local initialization logic.
	noGasCtx := sdkCtx.WithBlockGasMeter(storetypes.NewInfiniteGasMeter()).WithGasMeter(storetypes.NewInfiniteGasMeter())

	// check if memory store has not been initialized yet by checking if initialized flag is nil.
	if !k.IsInitialized(noGasCtx) {
		prefixStore := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(noGasCtx)), types.KeyPrefixIndexCapability)
		iterator := storetypes.KVStorePrefixIterator(prefixStore, nil)

		// initialize the in-memory store for all persisted capabilities
		defer iterator.Close()

		for ; iterator.Valid(); iterator.Next() {
			index := types.IndexFromKey(iterator.Key())

			var capOwners types.CapabilityOwners

			k.cdc.MustUnmarshal(iterator.Value(), &capOwners)
			k.InitializeCapability(noGasCtx, index, capOwners)
		}

		// set the initialized flag so we don't rerun initialization logic
		memStore := noGasCtx.KVStore(k.memKey)
		memStore.Set(types.KeyMemInitialized, []byte{1})
	}
}

// IsInitialized returns true if the keeper is properly initialized, and false otherwise.
func (k *Keeper) IsInitialized(ctx context.Context) bool {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
	memStore := sdkCtx.KVStore(k.memKey)
	return memStore.Has(types.KeyMemInitialized)
}

// InitializeIndex sets the index to one (or greater) in InitChain according
// to the GenesisState. It must only be called once.
// It will panic if the provided index is 0, or if the index is already set.
func (k Keeper) InitializeIndex(ctx context.Context, index uint64) error {
	if index == 0 {
		panic(errors.New("SetIndex requires index > 0"))
	}
	latest := k.GetLatestIndex(ctx)
	if latest > 0 {
		panic(errors.New("SetIndex requires index to not be set"))
	}

	// set the global index to the passed index
	store := k.storeService.OpenKVStore(ctx)
	return store.Set(types.KeyIndex, types.IndexToKey(index))
}

// GetLatestIndex returns the latest index of the CapabilityKeeper
func (k Keeper) GetLatestIndex(ctx context.Context) uint64 {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(types.KeyIndex)
	if err != nil {
		panic(err)
	}
	return types.IndexFromKey(bz)
}

// SetOwners set the capability owners to the store
func (k Keeper) SetOwners(ctx context.Context, index uint64, owners types.CapabilityOwners) {
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx)), types.KeyPrefixIndexCapability)
	indexKey := types.IndexToKey(index)

	// set owners in persistent store
	prefixStore.Set(indexKey, k.cdc.MustMarshal(&owners))
}

// GetOwners returns the capability owners with a given index.
func (k Keeper) GetOwners(ctx context.Context, index uint64) (types.CapabilityOwners, bool) {
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx)), types.KeyPrefixIndexCapability)
	indexKey := types.IndexToKey(index)

	// get owners for index from persistent store
	ownerBytes := prefixStore.Get(indexKey)
	if ownerBytes == nil {
		return types.CapabilityOwners{}, false
	}
	var owners types.CapabilityOwners
	k.cdc.MustUnmarshal(ownerBytes, &owners)
	return owners, true
}

// InitializeCapability takes in an index and an owners array. It creates the capability in memory
// and sets the fwd and reverse keys for each owner in the memstore.
// It is used during initialization from genesis.
func (k Keeper) InitializeCapability(ctx context.Context, index uint64, owners types.CapabilityOwners) {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
	memStore := sdkCtx.KVStore(k.memKey)

	capability := types.NewCapability(index)
	for _, owner := range owners.Owners {
		// Set the forward mapping between the module and capability tuple and the
		// capability name in the memKVStore
		memStore.Set(types.FwdCapabilityKey(owner.Module, capability), []byte(owner.Name))

		// Set the reverse mapping between the module and capability name and the
		// index in the in-memory store. Since marshalling and unmarshalling into a store
		// will change memory address of capability, we simply store index as value here
		// and retrieve the in-memory pointer to the capability from our map
		memStore.Set(types.RevCapabilityKey(owner.Module, owner.Name), sdk.Uint64ToBigEndian(index))

		// Set the mapping from index from index to in-memory capability in the go map
		k.capMap[index] = capability
	}
}

// NewCapability attempts to create a new capability with a given name. If the
// capability already exists in the in-memory store, an error will be returned.
// Otherwise, a new capability is created with the current global unique index.
// The newly created capability has the scoped module name and capability name
// tuple set as the initial owner. Finally, the global index is incremented along
// with forward and reverse indexes set in the in-memory store.
//
// Note, namespacing is completely local, which is safe since records are prefixed
// with the module name and no two ScopedKeeper can have the same module name.
func (sk ScopedKeeper) NewCapability(ctx context.Context, name string) (*types.Capability, error) {
	if strings.TrimSpace(name) == "" {
		return nil, errorsmod.Wrap(types.ErrInvalidCapabilityName, "capability name cannot be empty")
	}
	store := sk.storeService.OpenKVStore(ctx)

	if _, ok := sk.GetCapability(ctx, name); ok {
		return nil, errorsmod.Wrapf(types.ErrCapabilityTaken, fmt.Sprintf("module: %s, name: %s", sk.module, name))
	}

	// create new capability with the current global index
	bz, err := store.Get(types.KeyIndex)
	if err != nil {
		panic(err)
	}
	index := types.IndexFromKey(bz)
	capability := types.NewCapability(index)

	// update capability owner set
	if err := sk.addOwner(ctx, capability, name); err != nil {
		return nil, err
	}

	// increment global index
	store.Set(types.KeyIndex, types.IndexToKey(index+1))

	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
	memStore := sdkCtx.KVStore(sk.memKey)

	// Set the forward mapping between the module and capability tuple and the
	// capability name in the memKVStore
	memStore.Set(types.FwdCapabilityKey(sk.module, capability), []byte(name))

	// Set the reverse mapping between the module and capability name and the
	// index in the in-memory store. Since marshalling and unmarshalling into a store
	// will change memory address of capability, we simply store index as value here
	// and retrieve the in-memory pointer to the capability from our map
	memStore.Set(types.RevCapabilityKey(sk.module, name), sdk.Uint64ToBigEndian(index))

	// Set the mapping from index from index to in-memory capability in the go map
	sk.capMap[index] = capability

	logger(ctx).Info("created new capability", "module", sk.module, "name", name)

	return capability, nil
}

// AuthenticateCapability attempts to authenticate a given capability and name
// from a caller. It allows for a caller to check that a capability does in fact
// correspond to a particular name. The scoped keeper will lookup the capability
// from the internal in-memory store and check against the provided name. It returns
// true upon success and false upon failure.
//
// Note, the capability's forward mapping is indexed by a string which should
// contain its unique memory reference.
func (sk ScopedKeeper) AuthenticateCapability(ctx context.Context, cap *types.Capability, name string) bool {
	if strings.TrimSpace(name) == "" || cap == nil {
		return false
	}
	return sk.GetCapabilityName(ctx, cap) == name
}

// ClaimCapability attempts to claim a given Capability. The provided name and
// the scoped module's name tuple are treated as the owner. It will attempt
// to add the owner to the persistent set of capability owners for the capability
// index. If the owner already exists, it will return an error. Otherwise, it will
// also set a forward and reverse index for the capability and capability name.
func (sk ScopedKeeper) ClaimCapability(ctx context.Context, cap *types.Capability, name string) error {
	if cap == nil {
		return errorsmod.Wrap(types.ErrNilCapability, "cannot claim nil capability")
	}
	if strings.TrimSpace(name) == "" {
		return errorsmod.Wrap(types.ErrInvalidCapabilityName, "capability name cannot be empty")
	}
	// update capability owner set
	if err := sk.addOwner(ctx, cap, name); err != nil {
		return err
	}

	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
	memStore := sdkCtx.KVStore(sk.memKey)

	// Set the forward mapping between the module and capability tuple and the
	// capability name in the memKVStore
	memStore.Set(types.FwdCapabilityKey(sk.module, cap), []byte(name))

	// Set the reverse mapping between the module and capability name and the
	// index in the in-memory store. Since marshalling and unmarshalling into a store
	// will change memory address of capability, we simply store index as value here
	// and retrieve the in-memory pointer to the capability from our map
	memStore.Set(types.RevCapabilityKey(sk.module, name), sdk.Uint64ToBigEndian(cap.GetIndex()))

	logger(ctx).Info("claimed capability", "module", sk.module, "name", name, "capability", cap.GetIndex())

	return nil
}

// ReleaseCapability allows a scoped module to release a capability which it had
// previously claimed or created. After releasing the capability, if no more
// owners exist, the capability will be globally removed.
func (sk ScopedKeeper) ReleaseCapability(ctx context.Context, cap *types.Capability) error {
	if cap == nil {
		return errorsmod.Wrap(types.ErrNilCapability, "cannot release nil capability")
	}
	name := sk.GetCapabilityName(ctx, cap)
	if len(name) == 0 {
		return errorsmod.Wrap(types.ErrCapabilityNotOwned, sk.module)
	}

	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
	memStore := sdkCtx.KVStore(sk.memKey)

	// Delete the forward mapping between the module and capability tuple and the
	// capability name in the memKVStore
	memStore.Delete(types.FwdCapabilityKey(sk.module, cap))

	// Delete the reverse mapping between the module and capability name and the
	// index in the in-memory store.
	memStore.Delete(types.RevCapabilityKey(sk.module, name))

	// remove owner
	capOwners := sk.getOwners(ctx, cap)
	capOwners.Remove(types.NewOwner(sk.module, name))

	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(sk.storeService.OpenKVStore(ctx)), types.KeyPrefixIndexCapability)
	indexKey := types.IndexToKey(cap.GetIndex())

	if len(capOwners.Owners) == 0 {
		// remove capability owner set
		prefixStore.Delete(indexKey)
		// since no one owns capability, we can delete capability from map
		delete(sk.capMap, cap.GetIndex())
	} else {
		// update capability owner set
		prefixStore.Set(indexKey, sk.cdc.MustMarshal(capOwners))
	}

	return nil
}

// GetCapability allows a module to fetch a capability which it previously claimed
// by name. The module is not allowed to retrieve capabilities which it does not
// own.
func (sk ScopedKeeper) GetCapability(ctx context.Context, name string) (*types.Capability, bool) {
	if strings.TrimSpace(name) == "" {
		return nil, false
	}
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
	memStore := sdkCtx.KVStore(sk.memKey)

	key := types.RevCapabilityKey(sk.module, name)
	indexBytes := memStore.Get(key)
	index := sdk.BigEndianToUint64(indexBytes)

	if len(indexBytes) == 0 {
		// If a tx failed and NewCapability got reverted, it is possible
		// to still have the capability in the go map since changes to
		// go map do not automatically get reverted on tx failure,
		// so we delete here to remove unnecessary values in map
		// TODO: Delete index correctly from capMap by storing some reverse lookup
		// in-memory map. Issue: https://github.com/cosmos/cosmos-sdk/issues/7805

		return nil, false
	}

	capability := sk.capMap[index]
	if capability == nil {
		panic(errors.New("capability found in memstore is missing from map"))
	}

	return capability, true
}

// GetCapabilityName allows a module to retrieve the name under which it stored a given
// capability given the capability
func (sk ScopedKeeper) GetCapabilityName(ctx context.Context, cap *types.Capability) string {
	if cap == nil {
		return ""
	}
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
	memStore := sdkCtx.KVStore(sk.memKey)

	return string(memStore.Get(types.FwdCapabilityKey(sk.module, cap)))
}

// GetOwners all the Owners that own the capability associated with the name this ScopedKeeper uses
// to refer to the capability
func (sk ScopedKeeper) GetOwners(ctx context.Context, name string) (*types.CapabilityOwners, bool) {
	if strings.TrimSpace(name) == "" {
		return nil, false
	}
	capability, ok := sk.GetCapability(ctx, name)
	if !ok {
		return nil, false
	}

	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(sk.storeService.OpenKVStore(ctx)), types.KeyPrefixIndexCapability)
	indexKey := types.IndexToKey(capability.GetIndex())

	var capOwners types.CapabilityOwners

	bz := prefixStore.Get(indexKey)
	if len(bz) == 0 {
		return nil, false
	}

	sk.cdc.MustUnmarshal(bz, &capOwners)

	return &capOwners, true
}

// LookupModules returns all the module owners for a given capability
// as a string array and the capability itself.
// The method returns an error if either the capability or the owners cannot be
// retreived from the memstore.
func (sk ScopedKeeper) LookupModules(ctx context.Context, name string) ([]string, *types.Capability, error) {
	if strings.TrimSpace(name) == "" {
		return nil, nil, errorsmod.Wrap(types.ErrInvalidCapabilityName, "cannot lookup modules with empty capability name")
	}
	capability, ok := sk.GetCapability(ctx, name)
	if !ok {
		return nil, nil, errorsmod.Wrap(types.ErrCapabilityNotFound, name)
	}

	capOwners, ok := sk.GetOwners(ctx, name)
	if !ok {
		return nil, nil, errorsmod.Wrap(types.ErrCapabilityOwnersNotFound, name)
	}

	mods := make([]string, len(capOwners.Owners))
	for i, co := range capOwners.Owners {
		mods[i] = co.Module
	}

	return mods, capability, nil
}

func (sk ScopedKeeper) addOwner(ctx context.Context, cap *types.Capability, name string) error {
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(sk.storeService.OpenKVStore(ctx)), types.KeyPrefixIndexCapability)
	indexKey := types.IndexToKey(cap.GetIndex())

	capOwners := sk.getOwners(ctx, cap)

	if err := capOwners.Set(types.NewOwner(sk.module, name)); err != nil {
		return err
	}

	// update capability owner set
	prefixStore.Set(indexKey, sk.cdc.MustMarshal(capOwners))

	return nil
}

func (sk ScopedKeeper) getOwners(ctx context.Context, cap *types.Capability) *types.CapabilityOwners {
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(sk.storeService.OpenKVStore(ctx)), types.KeyPrefixIndexCapability)
	indexKey := types.IndexToKey(cap.GetIndex())

	bz := prefixStore.Get(indexKey)

	if len(bz) == 0 {
		return types.NewCapabilityOwners()
	}

	var capOwners types.CapabilityOwners
	sk.cdc.MustUnmarshal(bz, &capOwners)
	return &capOwners
}

func logger(ctx context.Context) log.Logger {
	sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
	return sdkCtx.Logger().With("module", "x/"+types.ModuleName)
}
