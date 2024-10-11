package unknownproto

import (
	"bytes"
	"compress/gzip"
	"errors"
	"fmt"
	"io"
	"reflect"
	"strings"
	"sync"

	"github.com/cosmos/gogoproto/jsonpb"
	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/protobuf/encoding/protowire"
	protov2 "google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/descriptorpb"

	"github.com/cosmos/cosmos-sdk/codec/types"
)

const bit11NonCritical = 1 << 10

type descriptorIface interface {
	Descriptor() ([]byte, []int)
}

// RejectUnknownFieldsStrict rejects any bytes bz with an error that has unknown fields for the provided proto.Message type.
// This function traverses inside of messages nested via google.protobuf.Any. It does not do any deserialization of the proto.Message.
// An AnyResolver must be provided for traversing inside google.protobuf.Any's.
func RejectUnknownFieldsStrict(bz []byte, msg proto.Message, resolver jsonpb.AnyResolver) error {
	_, err := RejectUnknownFields(bz, msg, false, resolver)
	return err
}

// RejectUnknownFields rejects any bytes bz with an error that has unknown fields for the provided proto.Message type with an
// option to allow non-critical fields (specified as those fields with bit 11) to pass through. In either case, the
// hasUnknownNonCriticals will be set to true if non-critical fields were encountered during traversal. This flag can be
// used to treat a message with non-critical field different in different security contexts (such as transaction signing).
// This function traverses inside of messages nested via google.protobuf.Any. It does not do any deserialization of the proto.Message.
// An AnyResolver must be provided for traversing inside google.protobuf.Any's.
func RejectUnknownFields(bz []byte, msg proto.Message, allowUnknownNonCriticals bool, resolver jsonpb.AnyResolver) (hasUnknownNonCriticals bool, err error) {
	if len(bz) == 0 {
		return hasUnknownNonCriticals, nil
	}

	fieldDescProtoFromTagNum, _, err := getDescriptorInfo(msg)
	if err != nil {
		return hasUnknownNonCriticals, err
	}

	for len(bz) > 0 {
		tagNum, wireType, m := protowire.ConsumeTag(bz)
		if m < 0 {
			return hasUnknownNonCriticals, errors.New("invalid length")
		}

		fieldDescProto, ok := fieldDescProtoFromTagNum[int32(tagNum)]
		switch {
		case ok:
			// Assert that the wireTypes match.
			if !canEncodeType(wireType, fieldDescProto.GetType()) {
				return hasUnknownNonCriticals, &errMismatchedWireType{
					Type:         reflect.ValueOf(msg).Type().String(),
					TagNum:       tagNum,
					GotWireType:  wireType,
					WantWireType: toProtowireType(fieldDescProto.GetType()),
				}
			}

		default:
			isCriticalField := tagNum&bit11NonCritical == 0

			if !isCriticalField {
				hasUnknownNonCriticals = true
			}

			if isCriticalField || !allowUnknownNonCriticals {
				// The tag is critical, so report it.
				return hasUnknownNonCriticals, &errUnknownField{
					Type:     reflect.ValueOf(msg).Type().String(),
					TagNum:   tagNum,
					WireType: wireType,
				}
			}
		}

		// Skip over the bytes that store fieldNumber and wireType bytes.
		bz = bz[m:]
		n := protowire.ConsumeFieldValue(tagNum, wireType, bz)
		if n < 0 {
			err = fmt.Errorf("could not consume field value for tagNum: %d, wireType: %q; %w",
				tagNum, wireTypeToString(wireType), protowire.ParseError(n))
			return hasUnknownNonCriticals, err
		}
		fieldBytes := bz[:n]
		bz = bz[n:]

		// An unknown but non-critical field or just a scalar type (aka *INT and BYTES like).
		if fieldDescProto == nil || isScalar(fieldDescProto) {
			continue
		}

		protoMessageName := fieldDescProto.GetTypeName()
		if protoMessageName == "" {
			switch typ := fieldDescProto.GetType(); typ {
			case descriptorpb.FieldDescriptorProto_TYPE_STRING, descriptorpb.FieldDescriptorProto_TYPE_BYTES:
				// At this point only TYPE_STRING is expected to be unregistered, since FieldDescriptorProto.IsScalar() returns false for
				// TYPE_BYTES and TYPE_STRING as per
				// https://github.com/cosmos/gogoproto/blob/5628607bb4c51c3157aacc3a50f0ab707582b805/protoc-gen-gogo/descriptor/descriptor.go#L95-L118
			default:
				return hasUnknownNonCriticals, fmt.Errorf("failed to get typename for message of type %v, can only be TYPE_STRING or TYPE_BYTES", typ)
			}
			continue
		}

		// Let's recursively traverse and typecheck the field.

		// consume length prefix of nested message
		_, o := protowire.ConsumeVarint(fieldBytes)
		fieldBytes = fieldBytes[o:]

		var msg proto.Message
		var err error

		if protoMessageName == ".google.protobuf.Any" {
			// Firstly typecheck types.Any to ensure nothing snuck in.
			hasUnknownNonCriticalsChild, err := RejectUnknownFields(fieldBytes, (*types.Any)(nil), allowUnknownNonCriticals, resolver)
			hasUnknownNonCriticals = hasUnknownNonCriticals || hasUnknownNonCriticalsChild
			if err != nil {
				return hasUnknownNonCriticals, err
			}
			// And finally we can extract the TypeURL containing the protoMessageName.
			any := new(types.Any)
			if err := proto.Unmarshal(fieldBytes, any); err != nil {
				return hasUnknownNonCriticals, err
			}
			protoMessageName = any.TypeUrl
			fieldBytes = any.Value
			msg, err = resolver.Resolve(protoMessageName)
			if err != nil {
				return hasUnknownNonCriticals, err
			}
		} else {
			msg, err = protoMessageForTypeName(protoMessageName[1:])
			if err != nil {
				return hasUnknownNonCriticals, err
			}
		}

		hasUnknownNonCriticalsChild, err := RejectUnknownFields(fieldBytes, msg, allowUnknownNonCriticals, resolver)
		hasUnknownNonCriticals = hasUnknownNonCriticals || hasUnknownNonCriticalsChild
		if err != nil {
			return hasUnknownNonCriticals, err
		}
	}

	return hasUnknownNonCriticals, nil
}

