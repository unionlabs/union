package internal

import (
	"fmt"
	"reflect"

	gogoproto "github.com/cosmos/gogoproto/proto"
	"github.com/cosmos/gogoproto/protoc-gen-gogo/descriptor"
	"google.golang.org/protobuf/encoding/protowire"
	protov2 "google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protodesc"
	"google.golang.org/protobuf/reflect/protoreflect"

	"cosmossdk.io/depinject/appconfig/v1alpha1"
)

// ModuleRegistry is the registry of module initializers indexed by their golang
// type to avoid any issues with protobuf descriptor initialization.
var ModuleRegistry = map[reflect.Type]*ModuleInitializer{}

// ModuleInitializer describes how to initialize a module.
type ModuleInitializer struct {
	ConfigGoType       reflect.Type
	ConfigProtoMessage gogoproto.Message
	Error              error
	Providers          []interface{}
	Invokers           []interface{}
}

// ModulesByModuleTypeName should be used to retrieve modules by their module type name.
// This is done lazily after module registration to deal with non-deterministic issues
// that can occur with respect to protobuf descriptor initialization.
func ModulesByModuleTypeName() (map[string]*ModuleInitializer, error) {
	res := map[string]*ModuleInitializer{}

	for _, initializer := range ModuleRegistry {
		// as of gogoproto v1.5.0 this should work with either gogoproto or golang v2 proto
		fullName := gogoproto.MessageName(initializer.ConfigProtoMessage)

		if desc, err := gogoproto.HybridResolver.FindDescriptorByName(protoreflect.FullName(fullName)); err == nil {
			modDesc, err := GetModuleDescriptor(desc)
			if err != nil {
				return nil, err
			}

			if modDesc == nil {
				return nil, fmt.Errorf(
					"protobuf type %s registered as a module should have the option %s",
					desc.FullName(),
					v1alpha1.E_Module.Name)
			}

			if modDesc.GoImport == "" {
				return nil, fmt.Errorf(
					"protobuf type %s registered as a module should have ModuleDescriptor.go_import specified",
					fullName,
				)
			}
		}

		if _, ok := res[fullName]; ok {
			return nil, fmt.Errorf("duplicate module registration for %s", fullName)
		}

		res[fullName] = initializer
	}

	return res, nil
}

// GetModuleDescriptor returns the cosmos.app.v1alpha1.ModuleDescriptor or nil if one isn't found.
// Errors are returned in unexpected cases.
func GetModuleDescriptor(desc protoreflect.Descriptor) (*v1alpha1.ModuleDescriptor, error) {
	// we need to take a somewhat round about way to get the extension here
	// our most complete type registry has a mix of gogoproto and protoreflect types
	// so we start with a protoreflect descriptor, convert it to a gogo descriptor
	// and then get the extension by its raw field value to avoid any unmarshaling errors

	rawV2Desc := protodesc.ToDescriptorProto(desc.(protoreflect.MessageDescriptor))
	bz, err := protov2.Marshal(rawV2Desc)
	if err != nil {
		return nil, err
	}
	var gogoDesc descriptor.DescriptorProto
	err = gogoproto.Unmarshal(bz, &gogoDesc)
	if err != nil {
		return nil, err
	}

	opts := gogoDesc.Options
	if !gogoproto.HasExtension(opts, v1alpha1.E_Module) {
		return nil, nil
	}

	bz, err = gogoproto.GetRawExtension(gogoproto.GetUnsafeExtensionsMap(opts), v1alpha1.E_Module.Field)
	if err != nil {
		return nil, err
	}

	// we have to skip the field tag and length prefix itself to actually get the raw bytes we want
	// this is really overly complex, but other methods caused runtime errors because of validation
	// that gogo does that appears simply not necessary
	_, _, n := protowire.ConsumeTag(bz)
	bz, _ = protowire.ConsumeBytes(bz[n:])

	var ext v1alpha1.ModuleDescriptor
	err = gogoproto.Unmarshal(bz, &ext)
	if err != nil {
		return nil, err
	}

	return &ext, nil
}
