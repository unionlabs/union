package depinject

import (
	"fmt"
	"reflect"

	"github.com/cockroachdb/errors"
)

// ErrMultipleImplicitInterfaceBindings defines an error condition where an attempt was made to implicitly bind
// Interface to a concrete type, but the container was unable to come to a resolution because multiple Matches
// were found.
type ErrMultipleImplicitInterfaceBindings struct {
	Interface reflect.Type
	Matches   []reflect.Type
}

func newErrMultipleImplicitInterfaceBindings(i reflect.Type, matches map[reflect.Type]reflect.Type) ErrMultipleImplicitInterfaceBindings {
	var ms []reflect.Type
	for k := range matches {
		ms = append(ms, k)
	}
	return ErrMultipleImplicitInterfaceBindings{Interface: i, Matches: ms}
}

func (err ErrMultipleImplicitInterfaceBindings) Error() string {
	matchesStr := ""
	for _, m := range err.Matches {
		matchesStr = fmt.Sprintf("%s\n  %s", matchesStr, fullyQualifiedTypeName(m))
	}
	return fmt.Sprintf("Multiple implementations found for interface %v: %s", err.Interface, matchesStr)
}

// ErrNoTypeForExplicitBindingFound defines an error condition where an explicit binding was specified from Interface
// to Implementation but no provider for the requested Implementation was found in the container.
type ErrNoTypeForExplicitBindingFound struct {
	Implementation string
	Interface      string
	ModuleName     string
}

func newErrNoTypeForExplicitBindingFound(p interfaceBinding) ErrNoTypeForExplicitBindingFound {
	var moduleName string
	if p.moduleKey != nil {
		moduleName = p.moduleKey.name
	}

	return ErrNoTypeForExplicitBindingFound{
		Implementation: p.implTypeName,
		Interface:      p.interfaceName,
		ModuleName:     moduleName,
	}
}

func (err ErrNoTypeForExplicitBindingFound) Error() string {
	if err.ModuleName != "" {
		return fmt.Sprintf("No type for explicit binding found.  Given the explicit interface binding %s in module %s, a provider of type %s was not found.",
			err.Interface, err.ModuleName, err.Implementation)
	}

	return fmt.Sprintf("No type for explicit binding found.  Given the explicit interface binding %s, a provider of type %s was not found.",
		err.Interface, err.Implementation)
}

func duplicateDefinitionError(typ reflect.Type, duplicateLoc Location, existingLoc string) error {
	return errors.Errorf("duplicate provision of type %v by %s\n\talready provided by %s",
		typ, duplicateLoc, existingLoc)
}