var (
	protoMessageForTypeNameMu    sync.RWMutex
	protoMessageForTypeNameCache = make(map[string]proto.Message)
)

// protoMessageForTypeName takes in a fully qualified name e.g. testdata.TestVersionFD1
// and returns a corresponding empty protobuf message that serves the prototype for typechecking.
func protoMessageForTypeName(protoMessageName string) (proto.Message, error) {
	protoMessageForTypeNameMu.RLock()
	msg, ok := protoMessageForTypeNameCache[protoMessageName]
	protoMessageForTypeNameMu.RUnlock()
	if ok {
		return msg, nil
	}

	concreteGoType := proto.MessageType(protoMessageName)
	if concreteGoType == nil {
		return nil, fmt.Errorf("failed to retrieve the message of type %q", protoMessageName)
	}

	value := reflect.New(concreteGoType).Elem()
	msg, ok = value.Interface().(proto.Message)
	if !ok {
		return nil, fmt.Errorf("%q does not implement proto.Message", protoMessageName)
	}

	// Now cache it.
	protoMessageForTypeNameMu.Lock()
	protoMessageForTypeNameCache[protoMessageName] = msg
	protoMessageForTypeNameMu.Unlock()

	return msg, nil
}

// checks is a mapping of protowire.Type to supported descriptor.FieldDescriptorProto_Type.
// it is implemented this way so as to have constant time lookups and avoid the overhead
// from O(n) walking of switch. The change to using this mapping boosts throughput by about 200%.
var checks = [...]map[descriptorpb.FieldDescriptorProto_Type]bool{
	// "0	Varint: int32, int64, uint32, uint64, sint32, sint64, bool, enum"
	0: {
		descriptorpb.FieldDescriptorProto_TYPE_INT32:  true,
		descriptorpb.FieldDescriptorProto_TYPE_INT64:  true,
		descriptorpb.FieldDescriptorProto_TYPE_UINT32: true,
		descriptorpb.FieldDescriptorProto_TYPE_UINT64: true,
		descriptorpb.FieldDescriptorProto_TYPE_SINT32: true,
		descriptorpb.FieldDescriptorProto_TYPE_SINT64: true,
		descriptorpb.FieldDescriptorProto_TYPE_BOOL:   true,
		descriptorpb.FieldDescriptorProto_TYPE_ENUM:   true,
	},

	// "1	64-bit:	fixed64, sfixed64, double"
	1: {
		descriptorpb.FieldDescriptorProto_TYPE_FIXED64:  true,
		descriptorpb.FieldDescriptorProto_TYPE_SFIXED64: true,
		descriptorpb.FieldDescriptorProto_TYPE_DOUBLE:   true,
	},

	// "2	Length-delimited: string, bytes, embedded messages, packed repeated fields"
	2: {
		descriptorpb.FieldDescriptorProto_TYPE_STRING:  true,
		descriptorpb.FieldDescriptorProto_TYPE_BYTES:   true,
		descriptorpb.FieldDescriptorProto_TYPE_MESSAGE: true,
		// The following types can be packed repeated.
		// ref: "Only repeated fields of primitive numeric types (types which use the varint, 32-bit, or 64-bit wire types) can be declared "packed"."
		// ref: https://protobuf.dev/programming-guides/encoding/#packed
		descriptorpb.FieldDescriptorProto_TYPE_INT32:    true,
		descriptorpb.FieldDescriptorProto_TYPE_INT64:    true,
		descriptorpb.FieldDescriptorProto_TYPE_UINT32:   true,
		descriptorpb.FieldDescriptorProto_TYPE_UINT64:   true,
		descriptorpb.FieldDescriptorProto_TYPE_SINT32:   true,
		descriptorpb.FieldDescriptorProto_TYPE_SINT64:   true,
		descriptorpb.FieldDescriptorProto_TYPE_BOOL:     true,
		descriptorpb.FieldDescriptorProto_TYPE_ENUM:     true,
		descriptorpb.FieldDescriptorProto_TYPE_FIXED64:  true,
		descriptorpb.FieldDescriptorProto_TYPE_SFIXED64: true,
		descriptorpb.FieldDescriptorProto_TYPE_DOUBLE:   true,
	},

	// "3	Start group:	groups (deprecated)"
	3: {
		descriptorpb.FieldDescriptorProto_TYPE_GROUP: true,
	},

	// "4	End group:	groups (deprecated)"
	4: {
		descriptorpb.FieldDescriptorProto_TYPE_GROUP: true,
	},

	// "5	32-bit:	fixed32, sfixed32, float"
	5: {
		descriptorpb.FieldDescriptorProto_TYPE_FIXED32:  true,
		descriptorpb.FieldDescriptorProto_TYPE_SFIXED32: true,
		descriptorpb.FieldDescriptorProto_TYPE_FLOAT:    true,
	},
}

