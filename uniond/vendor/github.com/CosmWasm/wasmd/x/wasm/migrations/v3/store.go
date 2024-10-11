package v3

import (
	"context"
	"encoding/binary"

	corestoretypes "cosmossdk.io/core/store"
	"cosmossdk.io/store/prefix"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// StoreCodeInfoFn stores code info
type StoreCodeInfoFn func(ctx context.Context, codeID uint64, codeInfo types.CodeInfo)

// Keeper abstract keeper
type wasmKeeper interface {
	SetParams(ctx context.Context, ps types.Params) error
}

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper          wasmKeeper
	storeCodeInfoFn StoreCodeInfoFn
}

// NewMigrator returns a new Migrator.
func NewMigrator(k wasmKeeper, fn StoreCodeInfoFn) Migrator {
	return Migrator{keeper: k, storeCodeInfoFn: fn}
}

// Migrate3to4 migrates from version 3 to 4.
func (m Migrator) Migrate3to4(ctx sdk.Context, storeService corestoretypes.KVStoreService, cdc codec.BinaryCodec) error {
	var legacyParams Params
	store := storeService.OpenKVStore(ctx)
	bz, err := store.Get(types.ParamsKey)
	if err != nil {
		return err
	}
	if bz != nil {
		cdc.MustUnmarshal(bz, &legacyParams)

		newParams := types.Params{}
		newParams.CodeUploadAccess = updateAccessConfig(legacyParams.CodeUploadAccess)

		if legacyParams.InstantiateDefaultPermission == AccessTypeOnlyAddress {
			newParams.InstantiateDefaultPermission = types.AccessTypeAnyOfAddresses
		} else {
			newParams.InstantiateDefaultPermission = types.AccessType(legacyParams.InstantiateDefaultPermission)
		}

		err := m.keeper.SetParams(ctx, newParams)
		if err != nil {
			return err
		}
	}

	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(store), types.CodeKeyPrefix)
	iter := prefixStore.Iterator(nil, nil)
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		var legacyCodeInfo CodeInfo
		cdc.MustUnmarshal(iter.Value(), &legacyCodeInfo)

		newAccessConfig := updateAccessConfig(legacyCodeInfo.InstantiateConfig)

		newCodeInfo := types.CodeInfo{
			CodeHash:          legacyCodeInfo.CodeHash,
			Creator:           legacyCodeInfo.Creator,
			InstantiateConfig: newAccessConfig,
		}

		m.storeCodeInfoFn(ctx, binary.BigEndian.Uint64(iter.Key()), newCodeInfo)
	}
	return nil
}

func updateAccessConfig(legacyAccessConfig AccessConfig) types.AccessConfig {
	newAccessConfig := types.AccessConfig{}

	switch legacyAccessConfig.Permission {
	case AccessTypeOnlyAddress:
		newAccessConfig.Permission = types.AccessTypeAnyOfAddresses
		newAccessConfig.Addresses = []string{legacyAccessConfig.Address}
	default:
		newAccessConfig.Permission = types.AccessType(legacyAccessConfig.Permission)
		newAccessConfig.Addresses = legacyAccessConfig.Addresses
	}
	return newAccessConfig
}
