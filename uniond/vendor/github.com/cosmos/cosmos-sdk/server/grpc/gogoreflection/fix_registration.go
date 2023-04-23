package gogoreflection

import (
	"reflect"

	_ "github.com/cosmos/gogoproto/gogoproto" // required so it does register the gogoproto file descriptor
	gogoproto "github.com/cosmos/gogoproto/proto"

	_ "github.com/cosmos/cosmos-proto" // look above
	"github.com/golang/protobuf/proto" //nolint:staticcheck
)

func getFileDescriptor(filePath string) []byte {
	// Since we got well known descriptors which are not registered into gogoproto
	// registry but are instead registered into the proto one, we need to check both.
	fd := gogoproto.FileDescriptor(filePath)
	if len(fd) != 0 {
		return fd
	}

	return proto.FileDescriptor(filePath) //nolint:staticcheck
}

func getMessageType(name string) reflect.Type {
	typ := gogoproto.MessageType(name)
	if typ != nil {
		return typ
	}

	return proto.MessageType(name) //nolint:staticcheck
}

func getExtension(extID int32, m proto.Message) *gogoproto.ExtensionDesc {
	// check first in gogoproto registry
	for id, desc := range gogoproto.RegisteredExtensions(m) {
		if id == extID {
			return desc
		}
	}

	// check into proto registry
	//nolint:staticcheck
	for id, desc := range proto.RegisteredExtensions(m) {
		if id == extID {
			return &gogoproto.ExtensionDesc{
				ExtendedType:  desc.ExtendedType,
				ExtensionType: desc.ExtensionType,
				Field:         desc.Field,
				Name:          desc.Name,
				Tag:           desc.Tag,
				Filename:      desc.Filename,
			}
		}
	}

	return nil
}

func getExtensionsNumbers(m proto.Message) []int32 {
	gogoProtoExts := gogoproto.RegisteredExtensions(m)

	out := make([]int32, 0, len(gogoProtoExts))
	for id := range gogoProtoExts {
		out = append(out, id)
	}
	if len(out) != 0 {
		return out
	}

	protoExts := proto.RegisteredExtensions(m) //nolint:staticcheck
	out = make([]int32, 0, len(protoExts))
	for id := range protoExts {
		out = append(out, id)
	}

	return out
}
