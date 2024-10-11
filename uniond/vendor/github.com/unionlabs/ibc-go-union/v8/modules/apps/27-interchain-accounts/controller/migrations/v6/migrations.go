package v6

import (
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitykeeper "github.com/cosmos/ibc-go/modules/capability/keeper"
	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	controllertypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
	coretypes "github.com/cosmos/ibc-go/v8/modules/core/types"
)

// MigrateICS27ChannelCapability performs a search on a prefix store using the provided store key and module name.
// It retrieves the associated channel capability index and reassigns ownership to the ICS27 controller submodule.
func MigrateICS27ChannelCapability(
	ctx sdk.Context,
	cdc codec.BinaryCodec,
	capabilityStoreKey storetypes.StoreKey,
	capabilityKeeper *capabilitykeeper.Keeper,
	module string, // the name of the scoped keeper for the underlying app module
) error {
	// construct a prefix store using the x/capability index prefix: index->capability owners
	prefixStore := prefix.NewStore(ctx.KVStore(capabilityStoreKey), capabilitytypes.KeyPrefixIndexCapability)
	iterator := storetypes.KVStorePrefixIterator(prefixStore, nil)
	defer coretypes.LogDeferred(ctx.Logger(), func() error { return iterator.Close() })

	for ; iterator.Valid(); iterator.Next() {
		// unmarshal the capability index value and set of owners
		index := capabilitytypes.IndexFromKey(iterator.Key())

		var owners capabilitytypes.CapabilityOwners
		cdc.MustUnmarshal(iterator.Value(), &owners)

		if !hasIBCOwner(owners.GetOwners()) {
			continue
		}

		for _, owner := range owners.GetOwners() {
			if owner.Module == module {
				// remove the owner from the set
				owners.Remove(owner)

				// reassign the owner module to icacontroller
				owner.Module = controllertypes.SubModuleName

				// add the controller submodule to the set of owners
				if err := owners.Set(owner); err != nil {
					return err
				}

				// set the new owners for the current capability index
				capabilityKeeper.SetOwners(ctx, index, owners)
			}
		}
	}

	// initialise the x/capability memstore
	capabilityKeeper.InitMemStore(ctx)

	return nil
}

func hasIBCOwner(owners []capabilitytypes.Owner) bool {
	if len(owners) != 2 {
		return false
	}

	for _, owner := range owners {
		if owner.Module == ibcexported.ModuleName {
			return true
		}
	}

	return false
}
