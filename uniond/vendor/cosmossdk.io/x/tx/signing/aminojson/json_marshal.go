package aminojson

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"sort"

	gogoproto "github.com/cosmos/gogoproto/proto"
	"github.com/pkg/errors"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"

	"cosmossdk.io/x/tx/signing"
)

// MessageEncoder is a function that can encode a protobuf protoreflect.Message to JSON.
type MessageEncoder func(*Encoder, protoreflect.Message, io.Writer) error

// FieldEncoder is a function that can encode a protobuf protoreflect.Value to JSON.
type FieldEncoder func(*Encoder, protoreflect.Value, io.Writer) error

// EncoderOptions are options for creating a new Encoder.
type EncoderOptions struct {
	// Indent can only be composed of space or tab characters.
	// It defines the indentation used for each level of indentation.
	Indent string
	// DoNotSortFields when set turns off sorting of field names.
	DoNotSortFields bool
	// EnumAsString when set will encode enums as strings instead of integers.
	// Caution: Enabling this option produce different sign bytes.
	EnumAsString bool
	// TypeResolver is used to resolve protobuf message types by TypeURL when marshaling any packed messages.
	TypeResolver signing.TypeResolver
	// FileResolver is used to resolve protobuf file descriptors TypeURL when TypeResolver fails.
	FileResolver signing.ProtoFileResolver
}

// Encoder is a JSON encoder that uses the Amino JSON encoding rules for protobuf messages.
type Encoder struct {
	// maps cosmos_proto.scalar -> field encoder
	cosmosProtoScalarEncoders map[string]FieldEncoder
	aminoMessageEncoders      map[string]MessageEncoder
	aminoFieldEncoders        map[string]FieldEncoder
	protoTypeEncoders         map[string]MessageEncoder
	fileResolver              signing.ProtoFileResolver
	typeResolver              protoregistry.MessageTypeResolver
	doNotSortFields           bool
	indent                    string
	enumsAsString             bool
}

// NewEncoder returns a new Encoder capable of serializing protobuf messages to JSON using the Amino JSON encoding
// rules.
func NewEncoder(options EncoderOptions) Encoder {
	if options.FileResolver == nil {
		options.FileResolver = gogoproto.HybridResolver
	}
	if options.TypeResolver == nil {
		options.TypeResolver = protoregistry.GlobalTypes
	}
	enc := Encoder{
		cosmosProtoScalarEncoders: map[string]FieldEncoder{
			"cosmos.Dec": cosmosDecEncoder,
			"cosmos.Int": cosmosIntEncoder,
		},
		aminoMessageEncoders: map[string]MessageEncoder{
			"key_field":        keyFieldEncoder,
			"module_account":   moduleAccountEncoder,
			"threshold_string": thresholdStringEncoder,
		},
		aminoFieldEncoders: map[string]FieldEncoder{
			"legacy_coins": nullSliceAsEmptyEncoder,
			"inline_json":  cosmosInlineJSON,
		},
		protoTypeEncoders: map[string]MessageEncoder{
			"google.protobuf.Timestamp": marshalTimestamp,
			"google.protobuf.Duration":  marshalDuration,
			"google.protobuf.Any":       marshalAny,
		},
		fileResolver:    options.FileResolver,
		typeResolver:    options.TypeResolver,
		doNotSortFields: options.DoNotSortFields,
		indent:          options.Indent,
		enumsAsString:   options.EnumAsString,
	}
	return enc
}

// DefineMessageEncoding defines a custom encoding for a protobuf message.  The `name` field must match a usage of
// an (amino.message_encoding) option in the protobuf message as in the following example.  This encoding will be
// used instead of the default encoding for all usages of the tagged message.
//
//	message ModuleAccount {
//	  option (amino.name)                        = "cosmos-sdk/ModuleAccount";
//	  option (amino.message_encoding)            = "module_account";
//	  ...
//	}
func (enc Encoder) DefineMessageEncoding(name string, encoder MessageEncoder) Encoder {
	if enc.aminoMessageEncoders == nil {
		enc.aminoMessageEncoders = map[string]MessageEncoder{}
	}
	enc.aminoMessageEncoders[name] = encoder
	return enc
}

