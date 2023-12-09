package baseapp

import (
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

type state struct {
	ms  storetypes.CacheMultiStore
	ctx sdk.Context
}

// CacheMultiStore calls and returns a CacheMultiStore on the state's underling
// CacheMultiStore.
func (st *state) CacheMultiStore() storetypes.CacheMultiStore {
	return st.ms.CacheMultiStore()
}

// Context returns the Context of the state.
func (st *state) Context() sdk.Context {
	return st.ctx
}
