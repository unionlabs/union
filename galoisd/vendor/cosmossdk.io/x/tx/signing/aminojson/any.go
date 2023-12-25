package aminojson

import (
	"fmt"
	"io"

	"github.com/pkg/errors"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/dynamicpb"
	"google.golang.org/protobuf/types/known/anypb"
)

func marshalAny(enc *Encoder, message protoreflect.Message, writer io.Writer) error {
	// when a message contains a nested any field, and the top-level message has been unmarshalled into a dyanmicpb,
	// the nested any field will also be a dynamicpb. In this case, we must use the dynamicpb API.
	_, ok := message.Interface().(*dynamicpb.Message)
	if ok {
		return marshalDynamic(enc, message, writer)
	}

	anyMsg, ok := message.Interface().(*anypb.Any)
	if !ok {
		return fmt.Errorf("expected *anypb.Any, got %T", message.Interface())
	}

	// build a message of the correct type
	var protoMessage protoreflect.Message
	typ, err := enc.typeResolver.FindMessageByURL(anyMsg.TypeUrl)
	if err == nil {
		// If the type is registered, we can use the proto API to unmarshal into a concrete type.
		valueMsg := typ.New()
		err = proto.Unmarshal(anyMsg.Value, valueMsg.Interface())
		if err != nil {
			return err
		}
		protoMessage = valueMsg
	} else {
		// otherwise we use the dynamicpb API to unmarshal into a dynamic message.
		desc, err := enc.fileResolver.FindDescriptorByName(protoreflect.FullName(anyMsg.TypeUrl[1:]))
		if err != nil {
			return errors.Wrapf(err, "can't resolve type URL %s", anyMsg.TypeUrl)
		}

		valueMsg := dynamicpb.NewMessageType(desc.(protoreflect.MessageDescriptor)).New().Interface()
		err = proto.Unmarshal(anyMsg.Value, valueMsg)
		if err != nil {
			return err
		}
		protoMessage = valueMsg.ProtoReflect()
	}

	return enc.beginMarshal(protoMessage, writer, true)
}

const (
	anyTypeURLFieldName = "type_url"
	anyValueFieldName   = "value"
)

func marshalDynamic(enc *Encoder, message protoreflect.Message, writer io.Writer) error {
	msgName := message.Get(message.Descriptor().Fields().ByName(anyTypeURLFieldName)).String()[1:]
	msgBytes := message.Get(message.Descriptor().Fields().ByName(anyValueFieldName)).Bytes()

	desc, err := enc.fileResolver.FindDescriptorByName(protoreflect.FullName(msgName))
	if err != nil {
		return errors.Wrapf(err, "can't resolve type URL %s", msgName)
	}

	valueMsg := dynamicpb.NewMessageType(desc.(protoreflect.MessageDescriptor)).New().Interface()
	err = proto.Unmarshal(msgBytes, valueMsg)
	if err != nil {
		return err
	}

	return enc.beginMarshal(valueMsg.ProtoReflect(), writer, true)
}