// canEncodeType returns true if the wireType is suitable for encoding the descriptor type.
// See https://protobuf.dev/programming-guides/encoding/#structure.
func canEncodeType(wireType protowire.Type, descType descriptorpb.FieldDescriptorProto_Type) bool {
	if iwt := int(wireType); iwt < 0 || iwt >= len(checks) {
		return false
	}
	return checks[wireType][descType]
}

// errMismatchedWireType describes a mismatch between
// expected and got wireTypes for a specific tag number.
type errMismatchedWireType struct {
	Type         string
	GotWireType  protowire.Type
	WantWireType protowire.Type
	TagNum       protowire.Number
}

// String implements fmt.Stringer.
func (mwt *errMismatchedWireType) String() string {
	return fmt.Sprintf("Mismatched %q: {TagNum: %d, GotWireType: %q != WantWireType: %q}",
		mwt.Type, mwt.TagNum, wireTypeToString(mwt.GotWireType), wireTypeToString(mwt.WantWireType))
}

// Error implements the error interface.
func (mwt *errMismatchedWireType) Error() string {
	return mwt.String()
}

var _ error = (*errMismatchedWireType)(nil)

func wireTypeToString(wt protowire.Type) string {
	switch wt {
	case 0:
		return "varint"
	case 1:
		return "fixed64"
	case 2:
		return "bytes"
	case 3:
		return "start_group"
	case 4:
		return "end_group"
	case 5:
		return "fixed32"
	default:
		return fmt.Sprintf("unknown type: %d", wt)
	}
}