// DefineFieldEncoding defines a custom encoding for a protobuf field.  The `name` field must match a usage of
// an (amino.encoding) option in the protobuf message as in the following example. This encoding will be used
// instead of the default encoding for all usages of the tagged field.
//
//	message Balance {
//	  repeated cosmos.base.v1beta1.Coin coins = 2 [
//	    (amino.encoding)         = "legacy_coins",
//	    (gogoproto.castrepeated) = "github.com/cosmos/cosmos-sdk/types.Coins",
//	    (gogoproto.nullable)     = false,
//	    (amino.dont_omitempty)   = true
//	  ];
//	  ...
//	}
func (enc Encoder) DefineFieldEncoding(name string, encoder FieldEncoder) Encoder {
	if enc.aminoFieldEncoders == nil {
		enc.aminoFieldEncoders = map[string]FieldEncoder{}
	}
	enc.aminoFieldEncoders[name] = encoder
	return enc
}

// DefineScalarEncoding defines a custom encoding for a protobuf scalar field.  The `name` field must match a usage of
// an (cosmos_proto.scalar) option in the protobuf message as in the following example. This encoding will be used
// instead of the default encoding for all usages of the tagged field.
//
//	message Balance {
//	  string address = 1 [(cosmos_proto.scalar) = "cosmos.AddressString"];
//	  ...
//	}
func (enc Encoder) DefineScalarEncoding(name string, encoder FieldEncoder) Encoder {
	if enc.cosmosProtoScalarEncoders == nil {
		enc.cosmosProtoScalarEncoders = map[string]FieldEncoder{}
	}
	enc.cosmosProtoScalarEncoders[name] = encoder
	return enc
}

// DefineTypeEncoding defines a custom encoding for a protobuf message type.  The `typeURL` field must match the
// type of the protobuf message as in the following example. This encoding will be used instead of the default
// encoding for all usages of the tagged message.
//
//	message Foo {
//	  google.protobuf.Duration type_url = 1;
//	  ...
//	}

func (enc Encoder) DefineTypeEncoding(typeURL string, encoder MessageEncoder) Encoder {
	if enc.protoTypeEncoders == nil {
		enc.protoTypeEncoders = map[string]MessageEncoder{}
	}
	enc.protoTypeEncoders[typeURL] = encoder
	return enc
}

// Marshal serializes a protobuf message to JSON.
func (enc Encoder) Marshal(message proto.Message) ([]byte, error) {
	buf := &bytes.Buffer{}
	err := enc.beginMarshal(message.ProtoReflect(), buf, false)
	if err != nil {
		return nil, err
	}

	if enc.indent != "" {
		indentBuf := &bytes.Buffer{}
		if err := json.Indent(indentBuf, buf.Bytes(), "", enc.indent); err != nil {
			return nil, err
		}

		return indentBuf.Bytes(), nil
	}

	return buf.Bytes(), nil
}

func (enc Encoder) beginMarshal(msg protoreflect.Message, writer io.Writer, isAny bool) error {
	var (
		name  string
		named bool
	)

	if isAny {
		name, named = getMessageAminoNameAny(msg), true
	} else {
		name, named = getMessageAminoName(msg)
	}

	if named {
		_, err := fmt.Fprintf(writer, `{"type":"%s","value":`, name)
		if err != nil {
			return err
		}
	}

	err := enc.marshal(protoreflect.ValueOfMessage(msg), nil /* no field descriptor needed here */, writer)
	if err != nil {
		return err
	}

	if named {
		_, err = io.WriteString(writer, "}")
		if err != nil {
			return err
		}
	}

	return nil
}

func (enc Encoder) marshal(value protoreflect.Value, fd protoreflect.FieldDescriptor, writer io.Writer) error {
	switch val := value.Interface().(type) {
	case protoreflect.Message:
		err := enc.marshalMessage(val, writer)
		return err

	case protoreflect.Map:
		return errors.New("maps are not supported")

	case protoreflect.List:
		if !val.IsValid() {
			_, err := io.WriteString(writer, "null")
			return err
		}
		return enc.marshalList(val, fd, writer)

	case string, bool, int32, uint32, []byte:
		return jsonMarshal(writer, val)

	case protoreflect.EnumNumber:
		if enc.enumsAsString && fd != nil {
			desc := fd.Enum().Values().ByNumber(val)
			if desc != nil {
				_, err := io.WriteString(writer, fmt.Sprintf(`"%s"`, desc.Name()))
				return err
			}
		}

		return jsonMarshal(writer, val)

	case uint64, int64:
		_, err := fmt.Fprintf(writer, `"%d"`, val) // quoted
		return err

	default:
		return errors.Errorf("unknown type %T", val)
	}
}

