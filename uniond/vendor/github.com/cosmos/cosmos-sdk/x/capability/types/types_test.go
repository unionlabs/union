package types_test

import (
	"fmt"
	"sort"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/x/capability/types"
)

func TestCapabilityKey(t *testing.T) {
	idx := uint64(3162)
	cap := types.NewCapability(idx)
	require.Equal(t, idx, cap.GetIndex())
	require.Equal(t, fmt.Sprintf("Capability{%p, %d}", cap, idx), cap.String())
}

func TestOwner(t *testing.T) {
	o := types.NewOwner("bank", "send")
	require.Equal(t, "bank/send", o.Key())
	require.Equal(t, "module: bank\nname: send\n", o.String())
}

func TestCapabilityOwners_Set(t *testing.T) {
	co := types.NewCapabilityOwners()

	owners := make([]types.Owner, 1024)
	for i := range owners {
		var owner types.Owner

		if i%2 == 0 {
			owner = types.NewOwner("bank", fmt.Sprintf("send-%d", i))
		} else {
			owner = types.NewOwner("slashing", fmt.Sprintf("slash-%d", i))
		}

		owners[i] = owner
		require.NoError(t, co.Set(owner))
	}

	sort.Slice(owners, func(i, j int) bool { return owners[i].Key() < owners[j].Key() })
	require.Equal(t, owners, co.Owners)

	for _, owner := range owners {
		require.Error(t, co.Set(owner))
	}
}

func TestCapabilityOwners_Remove(t *testing.T) {
	co := types.NewCapabilityOwners()

	co.Remove(types.NewOwner("bank", "send-0"))
	require.Len(t, co.Owners, 0)

	for i := 0; i < 5; i++ {
		require.NoError(t, co.Set(types.NewOwner("bank", fmt.Sprintf("send-%d", i))))
	}

	require.Len(t, co.Owners, 5)

	for i := 0; i < 5; i++ {
		co.Remove(types.NewOwner("bank", fmt.Sprintf("send-%d", i)))
		require.Len(t, co.Owners, 5-(i+1))
	}

	require.Len(t, co.Owners, 0)
}
