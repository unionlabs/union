package decode

import (
	"errors"
	"fmt"
	"strings"

	"google.golang.org/protobuf/encoding/protowire"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protodesc"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/known/anypb"
)

const bit11NonCritical = 1 << 10

var (
	anyDesc     = (&anypb.Any{}).ProtoReflect().Descriptor()
	anyFullName = anyDesc.FullName()
)

// RejectUnknownFieldsStrict operates by the same rules as RejectUnknownFields, but returns an error if any unknown
// non-critical fields are encountered.
func RejectUnknownFieldsStrict(bz []byte, msg protoreflect.MessageDescriptor, resolver protodesc.Resolver) error {
	_, err := RejectUnknownFields(bz, msg, false, resolver)
	return err
}

// RejectUnknownFields rejects any bytes bz with an error that has unknown fields for the provided proto.Message type with an
// option to allow non-critical fields (specified as those fields with bit 11) to pass through. In either case, the
// hasUnknownNonCriticals will be set to true if non-critical fields were encountered during traversal. This flag can be
// used to treat a message with non-critical field different in different security contexts (such as transaction signing).
// This function traverses inside of messages nested via google.protobuf.Any. It does not do any deserialization of the proto.Message.
// An AnyResolver must be provided for traversing inside google.protobuf.Any's.
func RejectUnknownFields(bz []byte, desc protoreflect.MessageDescriptor, allowUnknownNonCriticals bool, resolver protodesc.Resolver) (hasUnknownNonCriticals bool, err error) {
	if len(bz) == 0 {
		return hasUnknownNonCriticals, nil
	}

	fields := desc.Fields()

	for len(bz) > 0 {
		tagNum, wireType, m := protowire.ConsumeTag(bz)
		if m < 0 {
			return hasUnknownNonCriticals, errors.New("invalid length")
		}

		fieldDesc := fields.ByNumber(tagNum)
		if fieldDesc == nil {
			isCriticalField := tagNum&bit11NonCritical == 0

			if !isCriticalField {
				hasUnknownNonCriticals = true
			}

			if isCriticalField || !allowUnknownNonCriticals {
				// The tag is critical, so report it.
				return hasUnknownNonCriticals, ErrUnknownField.Wrapf(
					"%s: {TagNum: %d, WireType:%q}",
					desc.FullName(), tagNum, WireTypeToString(wireType))
			}
		}

		// Skip over the bytes that store fieldNumber and wireType bytes.
		bz = bz[m:]
		n := protowire.ConsumeFieldValue(tagNum, wireType, bz)
		if n < 0 {
			err = fmt.Errorf("could not consume field value for tagNum: %d, wireType: %q; %w",
				tagNum, WireTypeToString(wireType), protowire.ParseError(n))
			return hasUnknownNonCriticals, err
		}
		fieldBytes := bz[:n]
		bz = bz[n:]

		// An unknown but non-critical field
		if fieldDesc == nil {
			continue
		}

		fieldMessage := fieldDesc.Message()
		// not message or group kind
		if fieldMessage == nil {
			continue
		}

		// consume length prefix of nested message
		_, o := protowire.ConsumeVarint(fieldBytes)
		if o < 0 {
			err = fmt.Errorf("could not consume length prefix fieldBytes for nested message: %v: %w",
				fieldMessage, protowire.ParseError(o))
			return hasUnknownNonCriticals, err
		} else if o > len(fieldBytes) {
			err = fmt.Errorf("length prefix > len(fieldBytes) for nested message: %v", fieldMessage)
			return hasUnknownNonCriticals, err
		}

		fieldBytes = fieldBytes[o:]

		var err error

		if fieldMessage.FullName() == anyFullName {
			// Firstly typecheck types.Any to ensure nothing snuck in.
			hasUnknownNonCriticalsChild, err := RejectUnknownFields(fieldBytes, anyDesc, allowUnknownNonCriticals, resolver)
			hasUnknownNonCriticals = hasUnknownNonCriticals || hasUnknownNonCriticalsChild
			if err != nil {
				return hasUnknownNonCriticals, err
			}
			var a anypb.Any
			if err = proto.Unmarshal(fieldBytes, &a); err != nil {
				return hasUnknownNonCriticals, err
			}

			msgName := protoreflect.FullName(strings.TrimPrefix(a.TypeUrl, "/"))
			msgDesc, err := resolver.FindDescriptorByName(msgName)
			if err != nil {
				return hasUnknownNonCriticals, err
			}

			fieldMessage = msgDesc.(protoreflect.MessageDescriptor)
			fieldBytes = a.Value
		}

		hasUnknownNonCriticalsChild, err := RejectUnknownFields(fieldBytes, fieldMessage, allowUnknownNonCriticals, resolver)
		hasUnknownNonCriticals = hasUnknownNonCriticals || hasUnknownNonCriticalsChild
		if err != nil {
			return hasUnknownNonCriticals, err
		}
	}

	return hasUnknownNonCriticals, nil
}

// errUnknownField represents an error indicating that we encountered
// a field that isn't available in the target proto.Message.
type errUnknownField struct {
	Desc     protoreflect.MessageDescriptor
	TagNum   protowire.Number
	WireType protowire.Type
}

// String implements fmt.Stringer.
func (twt *errUnknownField) String() string {
	return fmt.Sprintf("errUnknownField %q: {TagNum: %d, WireType:%q}",
		twt.Desc.FullName(), twt.TagNum, WireTypeToString(twt.WireType))
}

// Error implements the error interface.
func (twt *errUnknownField) Error() string {
	return twt.String()
}

var _ error = (*errUnknownField)(nil)

// WireTypeToString returns a string representation of the given protowire.Type.
func WireTypeToString(wt protowire.Type) string {
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
