package depinject

import (
	"reflect"
)

// ModuleKey is a special type used to scope a provider to a "module".
//
// Special module-scoped providers can be used with Provide and ProvideInModule
// by declaring a provider with an input parameter of type ModuleKey. These
// providers may construct a unique value of a dependency for each module and
// will be called at most once per module.
//
// When being used with ProvideInModule, the provider will not receive its
// own ModuleKey but rather the key of the module requesting the dependency
// so that modules can provide module-scoped dependencies to other modules.
//
// In order for a module to retrieve their own module key they can define
// a provider which requires the OwnModuleKey type and DOES NOT require ModuleKey.
type ModuleKey struct {
	*moduleKey
}

type moduleKey struct {
	name string
}

func (k ModuleKey) Name() string {
	return k.name
}

var moduleKeyType = reflect.TypeOf(ModuleKey{})

// OwnModuleKey is a type which can be used in a module to retrieve its own
// ModuleKey. It MUST NOT be used together with a ModuleKey dependency.
type OwnModuleKey ModuleKey

var ownModuleKeyType = reflect.TypeOf((*OwnModuleKey)(nil)).Elem()