type nameAndIndex struct {
	i    int
	name string
}

func (enc Encoder) marshalMessage(msg protoreflect.Message, writer io.Writer) error {
	if msg == nil {
		return errors.New("nil message")
	}

	// check if we have a custom type encoder for this type
	if typeEnc, ok := enc.protoTypeEncoders[string(msg.Descriptor().FullName())]; ok {
		return typeEnc(&enc, msg, writer)
	}

	if encoder := enc.getMessageEncoder(msg); encoder != nil {
		err := encoder(&enc, msg, writer)
		return err
	}

	_, err := io.WriteString(writer, "{")
	if err != nil {
		return err
	}

	fields := msg.Descriptor().Fields()
	first := true
	emptyOneOfWritten := map[string]bool{}

	// 1. If permitted, ensure the names are sorted.
	indices := make([]*nameAndIndex, 0, fields.Len())
	for i := 0; i < fields.Len(); i++ {
		f := fields.Get(i)
		name := getAminoFieldName(f)
		indices = append(indices, &nameAndIndex{i: i, name: name})
	}

	if shouldSortFields := !enc.doNotSortFields; shouldSortFields {
		sort.Slice(indices, func(i, j int) bool {
			ni, nj := indices[i], indices[j]
			return ni.name < nj.name
		})
	}

	for _, ni := range indices {
		i := ni.i
		name := ni.name
		f := fields.Get(i)
		v := msg.Get(f)
		oneof := f.ContainingOneof()
		isOneOf := oneof != nil
		oneofFieldName, oneofTypeName, err := getOneOfNames(f)
		if err != nil && isOneOf {
			return err
		}
		writeNil := false

		if !msg.Has(f) {
			// msg.WhichOneof(oneof) == nil: no field of the oneof has been set
			// !emptyOneOfWritten: we haven't written a null for this oneof yet (only write one null per empty oneof)
			switch {
			case isOneOf && msg.WhichOneof(oneof) == nil && !emptyOneOfWritten[oneofFieldName]:
				name = oneofFieldName
				writeNil = true
				emptyOneOfWritten[oneofFieldName] = true
			case omitEmpty(f):
				continue
			case f.Kind() == protoreflect.MessageKind &&
				f.Cardinality() != protoreflect.Repeated &&
				!v.Message().IsValid():
				return errors.Errorf("not supported: dont_omit_empty=true on invalid (nil?) message field: %s", name)
			}
		}

		if !first {
			_, err = io.WriteString(writer, ",")
			if err != nil {
				return err
			}
		}

		if isOneOf && !writeNil {
			_, err = fmt.Fprintf(writer, `"%s":{"type":"%s","value":{`, oneofFieldName, oneofTypeName)
			if err != nil {
				return err
			}
		}

		err = jsonMarshal(writer, name)
		if err != nil {
			return err
		}

		_, err = io.WriteString(writer, ":")
		if err != nil {
			return err
		}

		// encode value
		if encoder := enc.getFieldEncoding(f); encoder != nil {
			err = encoder(&enc, v, writer)
			if err != nil {
				return err
			}
		} else if writeNil {
			_, err = io.WriteString(writer, "null")
			if err != nil {
				return err
			}
		} else {
			err = enc.marshal(v, f, writer)
			if err != nil {
				return err
			}
		}

		if isOneOf && !writeNil {
			_, err = io.WriteString(writer, "}}")
			if err != nil {
				return err
			}
		}

		first = false
	}

	_, err = io.WriteString(writer, "}")
	return err
}

func jsonMarshal(w io.Writer, v interface{}) error {
	blob, err := json.Marshal(v)
	if err != nil {
		return err
	}
	_, err = w.Write(blob)
	return err
}

func (enc Encoder) marshalList(list protoreflect.List, fd protoreflect.FieldDescriptor, writer io.Writer) error {
	n := list.Len()

	_, err := io.WriteString(writer, "[")
	if err != nil {
		return err
	}

	first := true
	for i := 0; i < n; i++ {
		if !first {
			_, err := io.WriteString(writer, ",")
			if err != nil {
				return err
			}
		}
		first = false

		err = enc.marshal(list.Get(i), fd, writer)
		if err != nil {
			return err
		}
	}

	_, err = io.WriteString(writer, "]")
	return err
}
