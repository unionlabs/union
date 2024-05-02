package keeper

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
func WithQueryPlugins(plugins *QueryPlugins) Option {
	return optsFn(func(k *Keeper) {
		currentPlugins := k.getQueryPlugins()
		newPlugins := currentPlugins.Merge(plugins)

		k.setQueryPlugins(newPlugins)
	})
}
