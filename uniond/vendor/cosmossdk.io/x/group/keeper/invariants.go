package keeper

import (
	"fmt"
	"maps"
	"math"
	"slices"

	storetypes "cosmossdk.io/core/store"
	"cosmossdk.io/x/group"
	"cosmossdk.io/x/group/errors"
	groupmath "cosmossdk.io/x/group/internal/math"
	"cosmossdk.io/x/group/internal/orm"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

const weightInvariant = "Group-TotalWeight"

// RegisterInvariants registers all group invariants.
func RegisterInvariants(ir sdk.InvariantRegistry, keeper Keeper) {
	ir.RegisterRoute(group.ModuleName, weightInvariant, GroupTotalWeightInvariant(keeper))
}

// GroupTotalWeightInvariant checks that group's TotalWeight must be equal to the sum of its members.
func GroupTotalWeightInvariant(keeper Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		msg, broken := GroupTotalWeightInvariantHelper(ctx, keeper.KVStoreService, keeper.groupTable, keeper.groupMemberByGroupIndex)
		return sdk.FormatInvariant(group.ModuleName, weightInvariant, msg), broken
	}
}

func GroupTotalWeightInvariantHelper(ctx sdk.Context, storeService storetypes.KVStoreService, groupTable orm.AutoUInt64Table, groupMemberByGroupIndex orm.Index) (string, bool) {
	var msg string
	var broken bool

	kvStore := storeService.OpenKVStore(ctx)

	groupIt, err := groupTable.PrefixScan(kvStore, 1, math.MaxUint64)
	if err != nil {
		msg += fmt.Sprintf("PrefixScan failure on group table\n%v\n", err)
		return msg, broken
	}
	defer groupIt.Close()

	groups := make(map[uint64]group.GroupInfo)
	for {
		var groupInfo group.GroupInfo
		_, err = groupIt.LoadNext(&groupInfo)
		if errors.ErrORMIteratorDone.Is(err) {
			break
		}
		if err != nil {
			msg += fmt.Sprintf("LoadNext failure on group table iterator\n%v\n", err)
			return msg, broken
		}
		groups[groupInfo.Id] = groupInfo
	}

	groupByIDs := slices.Collect(maps.Keys(groups))
	slices.SortFunc(groupByIDs, func(i, j uint64) int {
		if groupByIDs[i] < groupByIDs[j] {
			return -1
		} else if groupByIDs[i] > groupByIDs[j] {
			return 1
		}
		return 0
	})

	for _, groupID := range groupByIDs {
		groupInfo := groups[groupID]
		membersWeight, err := groupmath.NewNonNegativeDecFromString("0")
		if err != nil {
			msg += fmt.Sprintf("error while parsing positive dec zero for group member\n%v\n", err)
			return msg, broken
		}

		err = func() error {
			memIt, err := groupMemberByGroupIndex.Get(kvStore, groupInfo.Id)
			if err != nil {
				return fmt.Errorf("error while returning group member iterator for group with ID %d\n%w", groupInfo.Id, err)
			}
			defer memIt.Close()

			for {
				var groupMember group.GroupMember
				_, err = memIt.LoadNext(&groupMember)
				if errors.ErrORMIteratorDone.Is(err) {
					break
				}
				if err != nil {
					return fmt.Errorf("LoadNext failure on member table iterator\n%w", err)
				}

				curMemWeight, err := groupmath.NewPositiveDecFromString(groupMember.GetMember().GetWeight())
				if err != nil {
					return fmt.Errorf("error while parsing non-nengative decimal for group member %s\n%w", groupMember.Member.Address, err)
				}

				membersWeight, err = groupmath.Add(membersWeight, curMemWeight)
				if err != nil {
					return fmt.Errorf("decimal addition error while adding group member voting weight to total voting weight\n%w", err)
				}
			}
			return nil
		}()
		if err != nil {
			msg += err.Error() + "\n"
			return msg, broken
		}

		groupWeight, err := groupmath.NewNonNegativeDecFromString(groupInfo.GetTotalWeight())
		if err != nil {
			msg += fmt.Sprintf("error while parsing non-nengative decimal for group with ID %d\n%v\n", groupInfo.Id, err)
			return msg, broken
		}

		if groupWeight.Cmp(membersWeight) != 0 {
			broken = true
			msg += fmt.Sprintf("group's TotalWeight must be equal to the sum of its members' weights\ngroup weight: %s\nSum of group members weights: %s\n", groupWeight.String(), membersWeight.String())
			break
		}
	}

	return msg, broken
}
