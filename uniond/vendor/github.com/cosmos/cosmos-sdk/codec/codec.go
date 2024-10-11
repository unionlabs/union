package codec

import (
	"github.com/cosmos/gogoproto/proto"
	gogoprotoany "github.com/cosmos/gogoproto/types/any"
	"google.golang.org/grpc/encoding"
	"google.golang.org/protobuf/reflect/protoreflect"

	"github.com/cosmos/cosmos-sdk/codec/types"
)

type (
	// Codec defines a functionality for serializing other objects.
	// Users can define a custom Protobuf-based serialization.
	// Note, Amino can still be used without any dependency on Protobuf.
	// SDK provides to Codec implementations:
	//
	// 1. AminoCodec: Provides full Amino serialization compatibility.
	// 2. ProtoCodec: Provides full Protobuf serialization compatibility.
	Codec interface {
		BinaryCodec
		JSONCodec

		// InterfaceRegistry returns the interface registry.
		InterfaceRegistry() types.InterfaceRegistry

		// GetMsgAnySigners returns the signers of the given message encoded in a protobuf Any
		// as well as the decoded protoreflect.Message that was used to extract the
		// signers so that this can be used in other context where proto reflection
		// is needed.
		GetMsgAnySigners(msg *types.Any) ([][]byte, protoreflect.Message, error)

		// GetMsgSigners returns the signers of the given message plus the
		// decoded protoreflect.Message that was used to extract the
		// signers so that this can be used in other context where proto reflection
		// is needed.
		GetMsgSigners(msg proto.Message) ([][]byte, protoreflect.Message, error)

		// GetReflectMsgSigners returns the signers of the given reflected proto message.
		GetReflectMsgSigners(msg protoreflect.Message) ([][]byte, error)

		// mustEmbedCodec requires that all implementations of Codec embed an official implementation from the codec
		// package. This allows new methods to be added to the Codec interface without breaking backwards compatibility.
		mustEmbedCodec()
	}

	BinaryCodec interface {
		// Marshal returns binary encoding of v.
		Marshal(o proto.Message) ([]byte, error)
		// MustMarshal calls Marshal and panics if error is returned.
		MustMarshal(o proto.Message) []byte

		// MarshalLengthPrefixed returns binary encoding of v with bytes length prefix.
		MarshalLengthPrefixed(o proto.Message) ([]byte, error)
		// MustMarshalLengthPrefixed calls MarshalLengthPrefixed and panics if
		// error is returned.
		MustMarshalLengthPrefixed(o proto.Message) []byte

		// Unmarshal parses the data encoded with Marshal method and stores the result
		// in the value pointed to by v.
		Unmarshal(bz []byte, ptr proto.Message) error
		// MustUnmarshal calls Unmarshal and panics if error is returned.
		MustUnmarshal(bz []byte, ptr proto.Message)

		// UnmarshalLengthPrefixed parses the data encoded with UnmarshalLengthPrefixed method and stores
		// the result in the value pointed to by v.
		UnmarshalLengthPrefixed(bz []byte, ptr proto.Message) error
		// MustUnmarshalLengthPrefixed calls UnmarshalLengthPrefixed and panics if error
		// is returned.
		MustUnmarshalLengthPrefixed(bz []byte, ptr proto.Message)

		// MarshalInterface is a helper method which will wrap `i` into `Any` for correct
		// binary interface (de)serialization.
		MarshalInterface(i proto.Message) ([]byte, error)
		// UnmarshalInterface is a helper method which will parse binary enoded data
		// into `Any` and unpack any into the `ptr`. It fails if the target interface type
		// is not registered in codec, or is not compatible with the serialized data
		UnmarshalInterface(bz []byte, ptr interface{}) error

		gogoprotoany.AnyUnpacker
	}

	JSONCodec interface {
		// MarshalJSON returns JSON encoding of v.
		MarshalJSON(o proto.Message) ([]byte, error)
		// MustMarshalJSON calls MarshalJSON and panics if error is returned.
		MustMarshalJSON(o proto.Message) []byte
		// MarshalInterfaceJSON is a helper method which will wrap `i` into `Any` for correct
		// JSON interface (de)serialization.
		MarshalInterfaceJSON(i proto.Message) ([]byte, error)
		// UnmarshalInterfaceJSON is a helper method which will parse JSON enoded data
		// into `Any` and unpack any into the `ptr`. It fails if the target interface type
		// is not registered in codec, or is not compatible with the serialized data
		UnmarshalInterfaceJSON(bz []byte, ptr interface{}) error

		// UnmarshalJSON parses the data encoded with MarshalJSON method and stores the result
		// in the value pointed to by v.
		UnmarshalJSON(bz []byte, ptr proto.Message) error
		// MustUnmarshalJSON calls Unmarshal and panics if error is returned.
		MustUnmarshalJSON(bz []byte, ptr proto.Message)
	}

	// ProtoMarshaler defines an interface a type must implement to serialize itself
	// as a protocol buffer defined message.
	//
	// Deprecated: Use proto.Message instead from github.com/cosmos/gogoproto/proto.
	ProtoMarshaler interface {
		proto.Message // for JSON serialization

		Marshal() ([]byte, error)
		MarshalTo(data []byte) (n int, err error)
		MarshalToSizedBuffer(dAtA []byte) (int, error)
		Size() int
		Unmarshal(data []byte) error
	}

	// AminoMarshaler defines an interface a type must implement to serialize itself
	// for Amino codec.
	AminoMarshaler interface {
		MarshalAmino() ([]byte, error)
		UnmarshalAmino([]byte) error
		MarshalAminoJSON() ([]byte, error)
		UnmarshalAminoJSON([]byte) error
	}

	// GRPCCodecProvider is implemented by the Codec
	// implementations which return a gRPC encoding.Codec.
	// And it is used to decode requests and encode responses
	// passed through gRPC.
	GRPCCodecProvider interface {
		GRPCCodec() encoding.Codec
	}
)
