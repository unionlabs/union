package appconfig

import (
	"fmt"
	"reflect"

	"github.com/cosmos/gogoproto/proto"

	internal "cosmossdk.io/depinject/internal/appconfig"
)

var Register = RegisterModule

// RegisterModule registers a module with the global module registry. The provided
// protobuf message is used only to uniquely identify the protobuf module config
// type. The instance of the protobuf message used in the actual configuration
// will be injected into the container and can be requested by a provider
// function. All module initialization should be handled by the provided options.
//
// Protobuf message types used for module configuration should define the
// cosmos.app.v1alpha.module option and must explicitly specify go_package
// to make debugging easier for users.
func RegisterModule(config any, options ...Option) {
	protoConfig, ok := config.(proto.Message)
	if !ok {
		panic(fmt.Errorf("expected config to be a proto.Message, got %T", config))
	}

	ty := reflect.TypeOf(config)
	init := &internal.ModuleInitializer{
		ConfigProtoMessage: protoConfig,
		ConfigGoType:       ty,
	}
	internal.ModuleRegistry[ty] = init

	for _, option := range options {
		init.Error = option.apply(init)
		if init.Error != nil {
			return
		}
	}
}

// Option is a functional option for implementing modules.
type Option interface {
	apply(*internal.ModuleInitializer) error
}

type funcOption func(initializer *internal.ModuleInitializer) error

func (f funcOption) apply(initializer *internal.ModuleInitializer) error {
	return f(initializer)
}

// Provide registers providers with the dependency injection system that will be
// run within the module scope (depinject.ProvideInModule). See cosmossdk.io/depinject for
// documentation on the dependency injection system.
func Provide(providers ...interface{}) Option {
	return funcOption(func(initializer *internal.ModuleInitializer) error {
		initializer.Providers = append(initializer.Providers, providers...)
		return nil
	})
}

// Invoke registers invokers to run with depinject (depinject.InvokeInModule). Each invoker will be called
// at the end of dependency graph configuration in the order in which it was defined. Invokers may not define output
// parameters, although they may return an error, and all of their input parameters will be marked as optional so that
// invokers impose no additional constraints on the dependency graph. Invoker functions should nil-check all inputs.
func Invoke(invokers ...interface{}) Option {
	return funcOption(func(initializer *internal.ModuleInitializer) error {
		initializer.Invokers = append(initializer.Invokers, invokers...)
		return nil
	})
}