// errUnknownField represents an error indicating that we encountered
// a field that isn't available in the target proto.Message.
type errUnknownField struct {
	Type     string
	TagNum   protowire.Number
	WireType protowire.Type
}

// String implements fmt.Stringer.
func (twt *errUnknownField) String() string {
	return fmt.Sprintf("errUnknownField %q: {TagNum: %d, WireType:%q}",
		twt.Type, twt.TagNum, wireTypeToString(twt.WireType))
}

// Error implements the error interface.
func (twt *errUnknownField) Error() string {
	return twt.String()
}

var _ error = (*errUnknownField)(nil)

var (
	protoFileToDesc   = make(map[string]*descriptorpb.FileDescriptorProto)
	protoFileToDescMu sync.RWMutex
)

func unnestDesc(mdescs []*descriptorpb.DescriptorProto, indices []int) *descriptorpb.DescriptorProto {
	mdesc := mdescs[indices[0]]
	for _, index := range indices[1:] {
		mdesc = mdesc.NestedType[index]
	}
	return mdesc
}

// Invoking descriptorpb.ForMessage(proto.Message.(Descriptor).Descriptor()) is incredibly slow
// for every single message, thus the need for a hand-rolled custom version that's performant and cacheable.
func extractFileDescMessageDesc(msg proto.Message) (*descriptorpb.FileDescriptorProto, *descriptorpb.DescriptorProto, error) {
	desc, ok := msg.(descriptorIface)
	if !ok {
		return nil, nil, fmt.Errorf("%T does not have a Descriptor() method", msg)
	}
	gzippedPb, indices := desc.Descriptor()

	protoFileToDescMu.RLock()
	cached, ok := protoFileToDesc[string(gzippedPb)]
	protoFileToDescMu.RUnlock()

	if ok {
		return cached, unnestDesc(cached.MessageType, indices), nil
	}

	// Time to gunzip the content of the FileDescriptor and then proto unmarshal them.
	gzr, err := gzip.NewReader(bytes.NewReader(gzippedPb))
	if err != nil {
		return nil, nil, err
	}
	protoBlob, err := io.ReadAll(gzr)
	if err != nil {
		return nil, nil, err
	}

	fdesc := new(descriptorpb.FileDescriptorProto)
	if err := protov2.Unmarshal(protoBlob, fdesc); err != nil {
		return nil, nil, err
	}

	// Now cache the FileDescriptor.
	protoFileToDescMu.Lock()
	protoFileToDesc[string(gzippedPb)] = fdesc
	protoFileToDescMu.Unlock()

	// Unnest the type if necessary.
	return fdesc, unnestDesc(fdesc.MessageType, indices), nil
}

type descriptorMatch struct {
	cache map[int32]*descriptorpb.FieldDescriptorProto
	desc  *descriptorpb.DescriptorProto
}

var (
	descprotoCacheMu sync.RWMutex
	descprotoCache   = make(map[reflect.Type]*descriptorMatch)
)

// getDescriptorInfo retrieves the mapping of field numbers to their respective field descriptors.
func getDescriptorInfo(msg proto.Message) (map[int32]*descriptorpb.FieldDescriptorProto, *descriptorpb.DescriptorProto, error) {
	// we immediately check if the desc is present in the desc
	key := reflect.ValueOf(msg).Type()

	descprotoCacheMu.RLock()
	got, ok := descprotoCache[key]
	descprotoCacheMu.RUnlock()

	if ok {
		return got.cache, got.desc, nil
	}

	// Now compute and cache the index.
	_, md, err := extractFileDescMessageDesc(msg)
	if err != nil {
		return nil, nil, err
	}

	tagNumToTypeIndex := make(map[int32]*descriptorpb.FieldDescriptorProto)
	for _, field := range md.Field {
		tagNumToTypeIndex[field.GetNumber()] = field
	}

	descprotoCacheMu.Lock()
	descprotoCache[key] = &descriptorMatch{
		cache: tagNumToTypeIndex,
		desc:  md,
	}
	descprotoCacheMu.Unlock()

	return tagNumToTypeIndex, md, nil
}

