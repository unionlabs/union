package v4

import (
	"fmt"
	"sort"

	corestoretypes "cosmossdk.io/core/store"
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/gov/exported"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/migrations/v1"
	"github.com/cosmos/cosmos-sdk/x/gov/types"
	govv1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
)

func migrateParams(ctx sdk.Context, store storetypes.KVStore, legacySubspace exported.ParamSubspace, cdc codec.BinaryCodec) error {
	dp := govv1.DepositParams{}
	vp := govv1.VotingParams{}
	tp := govv1.TallyParams{}
	legacySubspace.Get(ctx, govv1.ParamStoreKeyDepositParams, &dp)
	legacySubspace.Get(ctx, govv1.ParamStoreKeyVotingParams, &vp)
	legacySubspace.Get(ctx, govv1.ParamStoreKeyTallyParams, &tp)

	defaultParams := govv1.DefaultParams()
	params := govv1.NewParams(
		dp.MinDeposit,
		defaultParams.ExpeditedMinDeposit,
		*dp.MaxDepositPeriod,
		*vp.VotingPeriod,
		*defaultParams.ExpeditedVotingPeriod,
		tp.Quorum,
		tp.Threshold,
		defaultParams.ExpeditedThreshold,
		tp.VetoThreshold,
		defaultParams.MinInitialDepositRatio,
		defaultParams.ProposalCancelRatio,
		defaultParams.ProposalCancelDest,
		defaultParams.BurnProposalDepositPrevote,
		defaultParams.BurnVoteQuorum,
		defaultParams.BurnVoteVeto,
		defaultParams.MinDepositRatio,
	)

	bz, err := cdc.Marshal(&params)
	if err != nil {
		return err
	}

	store.Set(ParamsKey, bz)

	return nil
}

func migrateProposalVotingPeriod(ctx sdk.Context, store storetypes.KVStore, cdc codec.BinaryCodec) error {
	propStore := prefix.NewStore(store, v1.ProposalsKeyPrefix)

	iter := propStore.Iterator(nil, nil)
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		var prop govv1.Proposal
		err := cdc.Unmarshal(iter.Value(), &prop)
		if err != nil {
			return err
		}

		if prop.Status == govv1.StatusVotingPeriod {
			store.Set(VotingPeriodProposalKey(prop.Id), []byte{1})
		}
	}

	return nil
}

// MigrateStore performs in-place store migrations from v3 (v0.46) to v4 (v0.47). The
// migration includes:
//
// Params migrations from x/params to gov
// Addition of the new min initial deposit ratio parameter that is set to 0 by default.
// Proposals in voting period are tracked in a separate index.
func MigrateStore(ctx sdk.Context, storeService corestoretypes.KVStoreService, legacySubspace exported.ParamSubspace, cdc codec.BinaryCodec) error {
	store := runtime.KVStoreAdapter(storeService.OpenKVStore(ctx))
	if err := migrateProposalVotingPeriod(ctx, store, cdc); err != nil {
		return err
	}

	return migrateParams(ctx, store, legacySubspace, cdc)
}

// AddProposerAddressToProposal will add proposer to proposal and set to the store. This function is optional.
func AddProposerAddressToProposal(ctx sdk.Context, storeService corestoretypes.KVStoreService, cdc codec.BinaryCodec, proposals map[uint64]string) error {
	proposalIDs := make([]uint64, 0, len(proposals))

	for proposalID := range proposals {
		proposalIDs = append(proposalIDs, proposalID)
	}

	// sort the proposalIDs
	sort.Slice(proposalIDs, func(i, j int) bool { return proposalIDs[i] < proposalIDs[j] })

	store := runtime.KVStoreAdapter(storeService.OpenKVStore(ctx))

	for _, proposalID := range proposalIDs {
		if len(proposals[proposalID]) == 0 {
			return fmt.Errorf("found missing proposer for proposal ID: %d", proposalID)
		}

		if _, err := sdk.AccAddressFromBech32(proposals[proposalID]); err != nil {
			return fmt.Errorf("invalid proposer address : %s", proposals[proposalID])
		}

		bz := store.Get(append(types.ProposalsKeyPrefix, sdk.Uint64ToBigEndian(proposalID)...))
		var proposal govv1.Proposal
		if err := cdc.Unmarshal(bz, &proposal); err != nil {
			panic(err)
		}

		// Check if proposal is active
		if proposal.Status != govv1.ProposalStatus_PROPOSAL_STATUS_VOTING_PERIOD &&
			proposal.Status != govv1.ProposalStatus_PROPOSAL_STATUS_DEPOSIT_PERIOD {
			return fmt.Errorf("invalid proposal : %s, proposal not active", proposals[proposalID])
		}

		proposal.Proposer = proposals[proposalID]

		// set the new proposal with proposer
		bz, err := cdc.Marshal(&proposal)
		if err != nil {
			panic(err)
		}
		store.Set(append(types.ProposalsKeyPrefix, sdk.Uint64ToBigEndian(proposalID)...), bz)
	}

	return nil
}
