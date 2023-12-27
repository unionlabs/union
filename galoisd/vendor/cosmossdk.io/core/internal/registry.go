package internal

import (
	"fmt"
	"reflect"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"

	appv1alpha1 "cosmossdk.io/api/cosmos/app/v1alpha1"
)

// ModuleRegistry is the registry of module initializers indexed by their golang
// type to avoid any issues with protobuf descriptor initialization.
var ModuleRegistry = map[reflect.Type]*ModuleInitializer{}

// ModuleInitializer describes how to initialize a module.
type ModuleInitializer struct {
	ConfigGoType       reflect.Type
	ConfigProtoMessage proto.Message
	Error              error
	Providers          []interface{}
	Invokers           []interface{}
}

// ModulesByProtoMessageName should be used to retrieve modules by their protobuf name.
// This is done lazily after module registration to deal with non-deterministic issues
// that can occur with respect to protobuf descriptor initialization.
func ModulesByProtoMessageName() (map[protoreflect.FullName]*ModuleInitializer, error) {
	res := map[protoreflect.FullName]*ModuleInitializer{}

	for _, initializer := range ModuleRegistry {
		descriptor := initializer.ConfigProtoMessage.ProtoReflect().Descriptor()
		fullName := descriptor.FullName()
		if _, ok := res[fullName]; ok {
			return nil, fmt.Errorf("duplicate module registratio for %s", fullName)
		}

		modDesc := proto.GetExtension(descriptor.Options(), appv1alpha1.E_Module).(*appv1alpha1.ModuleDescriptor)
		if modDesc == nil {
			return nil, fmt.Errorf(
				"protobuf type %s registered as a module should have the option %s",
				fullName,
				appv1alpha1.E_Module.TypeDescriptor().FullName())
		}

		if modDesc.GoImport == "" {
			return nil, fmt.Errorf(
				"protobuf type %s registered as a module should have ModuleDescriptor.go_import specified",
				fullName,
			)
		}

		res[fullName] = initializer
	}

	return res, nil
}
