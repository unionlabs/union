package depinject

import (
	"reflect"

	"github.com/cockroachdb/errors"
)

// Config is a functional configuration of a container.
type Config interface {
	apply(*container) error
}

// Provide defines a container configuration which registers the provided dependency
// injection providers. Each provider will be called at most once with the
// exception of module-scoped providers which are called at most once per module
// (see ModuleKey). All provider functions must be declared, exported functions not
// internal packages and all of their input and output types must also be declared
// and exported and not in internal packages. Note that generic type parameters
// will not be checked, but they should also be exported so that codegen is possible.
func Provide(providers ...interface{}) Config {
	return containerConfig(func(ctr *container) error {
		return provide(ctr, nil, providers)
	})
}

// ProvideInModule defines container configuration which registers the provided dependency
// injection providers that are to be run in the named module. Each provider
// will be called at most once. All provider functions must be declared, exported functions not
// internal packages and all of their input and output types must also be declared
// and exported and not in internal packages. Note that generic type parameters
// will not be checked, but they should also be exported so that codegen is possible.
func ProvideInModule(moduleName string, providers ...interface{}) Config {
	return containerConfig(func(ctr *container) error {
		if moduleName == "" {
			return errors.Errorf("expected non-empty module name")
		}

		return provide(ctr, ctr.moduleKeyContext.createOrGetModuleKey(moduleName), providers)
	})
}

func provide(ctr *container, key *moduleKey, providers []interface{}) error {
	for _, c := range providers {
		rc, err := extractProviderDescriptor(c)
		if err != nil {
			return errors.WithStack(err)
		}
		_, err = ctr.addNode(&rc, key)
		if err != nil {
			return errors.WithStack(err)
		}
	}
	return nil
}

// Invoke defines a container configuration which registers the provided invoker functions. Each invoker will be called
// at the end of dependency graph configuration in the order in which it was defined. Invokers may not define output
// parameters, although they may return an error, and all of their input parameters will be marked as optional so that
// invokers impose no additional constraints on the dependency graph. Invoker functions should nil-check all inputs.
// All invoker functions must be declared, exported functions not
// internal packages and all of their input and output types must also be declared
// and exported and not in internal packages. Note that generic type parameters
// will not be checked, but they should also be exported so that codegen is possible.
func Invoke(invokers ...interface{}) Config {
	return containerConfig(func(ctr *container) error {
		return invoke(ctr, nil, invokers)
	})
}

// InvokeInModule defines a container configuration which registers the provided invoker functions to run in the
// provided module scope. Each invoker will be called
// at the end of dependency graph configuration in the order in which it was defined. Invokers may not define output
// parameters, although they may return an error, and all of their input parameters will be marked as optional so that
// invokers impose no additional constraints on the dependency graph. Invoker functions should nil-check all inputs.
// All invoker functions must be declared, exported functions not
// internal packages and all of their input and output types must also be declared
// and exported and not in internal packages. Note that generic type parameters
// will not be checked, but they should also be exported so that codegen is possible.
func InvokeInModule(moduleName string, invokers ...interface{}) Config {
	return containerConfig(func(ctr *container) error {
		if moduleName == "" {
			return errors.Errorf("expected non-empty module name")
		}

		return invoke(ctr, ctr.moduleKeyContext.createOrGetModuleKey(moduleName), invokers)
	})
}

func invoke(ctr *container, key *moduleKey, invokers []interface{}) error {
	for _, c := range invokers {
		rc, err := extractInvokerDescriptor(c)
		if err != nil {
			return errors.WithStack(err)
		}
		err = ctr.addInvoker(&rc, key)
		if err != nil {
			return err
		}
	}
	return nil
}

// BindInterface defines a container configuration for an explicit interface binding of inTypeName to outTypeName
// in global scope.  The example below demonstrates a configuration where the container always provides a Canvasback
// instance when an interface of type Duck is requested as an input.
//
// BindInterface(
//
//	"cosmossdk.io/depinject_test/depinject_test.Duck",
//	"cosmossdk.io/depinject_test/depinject_test.Canvasback")
func BindInterface(inTypeName, outTypeName string) Config {
	return containerConfig(func(ctr *container) error {
		return bindInterface(ctr, inTypeName, outTypeName, "")
	})
}

// BindInterfaceInModule defines a container configuration for an explicit interface binding of inTypeName to outTypeName
// in the scope of the module with name moduleName.  The example below demonstrates a configuration where the container
// provides a Canvasback instance when an interface of type Duck is requested as an input, but only in the scope of
// "moduleFoo".
//
// BindInterfaceInModule(
//
//	 "moduleFoo",
//		"cosmossdk.io/depinject_test/depinject_test.Duck",
//		"cosmossdk.io/depinject_test/depinject_test.Canvasback")
func BindInterfaceInModule(moduleName, inTypeName, outTypeName string) Config {
	return containerConfig(func(ctr *container) error {
		return bindInterface(ctr, inTypeName, outTypeName, moduleName)
	})
}

func bindInterface(ctr *container, inTypeName, outTypeName, moduleName string) error {
	var mk *moduleKey
	if moduleName != "" {
		mk = &moduleKey{name: moduleName}
	}
	ctr.addBinding(interfaceBinding{
		interfaceName: inTypeName,
		implTypeName:  outTypeName,
		moduleKey:     mk,
	})

	return nil
}

func Supply(values ...interface{}) Config {
	loc := LocationFromCaller(1)
	return containerConfig(func(ctr *container) error {
		for _, v := range values {
			err := ctr.supply(reflect.ValueOf(v), loc)
			if err != nil {
				return errors.WithStack(err)
			}
		}
		return nil
	})
}

// Error defines configuration which causes the dependency injection container to
// fail immediately.
func Error(err error) Config {
	return containerConfig(func(*container) error {
		return errors.WithStack(err)
	})
}

// Configs defines a configuration which bundles together multiple Config definitions.
func Configs(opts ...Config) Config {
	return containerConfig(func(ctr *container) error {
		for _, opt := range opts {
			err := opt.apply(ctr)
			if err != nil {
				return errors.WithStack(err)
			}
		}
		return nil
	})
}

type containerConfig func(*container) error

func (c containerConfig) apply(ctr *container) error {
	return c(ctr)
}

var _ Config = (*containerConfig)(nil)
