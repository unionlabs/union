package codec

import (
	"encoding/binary"
	"errors"
	"fmt"
	"strings"

	"github.com/cosmos/cosmos-proto/anyutil"
	"github.com/cosmos/gogoproto/jsonpb"
	gogoproto "github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc/encoding"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"
	"google.golang.org/protobuf/types/dynamicpb"
	"google.golang.org/protobuf/types/known/anypb"

	"cosmossdk.io/x/tx/signing/aminojson"

	"github.com/cosmos/cosmos-sdk/codec/types"
)

// ProtoCodecMarshaler defines an interface for codecs that utilize Protobuf for both
// binary and JSON encoding.
// Deprecated: Use Codec instead.
type ProtoCodecMarshaler interface {
	Codec
}

// ProtoCodec defines a codec that utilizes Protobuf for both binary and JSON
// encoding.
type ProtoCodec struct {
	interfaceRegistry types.InterfaceRegistry
}

var _ Codec = (*ProtoCodec)(nil)

// NewProtoCodec returns a reference to a new ProtoCodec
func NewProtoCodec(interfaceRegistry types.InterfaceRegistry) *ProtoCodec {
	return &ProtoCodec{
		interfaceRegistry: interfaceRegistry,
	}
}

// Marshal implements BinaryMarshaler.Marshal method.
// NOTE: this function must be used with a concrete type which
// implements proto.Message. For interface please use the codec.MarshalInterface
func (pc *ProtoCodec) Marshal(o gogoproto.Message) ([]byte, error) {
	// Size() check can catch the typed nil value.
	if o == nil || gogoproto.Size(o) == 0 {
		// return empty bytes instead of nil, because nil has special meaning in places like store.Set
		return []byte{}, nil
	}

	return gogoproto.Marshal(o)
}

// MustMarshal implements BinaryMarshaler.MustMarshal method.
// NOTE: this function must be used with a concrete type which
// implements proto.Message. For interface please use the codec.MarshalInterface
func (pc *ProtoCodec) MustMarshal(o gogoproto.Message) []byte {
	bz, err := pc.Marshal(o)
	if err != nil {
		panic(err)
	}

	return bz
}

// MarshalLengthPrefixed implements BinaryMarshaler.MarshalLengthPrefixed method.
func (pc *ProtoCodec) MarshalLengthPrefixed(o gogoproto.Message) ([]byte, error) {
	bz, err := pc.Marshal(o)
	if err != nil {
		return nil, err
	}

	var sizeBuf [binary.MaxVarintLen64]byte
	n := binary.PutUvarint(sizeBuf[:], uint64(len(bz)))
	return append(sizeBuf[:n], bz...), nil
}

// MustMarshalLengthPrefixed implements BinaryMarshaler.MustMarshalLengthPrefixed method.
func (pc *ProtoCodec) MustMarshalLengthPrefixed(o gogoproto.Message) []byte {
	bz, err := pc.MarshalLengthPrefixed(o)
	if err != nil {
		panic(err)
	}

	return bz
}

// Unmarshal implements BinaryMarshaler.Unmarshal method.
// NOTE: this function must be used with a concrete type which
// implements proto.Message. For interface please use the codec.UnmarshalInterface
func (pc *ProtoCodec) Unmarshal(bz []byte, ptr gogoproto.Message) error {
	err := gogoproto.Unmarshal(bz, ptr)
	if err != nil {
		return err
	}
	err = types.UnpackInterfaces(ptr, pc.interfaceRegistry)
	if err != nil {
		return err
	}
	return nil
}

// MustUnmarshal implements BinaryMarshaler.MustUnmarshal method.
// NOTE: this function must be used with a concrete type which
// implements proto.Message. For interface please use the codec.UnmarshalInterface
func (pc *ProtoCodec) MustUnmarshal(bz []byte, ptr gogoproto.Message) {
	if err := pc.Unmarshal(bz, ptr); err != nil {
		panic(err)
	}
}

// UnmarshalLengthPrefixed implements BinaryMarshaler.UnmarshalLengthPrefixed method.
func (pc *ProtoCodec) UnmarshalLengthPrefixed(bz []byte, ptr gogoproto.Message) error {
	size, n := binary.Uvarint(bz)
	if n < 0 {
		return fmt.Errorf("invalid number of bytes read from length-prefixed encoding: %d", n)
	}

	if size > uint64(len(bz)-n) {
		return fmt.Errorf("not enough bytes to read; want: %v, got: %v", size, len(bz)-n)
	} else if size < uint64(len(bz)-n) {
		return fmt.Errorf("too many bytes to read; want: %v, got: %v", size, len(bz)-n)
	}

	bz = bz[n:]
	return pc.Unmarshal(bz, ptr)
}

// MustUnmarshalLengthPrefixed implements BinaryMarshaler.MustUnmarshalLengthPrefixed method.
func (pc *ProtoCodec) MustUnmarshalLengthPrefixed(bz []byte, ptr gogoproto.Message) {
	if err := pc.UnmarshalLengthPrefixed(bz, ptr); err != nil {
		panic(err)
	}
}

