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

// Name returns the module key's name.
func (k ModuleKey) Name() string {
	return k.name
}

// Equals checks if the module key is equal to another module key. Module keys
// will be equal only if they have the same name and come from the same
// ModuleKeyContext.
func (k ModuleKey) Equals(other ModuleKey) bool {
	return k.moduleKey == other.moduleKey
}

var moduleKeyType = reflect.TypeOf(ModuleKey{})

// OwnModuleKey is a type which can be used in a module to retrieve its own
// ModuleKey. It MUST NOT be used together with a ModuleKey dependency.
type OwnModuleKey ModuleKey

var ownModuleKeyType = reflect.TypeOf((*OwnModuleKey)(nil)).Elem()

// ModuleKeyContext defines a context for non-forgeable module keys.
// All module keys with the same name from the same context should be equal
// and module keys with the same name but from different contexts should be
// not equal.
//
// Usage:
//
//	moduleKeyCtx := &ModuleKeyContext{}
//	fooKey := moduleKeyCtx.For("foo")
type ModuleKeyContext struct {
	moduleKeys map[string]*moduleKey
}

// For returns a new or existing module key for the given name within the context.
func (c *ModuleKeyContext) For(moduleName string) ModuleKey {
	return ModuleKey{c.createOrGetModuleKey(moduleName)}
}

func (c *ModuleKeyContext) createOrGetModuleKey(moduleName string) *moduleKey {
	if c.moduleKeys == nil {
		c.moduleKeys = map[string]*moduleKey{}
	}

	if k, ok := c.moduleKeys[moduleName]; ok {
		return k
	}

	k := &moduleKey{moduleName}
	c.moduleKeys[moduleName] = k
	return k
}
