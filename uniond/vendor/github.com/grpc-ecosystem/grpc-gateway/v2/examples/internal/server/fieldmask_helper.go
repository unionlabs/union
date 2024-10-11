package server

import (
	"strings"

	"google.golang.org/genproto/protobuf/field_mask"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
)

func applyFieldMask(patchee, patcher proto.Message, mask *field_mask.FieldMask) {
	if mask == nil {
		return
	}
	if patchee.ProtoReflect().Descriptor().FullName() != patcher.ProtoReflect().Descriptor().FullName() {
		panic("patchee and patcher must be same type")
	}

	for _, path := range mask.GetPaths() {
		patcherField, patcherParent := getField(patcher.ProtoReflect(), path)
		patcheeField, patcheeParent := getField(patchee.ProtoReflect(), path)
		patcheeParent.Set(patcheeField, patcherParent.Get(patcherField))
	}
}

func getField(msg protoreflect.Message, path string) (field protoreflect.FieldDescriptor, parent protoreflect.Message) {
	fields := msg.Descriptor().Fields()
	parent = msg
	names := strings.Split(path, ".")
	for i, name := range names {
		field = fields.ByName(protoreflect.Name(name))

		if i < len(names)-1 {
			parent = parent.Get(field).Message()
			fields = field.Message().Fields()
		}
	}

	return field, parent
}