// DefaultAnyResolver is a default implementation of AnyResolver which uses
// the default encoding of type URLs as specified by the protobuf specification.
type DefaultAnyResolver struct{}

var _ jsonpb.AnyResolver = DefaultAnyResolver{}

// Resolve is the AnyResolver.Resolve method.
func (d DefaultAnyResolver) Resolve(typeURL string) (proto.Message, error) {
	// Only the part of typeURL after the last slash is relevant.
	mname := typeURL
	if slash := strings.LastIndex(mname, "/"); slash >= 0 {
		mname = mname[slash+1:]
	}
	mt := proto.MessageType(mname)
	if mt == nil {
		return nil, fmt.Errorf("unknown message type %q", mname)
	}
	return reflect.New(mt.Elem()).Interface().(proto.Message), nil
}

// toProtowireType converts a descriptorpb.FieldDescriptorProto_Type to a protowire.Type.
func toProtowireType(fieldType descriptorpb.FieldDescriptorProto_Type) protowire.Type {
	switch fieldType {
	// varint encoded
	case descriptorpb.FieldDescriptorProto_TYPE_INT64,
		descriptorpb.FieldDescriptorProto_TYPE_UINT64,
		descriptorpb.FieldDescriptorProto_TYPE_INT32,
		descriptorpb.FieldDescriptorProto_TYPE_UINT32,
		descriptorpb.FieldDescriptorProto_TYPE_BOOL,
		descriptorpb.FieldDescriptorProto_TYPE_ENUM,
		descriptorpb.FieldDescriptorProto_TYPE_SINT32,
		descriptorpb.FieldDescriptorProto_TYPE_SINT64:
		return protowire.VarintType

	// fixed64 encoded
	case descriptorpb.FieldDescriptorProto_TYPE_DOUBLE,
		descriptorpb.FieldDescriptorProto_TYPE_FIXED64,
		descriptorpb.FieldDescriptorProto_TYPE_SFIXED64:
		return protowire.Fixed64Type

	// fixed32 encoded
	case descriptorpb.FieldDescriptorProto_TYPE_FLOAT,
		descriptorpb.FieldDescriptorProto_TYPE_FIXED32,
		descriptorpb.FieldDescriptorProto_TYPE_SFIXED32:
		return protowire.Fixed32Type

	// bytes encoded
	case descriptorpb.FieldDescriptorProto_TYPE_STRING,
		descriptorpb.FieldDescriptorProto_TYPE_BYTES,
		descriptorpb.FieldDescriptorProto_TYPE_MESSAGE,
		descriptorpb.FieldDescriptorProto_TYPE_GROUP:
		return protowire.BytesType
	default:
		panic(fmt.Sprintf("unknown field type %s", fieldType))
	}
}

// isScalar defines whether a field is a scalar type.
// Copied from gogo/protobuf/protoc-gen-gogo
// https://github.com/gogo/protobuf/blob/b03c65ea87cdc3521ede29f62fe3ce239267c1bc/protoc-gen-gogo/descriptor/descriptor.go#L95
func isScalar(field *descriptorpb.FieldDescriptorProto) bool {
	if field.Type == nil {
		return false
	}
	switch *field.Type {
	case descriptorpb.FieldDescriptorProto_TYPE_DOUBLE,
		descriptorpb.FieldDescriptorProto_TYPE_FLOAT,
		descriptorpb.FieldDescriptorProto_TYPE_INT64,
		descriptorpb.FieldDescriptorProto_TYPE_UINT64,
		descriptorpb.FieldDescriptorProto_TYPE_INT32,
		descriptorpb.FieldDescriptorProto_TYPE_FIXED64,
		descriptorpb.FieldDescriptorProto_TYPE_FIXED32,
		descriptorpb.FieldDescriptorProto_TYPE_BOOL,
		descriptorpb.FieldDescriptorProto_TYPE_UINT32,
		descriptorpb.FieldDescriptorProto_TYPE_ENUM,
		descriptorpb.FieldDescriptorProto_TYPE_SFIXED32,
		descriptorpb.FieldDescriptorProto_TYPE_SFIXED64,
		descriptorpb.FieldDescriptorProto_TYPE_SINT32,
		descriptorpb.FieldDescriptorProto_TYPE_SINT64:
		return true
	default:
		return false
	}
}
