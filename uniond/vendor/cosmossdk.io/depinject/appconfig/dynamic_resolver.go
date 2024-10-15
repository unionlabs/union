package appconfig

import (
	"strings"

	"google.golang.org/protobuf/reflect/protodesc"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"
	"google.golang.org/protobuf/types/dynamicpb"
)

// dynamic resolver allows marshaling gogo proto messages from the gogoproto.HybridResolver as long as those
// files have been imported before calling LoadJSON. There is similar code in autocli, this should probably
// eventually be moved into a library.
type dynamicTypeResolver struct {
	resolver protodesc.Resolver
}

func (r dynamicTypeResolver) FindExtensionByName(field protoreflect.FullName) (protoreflect.ExtensionType, error) {
	ext, err := protoregistry.GlobalTypes.FindExtensionByName(field)
	if err == nil {
		return ext, nil
	}

	desc, err := r.resolver.FindDescriptorByName(field)
	if err != nil {
		return nil, err
	}

	return dynamicpb.NewExtensionType(desc.(protoreflect.ExtensionTypeDescriptor)), nil
}

func (r dynamicTypeResolver) FindExtensionByNumber(message protoreflect.FullName, field protoreflect.FieldNumber) (protoreflect.ExtensionType, error) {
	ext, err := protoregistry.GlobalTypes.FindExtensionByNumber(message, field)
	if err == nil {
		return ext, nil
	}

	desc, err := r.resolver.FindDescriptorByName(message)
	if err != nil {
		return nil, err
	}

	messageDesc := desc.(protoreflect.MessageDescriptor)
	exts := messageDesc.Extensions()
	n := exts.Len()
	for i := 0; i < n; i++ {
		ext := exts.Get(i)
		if ext.Number() == field {
			return dynamicpb.NewExtensionType(ext), nil
		}
	}

	return nil, protoregistry.NotFound
}

func (r dynamicTypeResolver) FindMessageByName(message protoreflect.FullName) (protoreflect.MessageType, error) {
	typ, err := protoregistry.GlobalTypes.FindMessageByName(message)
	if err == nil {
		return typ, nil
	}

	desc, err := r.resolver.FindDescriptorByName(message)
	if err != nil {
		return nil, err
	}

	return dynamicpb.NewMessageType(desc.(protoreflect.MessageDescriptor)), nil
}

func (r dynamicTypeResolver) FindMessageByURL(url string) (protoreflect.MessageType, error) {
	if i := strings.LastIndexByte(url, '/'); i >= 0 {
		url = url[i+1:]
	}

	return r.FindMessageByName(protoreflect.FullName(url))
}

var (
	_ protoregistry.MessageTypeResolver   = dynamicTypeResolver{}
	_ protoregistry.ExtensionTypeResolver = dynamicTypeResolver{}
)
