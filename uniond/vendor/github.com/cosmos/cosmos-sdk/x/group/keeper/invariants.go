package keeper

import (
	"fmt"
	"math"
	"sort"

	"golang.org/x/exp/maps"

	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/group"
	"github.com/cosmos/cosmos-sdk/x/group/errors"
	groupmath "github.com/cosmos/cosmos-sdk/x/group/internal/math"
	"github.com/cosmos/cosmos-sdk/x/group/internal/orm"
)

const weightInvariant = "Group-TotalWeight"

// RegisterInvariants registers all group invariants.
func RegisterInvariants(ir sdk.InvariantRegistry, keeper Keeper) {
	ir.RegisterRoute(group.ModuleName, weightInvariant, GroupTotalWeightInvariant(keeper))
}

// GroupTotalWeightInvariant checks that group's TotalWeight must be equal to the sum of its members.
func GroupTotalWeightInvariant(keeper Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		msg, broken := GroupTotalWeightInvariantHelper(ctx, keeper.key, keeper.groupTable, keeper.groupMemberByGroupIndex)
		return sdk.FormatInvariant(group.ModuleName, weightInvariant, msg), broken
	}
}

func GroupTotalWeightInvariantHelper(ctx sdk.Context, key storetypes.StoreKey, groupTable orm.AutoUInt64Table, groupMemberByGroupIndex orm.Index) (string, bool) {
	var msg string
	var broken bool

	groupIt, err := groupTable.PrefixScan(ctx.KVStore(key), 1, math.MaxUint64)
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

	groupByIDs := maps.Keys(groups)
	sort.Slice(groupByIDs, func(i, j int) bool {
		return groupByIDs[i] < groupByIDs[j]
	})
	for _, groupID := range groupByIDs {
		groupInfo := groups[groupID]
		membersWeight, err := groupmath.NewNonNegativeDecFromString("0")
		if err != nil {
			msg += fmt.Sprintf("error while parsing positive dec zero for group member\n%v\n", err)
			return msg, broken
		}

		memIt, err := groupMemberByGroupIndex.Get(ctx.KVStore(key), groupInfo.Id)
		if err != nil {
			msg += fmt.Sprintf("error while returning group member iterator for group with ID %d\n%v\n", groupInfo.Id, err)
			return msg, broken
		}
		defer memIt.Close()

		for {
			var groupMember group.GroupMember
			_, err = memIt.LoadNext(&groupMember)
			if errors.ErrORMIteratorDone.Is(err) {
				break
			}
			if err != nil {
				msg += fmt.Sprintf("LoadNext failure on member table iterator\n%v\n", err)
				return msg, broken
			}

			curMemWeight, err := groupmath.NewPositiveDecFromString(groupMember.GetMember().GetWeight())
			if err != nil {
				msg += fmt.Sprintf("error while parsing non-nengative decimal for group member %s\n%v\n", groupMember.Member.Address, err)
				return msg, broken
			}

			membersWeight, err = groupmath.Add(membersWeight, curMemWeight)
			if err != nil {
				msg += fmt.Sprintf("decimal addition error while adding group member voting weight to total voting weight\n%v\n", err)
				return msg, broken
			}
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
