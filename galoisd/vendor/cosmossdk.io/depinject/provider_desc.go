package depinject

import (
	"reflect"
	"strings"
	"unicode"

	"github.com/cockroachdb/errors"
	"golang.org/x/exp/slices"
)

// providerDescriptor defines a special provider type that is defined by
// reflection. It should be passed as a value to the Provide function.
// Ex:
//
//	option.Provide(providerDescriptor{ ... })
type providerDescriptor struct {
	// Inputs defines the in parameter types to Fn.
	Inputs []providerInput

	// Outputs defines the out parameter types to Fn.
	Outputs []providerOutput

	// Fn defines the provider function.
	Fn func([]reflect.Value) ([]reflect.Value, error)

	// Location defines the source code location to be used for this provider
	// in error messages.
	Location Location
}

type providerInput struct {
	Type     reflect.Type
	Optional bool
}

type providerOutput struct {
	Type reflect.Type
}

func extractProviderDescriptor(provider interface{}) (providerDescriptor, error) {
	rctr, err := doExtractProviderDescriptor(provider)
	if err != nil {
		return providerDescriptor{}, err
	}
	return postProcessProvider(rctr)
}

func extractInvokerDescriptor(provider interface{}) (providerDescriptor, error) {
	rctr, err := doExtractProviderDescriptor(provider)
	if err != nil {
		return providerDescriptor{}, err
	}

	// mark all inputs as optional
	for i, input := range rctr.Inputs {
		input.Optional = true
		rctr.Inputs[i] = input
	}

	return postProcessProvider(rctr)
}

func doExtractProviderDescriptor(ctr interface{}) (providerDescriptor, error) {
	val := reflect.ValueOf(ctr)
	typ := val.Type()
	if typ.Kind() != reflect.Func {
		return providerDescriptor{}, errors.Errorf("expected a Func type, got %v", typ)
	}

	loc := LocationFromPC(val.Pointer()).(*location)
	nameParts := strings.Split(loc.name, ".")
	if len(nameParts) == 0 {
		return providerDescriptor{}, errors.Errorf("missing function name %s", loc)
	}

	lastNamePart := nameParts[len(nameParts)-1]

	if unicode.IsLower([]rune(lastNamePart)[0]) {
		return providerDescriptor{}, errors.Errorf("function must be exported: %s", loc)
	}

	if strings.Contains(lastNamePart, "-") {
		return providerDescriptor{}, errors.Errorf("function can't be used as a provider (it might be a bound instance method): %s", loc)
	}

	pkgParts := strings.Split(loc.pkg, "/")
	if slices.Contains(pkgParts, "internal") {
		return providerDescriptor{}, errors.Errorf("function must not be in an internal package: %s", loc)
	}

	if typ.IsVariadic() {
		return providerDescriptor{}, errors.Errorf("variadic function can't be used as a provider: %s", loc)
	}

	numIn := typ.NumIn()
	in := make([]providerInput, numIn)
	for i := 0; i < numIn; i++ {
		in[i] = providerInput{
			Type: typ.In(i),
		}
	}

	errIdx := -1
	numOut := typ.NumOut()
	var out []providerOutput
	for i := 0; i < numOut; i++ {
		t := typ.Out(i)
		if t == errType {
			if i != numOut-1 {
				return providerDescriptor{}, errors.Errorf("output error parameter is not last parameter in function %s", loc)
			}
			errIdx = i
		} else {
			out = append(out, providerOutput{Type: t})
		}
	}

	return providerDescriptor{
		Inputs:  in,
		Outputs: out,
		Fn: func(values []reflect.Value) ([]reflect.Value, error) {
			res := val.Call(values)
			if errIdx >= 0 {
				err := res[errIdx]
				if !err.IsZero() {
					return nil, err.Interface().(error)
				}
				return res[0:errIdx], nil
			}
			return res, nil
		},
		Location: loc,
	}, nil
}

var errType = reflect.TypeOf((*error)(nil)).Elem()

func postProcessProvider(descriptor providerDescriptor) (providerDescriptor, error) {
	descriptor, err := expandStructArgsProvider(descriptor)
	if err != nil {
		return providerDescriptor{}, err
	}
	err = checkInputAndOutputTypes(descriptor)
	return descriptor, err
}

func checkInputAndOutputTypes(descriptor providerDescriptor) error {
	for _, input := range descriptor.Inputs {
		err := isExportedType(input.Type)
		if err != nil {
			return err
		}
	}

	for _, output := range descriptor.Outputs {
		err := isExportedType(output.Type)
		if err != nil {
			return err
		}
	}

	return nil
}