// MarshalJSON implements JSONCodec.MarshalJSON method,
// it marshals to JSON using proto codec.
// NOTE: this function must be used with a concrete type which
// implements proto.Message. For interface please use the codec.MarshalInterfaceJSON
func (pc *ProtoCodec) MarshalJSON(o gogoproto.Message) ([]byte, error) { //nolint:stdmethods // we don't want to implement Marshaler interface
	if o == nil {
		return nil, errors.New("cannot protobuf JSON encode nil")
	}
	return ProtoMarshalJSON(o, pc.interfaceRegistry)
}

// MustMarshalJSON implements JSONCodec.MustMarshalJSON method,
// it executes MarshalJSON except it panics upon failure.
// NOTE: this function must be used with a concrete type which
// implements proto.Message. For interface please use the codec.MarshalInterfaceJSON
func (pc *ProtoCodec) MustMarshalJSON(o gogoproto.Message) []byte {
	bz, err := pc.MarshalJSON(o)
	if err != nil {
		panic(err)
	}

	return bz
}

// MarshalAminoJSON provides aminojson.Encoder compatibility for gogoproto messages.
// x/tx/signing/aminojson cannot marshal gogoproto messages directly since this type does not implement
// the standard library google.golang.org/protobuf/proto.Message.
// We convert gogo types to dynamicpb messages and then marshal that directly to amino JSON.
func (pc *ProtoCodec) MarshalAminoJSON(msg gogoproto.Message) ([]byte, error) {
	encoder := aminojson.NewEncoder(aminojson.EncoderOptions{FileResolver: pc.interfaceRegistry})
	gogoBytes, err := gogoproto.Marshal(msg)
	if err != nil {
		return nil, err
	}

	var protoMsg protoreflect.ProtoMessage
	typ, err := protoregistry.GlobalTypes.FindMessageByURL(fmt.Sprintf("/%s", gogoproto.MessageName(msg)))
	if typ != nil && err != nil {
		protoMsg = typ.New().Interface()
	} else {
		desc, err := pc.interfaceRegistry.FindDescriptorByName(protoreflect.FullName(gogoproto.MessageName(msg)))
		if err != nil {
			return nil, err
		}
		dynamicMsgType := dynamicpb.NewMessageType(desc.(protoreflect.MessageDescriptor))
		protoMsg = dynamicMsgType.New().Interface()
	}

	err = proto.Unmarshal(gogoBytes, protoMsg)
	if err != nil {
		return nil, err
	}
	return encoder.Marshal(protoMsg)
}

// UnmarshalJSON implements JSONCodec.UnmarshalJSON method,
// it unmarshals from JSON using proto codec.
// NOTE: this function must be used with a concrete type which
// implements proto.Message. For interface please use the codec.UnmarshalInterfaceJSON
func (pc *ProtoCodec) UnmarshalJSON(bz []byte, ptr gogoproto.Message) error {
	if ptr == nil {
		return fmt.Errorf("cannot protobuf JSON decode unsupported type: %T", ptr)
	}
	unmarshaler := jsonpb.Unmarshaler{AnyResolver: pc.interfaceRegistry}
	err := unmarshaler.Unmarshal(strings.NewReader(string(bz)), ptr)
	if err != nil {
		return err
	}

	return types.UnpackInterfaces(ptr, pc.interfaceRegistry)
}

// MustUnmarshalJSON implements JSONCodec.MustUnmarshalJSON method,
// it executes UnmarshalJSON except it panics upon failure.
// NOTE: this function must be used with a concrete type which
// implements proto.Message. For interface please use the codec.UnmarshalInterfaceJSON
func (pc *ProtoCodec) MustUnmarshalJSON(bz []byte, ptr gogoproto.Message) {
	if err := pc.UnmarshalJSON(bz, ptr); err != nil {
		panic(err)
	}
}

// MarshalInterface is a convenience function for proto marshaling interfaces. It packs
// the provided value, which must be an interface, in an Any and then marshals it to bytes.
// NOTE: to marshal a concrete type, you should use Marshal instead
func (pc *ProtoCodec) MarshalInterface(i gogoproto.Message) ([]byte, error) {
	if err := assertNotNil(i); err != nil {
		return nil, err
	}
	any, err := types.NewAnyWithValue(i)
	if err != nil {
		return nil, err
	}
	err = pc.interfaceRegistry.EnsureRegistered(i)
	if err != nil {
		return nil, err
	}

	return pc.Marshal(any)
}

