package keeper

import (
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/params/types"
	"github.com/cosmos/cosmos-sdk/x/params/types/proposal"
)

// Keeper of the global paramstore
type Keeper struct {
	cdc         codec.BinaryCodec
	legacyAmino *codec.LegacyAmino
	key         storetypes.StoreKey
	tkey        storetypes.StoreKey
	spaces      map[string]*types.Subspace
}

// NewKeeper constructs a params keeper
func NewKeeper(cdc codec.BinaryCodec, legacyAmino *codec.LegacyAmino, key, tkey storetypes.StoreKey) Keeper {
	return Keeper{
		cdc:         cdc,
		legacyAmino: legacyAmino,
		key:         key,
		tkey:        tkey,
		spaces:      make(map[string]*types.Subspace),
	}
}

// Logger returns a module-specific logger.
func (k Keeper) Logger(ctx sdk.Context) log.Logger {
	return ctx.Logger().With("module", "x/"+proposal.ModuleName)
}

// Allocate subspace used for keepers
func (k Keeper) Subspace(s string) types.Subspace {
	_, ok := k.spaces[s]
	if ok {
		panic("subspace already occupied")
	}

	if s == "" {
		panic("cannot use empty string for subspace")
	}

	space := types.NewSubspace(k.cdc, k.legacyAmino, k.key, k.tkey, s)
	k.spaces[s] = &space

	return space
}

// Get existing substore from keeper
func (k Keeper) GetSubspace(s string) (types.Subspace, bool) {
	space, ok := k.spaces[s]
	if !ok {
		return types.Subspace{}, false
	}
	return *space, ok
}

// GetSubspaces returns all the registered subspaces.
func (k Keeper) GetSubspaces() []types.Subspace {
	spaces := make([]types.Subspace, len(k.spaces))
	i := 0
	for _, ss := range k.spaces {
		spaces[i] = *ss
		i++
	}

	return spaces
}
