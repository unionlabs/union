package depinject

import (
	"fmt"
	"reflect"
	"strings"
	"unicode"

	"golang.org/x/exp/slices"
)

// isExportedType checks if the type is exported and not in an internal
// package. NOTE: generic type parameters are not checked because this
// would involve complex parsing of type names (there is no reflect API for
// generic type parameters). Parsing of these parameters should be possible
// if someone chooses to do it in the future, but care should be taken to
// be exhaustive and cover all cases like pointers, map's, chan's, etc. which
// means you actually need a real parser and not just a regex.
func isExportedType(typ reflect.Type) error {
	name := typ.Name()
	pkgPath := typ.PkgPath()
	if name != "" && pkgPath != "" {
		if unicode.IsLower([]rune(name)[0]) {
			return fmt.Errorf("type must be exported: %s", typ)
		}

		pkgParts := strings.Split(pkgPath, "/")
		if slices.Contains(pkgParts, "internal") {
			return fmt.Errorf("type must not come from an internal package: %s", typ)
		}

		return nil
	}

	switch typ.Kind() {
	case reflect.Array, reflect.Slice, reflect.Chan, reflect.Pointer:
		return isExportedType(typ.Elem())

	case reflect.Func:
		numIn := typ.NumIn()
		for i := 0; i < numIn; i++ {
			err := isExportedType(typ.In(i))
			if err != nil {
				return err
			}
		}

		numOut := typ.NumOut()
		for i := 0; i < numOut; i++ {
			err := isExportedType(typ.Out(i))
			if err != nil {
				return err
			}
		}

		return nil

	case reflect.Map:
		err := isExportedType(typ.Key())
		if err != nil {
			return err
		}
		return isExportedType(typ.Elem())

	default:
		// all the remaining types are builtin, non-composite types (like integers), so they are fine to use
		return nil
	}
}
