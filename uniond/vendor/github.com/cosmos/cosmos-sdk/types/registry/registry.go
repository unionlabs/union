package registry

import (
	"sync"

	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"

	"cosmossdk.io/x/tx/signing"
)

var (
	mergedRegistryOnce sync.Once
	mergedRegistry     *protoregistry.Files
	_                  signing.ProtoFileResolver = lazyProtoRegistry{}
)

// lazyProtoRegistry is a lazy loading wrapper around the global protobuf registry.
type lazyProtoRegistry struct{}

func getRegistry() *protoregistry.Files {
	var err error
	mergedRegistryOnce.Do(func() {
		mergedRegistry, err = proto.MergedRegistry()
		if err != nil {
			panic(err)
		}
	})
	return mergedRegistry
}

func (l lazyProtoRegistry) FindFileByPath(s string) (protoreflect.FileDescriptor, error) {
	reg := getRegistry()
	return reg.FindFileByPath(s)
}

func (l lazyProtoRegistry) FindDescriptorByName(name protoreflect.FullName) (protoreflect.Descriptor, error) {
	reg := getRegistry()
	return reg.FindDescriptorByName(name)
}

func (l lazyProtoRegistry) RangeFiles(f func(protoreflect.FileDescriptor) bool) {
	reg := getRegistry()
	reg.RangeFiles(f)
}

// MergedProtoRegistry returns a lazy loading wrapper around the global protobuf registry, a merged registry
// containing both gogo proto and pulsar types.
func MergedProtoRegistry() signing.ProtoFileResolver {
	return lazyProtoRegistry{}
}
