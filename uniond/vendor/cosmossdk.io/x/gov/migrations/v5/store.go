package v5

import (
	"context"

	"cosmossdk.io/collections"
	corestoretypes "cosmossdk.io/core/store"
	govv1 "cosmossdk.io/x/gov/types/v1"

	"github.com/cosmos/cosmos-sdk/codec"
)

var (
	// ParamsKey is the key of x/gov params
	ParamsKey = []byte{0x30}
	// ConstitutionKey is the key of x/gov constitution
	ConstitutionKey = collections.NewPrefix(49)
)

// MigrateStore performs in-place store migrations from v4 (v0.47) to v5 (v0.50). The
// migration includes:
//
// Addition of the new proposal expedited parameters that are set to 0 by default.
// Set of default chain constitution.
func MigrateStore(ctx context.Context, storeService corestoretypes.KVStoreService, cdc codec.BinaryCodec, constitutionCollection collections.Item[string]) error {
	store := storeService.OpenKVStore(ctx)
	paramsBz, err := store.Get(ParamsKey)
	if err != nil {
		return err
	}

	var params govv1.Params
	err = cdc.Unmarshal(paramsBz, &params)
	if err != nil {
		return err
	}

	defaultParams := govv1.DefaultParams()
	params.ExpeditedMinDeposit = defaultParams.ExpeditedMinDeposit
	params.ExpeditedVotingPeriod = defaultParams.ExpeditedVotingPeriod
	params.ExpeditedThreshold = defaultParams.ExpeditedThreshold
	params.ProposalCancelRatio = defaultParams.ProposalCancelRatio
	params.ProposalCancelDest = defaultParams.ProposalCancelDest
	params.MinDepositRatio = defaultParams.MinDepositRatio

	bz, err := cdc.Marshal(&params)
	if err != nil {
		return err
	}

	if err := store.Set(ParamsKey, bz); err != nil {
		return err
	}

	// Set the default consisitution if it is not set
	if ok, err := constitutionCollection.Has(ctx); !ok || err != nil {
		if err := constitutionCollection.Set(ctx, "This chain has no constitution."); err != nil {
			return err
		}
	}

	return nil
}
