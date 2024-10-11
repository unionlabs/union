package keeper

import (
	"errors"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

// Option is an extension point to instantiate keeper with non default values
type Option interface {
	apply(*Keeper)
}

type optsFn func(*Keeper)

func (f optsFn) apply(keeper *Keeper) {
	f(keeper)
}

// WithQueryPlugins is an optional constructor parameter to pass custom query plugins for wasmVM requests.
// Missing fields will be filled with default queriers.
func WithQueryPlugins(plugins *types.QueryPlugins) Option {
	return optsFn(func(_ *Keeper) {
		currentPlugins, ok := ibcwasm.GetQueryPlugins().(*types.QueryPlugins)
		if !ok {
			panic(errors.New("invalid query plugins type"))
		}
		newPlugins := currentPlugins.Merge(plugins)
		ibcwasm.SetQueryPlugins(&newPlugins)
	})
}
