package aminojson

import (
	cosmos_proto "github.com/cosmos/cosmos-proto"
	gogoproto "github.com/cosmos/gogoproto/proto"
	"github.com/iancoleman/strcase"
	"github.com/pkg/errors"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"

	"cosmossdk.io/api/amino"
)

// getMessageAminoName returns the amino name of a message if it has been set by the `amino.name` option.
// If the message does not have an amino name, then the function returns false.
func getMessageAminoName(msg protoreflect.Message) (string, bool) {
	messageOptions := msg.Descriptor().Options()
	if proto.HasExtension(messageOptions, amino.E_Name) {
		name := proto.GetExtension(messageOptions, amino.E_Name)
		return name.(string), true
	}

	return "", false
}

// getMessageAminoName returns the amino name of a message if it has been set by the `amino.name` option.
// If the message does not have an amino name, then it returns the msg url.
// If it cannot get the msg url, then it returns false.
func getMessageAminoNameAny(msg protoreflect.Message) string {
	messageOptions := msg.Descriptor().Options()
	if proto.HasExtension(messageOptions, amino.E_Name) {
		name := proto.GetExtension(messageOptions, amino.E_Name)
		return name.(string)
	}

	msgURL := "/" + string(msg.Descriptor().FullName())
	if msgURL != "/" {
		return msgURL
	}

	if m, ok := msg.(gogoproto.Message); ok {
		return "/" + gogoproto.MessageName(m)
	}

	return ""
}

// omitEmpty returns true if the field should be omitted if empty. Empty field omission is the default behavior.
func omitEmpty(field protoreflect.FieldDescriptor) bool {
	opts := field.Options()
	if proto.HasExtension(opts, amino.E_DontOmitempty) {
		dontOmitEmpty := proto.GetExtension(opts, amino.E_DontOmitempty).(bool)
		return !dontOmitEmpty
	}
	return true
}

// getAminoFieldName returns the amino field name of a field if it has been set by the `amino.field_name` option.
// If the field does not have an amino field name, then the function returns the protobuf field name.
func getAminoFieldName(field protoreflect.FieldDescriptor) string {
	opts := field.Options()
	if proto.HasExtension(opts, amino.E_FieldName) {
		return proto.GetExtension(opts, amino.E_FieldName).(string)
	}
	return string(field.Name())
}

func getOneOfNames(field protoreflect.FieldDescriptor) (string, string, error) {
	opts := field.Options()
	oneOf := field.ContainingOneof()
	if oneOf == nil {
		return "", "", errors.Errorf("field %s must be within a oneof", field.Name())
	}

	fieldName := strcase.ToCamel(string(oneOf.Name()))
	var typeName string

	if proto.HasExtension(opts, amino.E_OneofName) {
		typeName = proto.GetExtension(opts, amino.E_OneofName).(string)
	} else {
		return "", "", errors.Errorf("field %s within a oneof must have the amino.oneof_type_name option set",
			field.Name())
	}

	return fieldName, typeName, nil
}

func (enc Encoder) getMessageEncoder(message protoreflect.Message) MessageEncoder {
	opts := message.Descriptor().Options()
	if proto.HasExtension(opts, amino.E_MessageEncoding) {
		encoding := proto.GetExtension(opts, amino.E_MessageEncoding).(string)
		if fn, ok := enc.aminoMessageEncoders[encoding]; ok {
			return fn
		}
	}
	return nil
}

func (enc Encoder) getFieldEncoding(field protoreflect.FieldDescriptor) FieldEncoder {
	opts := field.Options()
	if proto.HasExtension(opts, amino.E_Encoding) {
		encoding := proto.GetExtension(opts, amino.E_Encoding).(string)
		if fn, ok := enc.aminoFieldEncoders[encoding]; ok {
			return fn
		}
	}
	if proto.HasExtension(opts, cosmos_proto.E_Scalar) {
		scalar := proto.GetExtension(opts, cosmos_proto.E_Scalar).(string)
		if fn, ok := enc.cosmosProtoScalarEncoders[scalar]; ok {
			return fn
		}
	}
	return nil
}