// UnmarshalInterface is a convenience function for proto unmarshaling interfaces. It
// unmarshals an Any from bz bytes and then unpacks it to the `ptr`, which must
// be a pointer to a non empty interface with registered implementations.
// NOTE: to unmarshal a concrete type, you should use Unmarshal instead
//
// Example:
//
//	var x MyInterface
//	err := cdc.UnmarshalInterface(bz, &x)
func (pc *ProtoCodec) UnmarshalInterface(bz []byte, ptr interface{}) error {
	any := &types.Any{}
	err := pc.Unmarshal(bz, any)
	if err != nil {
		return err
	}

	return pc.UnpackAny(any, ptr)
}

// MarshalInterfaceJSON is a convenience function for proto marshaling interfaces. It
// packs the provided value in an Any and then marshals it to bytes.
// NOTE: to marshal a concrete type, you should use MarshalJSON instead
func (pc *ProtoCodec) MarshalInterfaceJSON(x gogoproto.Message) ([]byte, error) {
	any, err := types.NewAnyWithValue(x)
	if err != nil {
		return nil, err
	}
	return pc.MarshalJSON(any)
}

// UnmarshalInterfaceJSON is a convenience function for proto unmarshaling interfaces.
// It unmarshals an Any from bz bytes and then unpacks it to the `iface`, which must
// be a pointer to a non empty interface, implementing proto.Message with registered implementations.
// NOTE: to unmarshal a concrete type, you should use UnmarshalJSON instead
//
// Example:
//
//	var x MyInterface  // must implement proto.Message
//	err := cdc.UnmarshalInterfaceJSON(&x, bz)
func (pc *ProtoCodec) UnmarshalInterfaceJSON(bz []byte, iface interface{}) error {
	any := &types.Any{}
	err := pc.UnmarshalJSON(bz, any)
	if err != nil {
		return err
	}
	return pc.UnpackAny(any, iface)
}

// UnpackAny implements AnyUnpacker.UnpackAny method,
// it unpacks the value in any to the interface pointer passed in as
// iface.
func (pc *ProtoCodec) UnpackAny(any *types.Any, iface interface{}) error {
	return pc.interfaceRegistry.UnpackAny(any, iface)
}

// InterfaceRegistry returns InterfaceRegistry
func (pc *ProtoCodec) InterfaceRegistry() types.InterfaceRegistry {
	return pc.interfaceRegistry
}

func (pc ProtoCodec) GetMsgAnySigners(msg *types.Any) ([][]byte, protoreflect.Message, error) {
	msgv2, err := anyutil.Unpack(&anypb.Any{
		TypeUrl: msg.TypeUrl,
		Value:   msg.Value,
	}, pc.interfaceRegistry, nil)
	if err != nil {
		return nil, nil, err
	}

	signers, err := pc.interfaceRegistry.SigningContext().GetSigners(msgv2)
	return signers, msgv2.ProtoReflect(), err
}

func (pc *ProtoCodec) GetReflectMsgSigners(msg protoreflect.Message) ([][]byte, error) {
	return pc.interfaceRegistry.SigningContext().GetSigners(msg.Interface())
}

func (pc *ProtoCodec) GetMsgSigners(msg gogoproto.Message) ([][]byte, protoreflect.Message, error) {
	if msgV2, ok := msg.(proto.Message); ok {
		signers, err := pc.interfaceRegistry.SigningContext().GetSigners(msgV2)
		return signers, msgV2.ProtoReflect(), err
	}
	a, err := types.NewAnyWithValue(msg)
	if err != nil {
		return nil, nil, err
	}
	return pc.GetMsgAnySigners(a)
}

// GRPCCodec returns the gRPC Codec for this specific ProtoCodec
func (pc *ProtoCodec) GRPCCodec() encoding.Codec {
	return &grpcProtoCodec{cdc: pc}
}

func (pc *ProtoCodec) mustEmbedCodec() {}

var errUnknownProtoType = errors.New("codec: unknown proto type") // sentinel error

// grpcProtoCodec is the implementation of the gRPC proto codec.
type grpcProtoCodec struct {
	cdc *ProtoCodec
}

func (g grpcProtoCodec) Marshal(v interface{}) ([]byte, error) {
	switch m := v.(type) {
	case proto.Message:
		protov2MarshalOpts := proto.MarshalOptions{Deterministic: true}
		return protov2MarshalOpts.Marshal(m)
	case gogoproto.Message:
		return g.cdc.Marshal(m)
	default:
		return nil, fmt.Errorf("%w: cannot marshal type %T", errUnknownProtoType, v)
	}
}

func (g grpcProtoCodec) Unmarshal(data []byte, v interface{}) error {
	switch m := v.(type) {
	case proto.Message:
		return proto.Unmarshal(data, m)
	case gogoproto.Message:
		return g.cdc.Unmarshal(data, m)
	default:
		return fmt.Errorf("%w: cannot unmarshal type %T", errUnknownProtoType, v)
	}
}

func (g grpcProtoCodec) Name() string {
	return "cosmos-sdk-grpc-codec"
}

func assertNotNil(i interface{}) error {
	if i == nil {
		return errors.New("can't marshal <nil> value")
	}
	return nil
}
