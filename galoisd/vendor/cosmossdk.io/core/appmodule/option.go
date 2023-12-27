package appmodule

import (
	"cosmossdk.io/core/internal"
)

// Option is a functional option for implementing modules.
type Option interface {
	apply(*internal.ModuleInitializer) error
}

type funcOption func(initializer *internal.ModuleInitializer) error

func (f funcOption) apply(initializer *internal.ModuleInitializer) error {
	return f(initializer)
}

// Provide registers providers with the dependency injection system that will be
// run within the module scope. See cosmossdk.io/depinject for
// documentation on the dependency injection system.
func Provide(providers ...interface{}) Option {
	return funcOption(func(initializer *internal.ModuleInitializer) error {
		initializer.Providers = append(initializer.Providers, providers...)
		return nil
	})
}

// Invoke registers invokers to run with depinject. Each invoker will be called
// at the end of dependency graph configuration in the order in which it was defined. Invokers may not define output
// parameters, although they may return an error, and all of their input parameters will be marked as optional so that
// invokers impose no additional constraints on the dependency graph. Invoker functions should nil-check all inputs.
func Invoke(invokers ...interface{}) Option {
	return funcOption(func(initializer *internal.ModuleInitializer) error {
		initializer.Invokers = append(initializer.Invokers, invokers...)
		return nil
	})
}
