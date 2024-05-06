package textual

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"reflect"

	cosmos_proto "github.com/cosmos/cosmos-proto"
	gogoproto "github.com/cosmos/gogoproto/proto"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/timestamppb"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	signingv1beta1 "cosmossdk.io/api/cosmos/tx/signing/v1beta1"
	"cosmossdk.io/x/tx/signing"
	"cosmossdk.io/x/tx/signing/textual/internal/textualpb"
)

const specVersion = 0

// CoinMetadataQueryFn defines a function that queries state for the coin denom
// metadata. It is meant to be passed as an argument into `NewSignModeHandler`.
type CoinMetadataQueryFn func(ctx context.Context, denom string) (*bankv1beta1.Metadata, error)

// ValueRendererCreator is a function returning a textual.
type ValueRendererCreator func(protoreflect.FieldDescriptor) ValueRenderer

// SignModeOptions are options to be passed to Textual's sign mode handler.
type SignModeOptions struct {
	// coinMetadataQuerier defines a function to query the coin metadata from
	// state. It should use bank module's `DenomsMetadata` gRPC query to fetch
	// each denom's associated metadata, either using the bank keeper (for
	// server-side code) or a gRPC query client (for client-side code).
	CoinMetadataQuerier CoinMetadataQueryFn

	// FileResolver are the protobuf files to use for resolving message
	// descriptors. If it is nil, the global protobuf registry will be used.
	FileResolver signing.ProtoFileResolver

	// TypeResolver are the protobuf type resolvers to use for resolving message
	// types. If it is nil, then a dynamicpb will be used on top of FileResolver.
	TypeResolver protoregistry.MessageTypeResolver
}

// SignModeHandler holds the configuration for dispatching
// to specific value renderers for SIGN_MODE_TEXTUAL.
type SignModeHandler struct {
	fileResolver        signing.ProtoFileResolver
	typeResolver        protoregistry.MessageTypeResolver
	coinMetadataQuerier CoinMetadataQueryFn
	// scalars defines a registry for Cosmos scalars.
	scalars map[string]ValueRendererCreator
	// messages defines a registry for custom message renderers.
	// Note that we also use this same registry for the
	// following messages, as they can be thought of custom message rendering:
	// - SDK coin and coins
	// - Protobuf timestamp
	// - Protobuf duration
	messages map[protoreflect.FullName]ValueRenderer
}

// NewSignModeHandler returns a new SignModeHandler which generates sign bytes and provides  value renderers.
func NewSignModeHandler(o SignModeOptions) (*SignModeHandler, error) {
	if o.CoinMetadataQuerier == nil {
		return nil, errors.New("coinMetadataQuerier must be non-empty")
	}
	if o.FileResolver == nil {
		o.FileResolver = gogoproto.HybridResolver
	}
	if o.TypeResolver == nil {
		o.TypeResolver = protoregistry.GlobalTypes
	}

	t := &SignModeHandler{
		coinMetadataQuerier: o.CoinMetadataQuerier,
		fileResolver:        o.FileResolver,
		typeResolver:        o.TypeResolver,
	}
	t.init()

	return t, nil
}

// SpecVersion returns the spec version this SignModeHandler implementation
// is following.
func (r *SignModeHandler) SpecVersion() uint64 {
	return specVersion
}

// GetFieldValueRenderer returns the value renderer for the given FieldDescriptor.
func (r *SignModeHandler) GetFieldValueRenderer(fd protoreflect.FieldDescriptor) (ValueRenderer, error) {
	switch {
	// Scalars, such as math.Int and math.Dec encoded as strings.
	case fd.Kind() == protoreflect.StringKind:
		if proto.GetExtension(fd.Options(), cosmos_proto.E_Scalar) != "" {
			scalar, ok := proto.GetExtension(fd.Options(), cosmos_proto.E_Scalar).(string)
			if !ok || scalar == "" {
				return nil, fmt.Errorf("got extension option %s of type %T", scalar, scalar)
			}

			vr := r.scalars[scalar]
			if vr != nil {
				return vr(fd), nil
			}
		}

		return NewStringValueRenderer(), nil

	case fd.Kind() == protoreflect.BytesKind:
		return NewBytesValueRenderer(), nil

	// Integers
	case fd.Kind() == protoreflect.Uint32Kind ||
		fd.Kind() == protoreflect.Uint64Kind ||
		fd.Kind() == protoreflect.Int32Kind ||
		fd.Kind() == protoreflect.Int64Kind:
		return NewIntValueRenderer(fd), nil

	case fd.Kind() == protoreflect.EnumKind:
		return NewEnumValueRenderer(fd), nil

	case fd.Kind() == protoreflect.MessageKind:
		md := fd.Message()
		fullName := md.FullName()

		vr, found := r.messages[fullName]
		if found {
			return vr, nil
		}

		if fd.IsMap() {
			return nil, errors.New("value renderers cannot format value of type map")
		}
		return NewMessageValueRenderer(r, md), nil
	case fd.Kind() == protoreflect.BoolKind:
		return NewBoolValueRenderer(), nil

	default:
		return nil, fmt.Errorf("value renderers cannot format value of type %s", fd.Kind())
	}
}

// GetMessageValueRenderer returns a value renderer for a message.
// It is useful when the message type is discovered outside the context of a field,
// e.g. when handling a google.protobuf.Any.
func (r *SignModeHandler) GetMessageValueRenderer(md protoreflect.MessageDescriptor) (ValueRenderer, error) {
	fullName := md.FullName()
	vr, found := r.messages[fullName]
	if found {
		return vr, nil
	}
	return NewMessageValueRenderer(r, md), nil
}

// init initializes Textual's internal `scalars` and `messages` registry for
// custom scalar and message renderers.
//
// It is an idempotent method.
func (r *SignModeHandler) init() {
	if r.scalars == nil {
		r.scalars = map[string]ValueRendererCreator{}
		r.scalars["cosmos.Int"] = NewIntValueRenderer
		r.scalars["cosmos.Dec"] = func(_ protoreflect.FieldDescriptor) ValueRenderer { return NewDecValueRenderer() }
	}
	if r.messages == nil {
		r.messages = map[protoreflect.FullName]ValueRenderer{}
		r.messages[(&basev1beta1.Coin{}).ProtoReflect().Descriptor().FullName()] = NewCoinsValueRenderer(r.coinMetadataQuerier)
		r.messages[(&durationpb.Duration{}).ProtoReflect().Descriptor().FullName()] = NewDurationValueRenderer()
		r.messages[(&timestamppb.Timestamp{}).ProtoReflect().Descriptor().FullName()] = NewTimestampValueRenderer()
		r.messages[(&anypb.Any{}).ProtoReflect().Descriptor().FullName()] = NewAnyValueRenderer(r)
		r.messages[(&textualpb.TextualData{}).ProtoReflect().Descriptor().FullName()] = NewTxValueRenderer(r)
	}
}

// DefineScalar adds a value renderer to the given Cosmos scalar.
func (r *SignModeHandler) DefineScalar(scalar string, vr ValueRendererCreator) {
	r.init()
	r.scalars[scalar] = vr
}

// DefineMessageRenderer adds a new custom message renderer.
func (r *SignModeHandler) DefineMessageRenderer(name protoreflect.FullName, vr ValueRenderer) {
	r.init()
	r.messages[name] = vr
}

// GetSignBytes returns the transaction sign bytes which is the CBOR representation
// of a list of screens created from the TX data.
func (r *SignModeHandler) GetSignBytes(ctx context.Context, signerData signing.SignerData, txData signing.TxData) ([]byte, error) {
	data := &textualpb.TextualData{
		BodyBytes:     txData.BodyBytes,
		AuthInfoBytes: txData.AuthInfoBytes,
		SignerData: &textualpb.SignerData{
			Address:       signerData.Address,
			ChainId:       signerData.ChainID,
			AccountNumber: signerData.AccountNumber,
			Sequence:      signerData.Sequence,
			PubKey:        signerData.PubKey,
		},
	}

	screens, err := NewTxValueRenderer(r).Format(ctx, protoreflect.ValueOf(data.ProtoReflect()))
	if err != nil {
		return nil, err
	}

	var buf bytes.Buffer
	err = encode(screens, &buf)
	if err != nil {
		return nil, err
	}

	return buf.Bytes(), nil
}

func (r *SignModeHandler) Mode() signingv1beta1.SignMode {
	return signingv1beta1.SignMode_SIGN_MODE_TEXTUAL
}

var _ signing.SignModeHandler = &SignModeHandler{}

// getValueFromFieldName is an utility function to get the protoreflect.Value of a
// proto Message from its field name.
func getValueFromFieldName(m proto.Message, fieldName string) protoreflect.Value {
	fd := m.ProtoReflect().Descriptor().Fields().ByName(protoreflect.Name(fieldName))

	return m.ProtoReflect().Get(fd)
}

// coerceToMessage initializes the given desiredMsg (presented as a protov2
// concrete message) with the values of givenMsg.
//
// If givenMsg is a protov2 concrete message of the same type, then it will
// fast-path to be initialized to the same pointer value.
// For a dynamicpb message it checks that the names match then uses proto
// reflection to initialize the fields of desiredMsg.
// Otherwise throws an error.
//
// Example:
//
// // Assume `givenCoin` is a dynamicpb.Message representing a Coin
// coin := &basev1beta1.Coin{}
// err := coerceToMessage(givenCoin, coin)
// if err != nil { /* handler error */ }
// fmt.Println(coin) // Will have the same field values as `givenCoin`
func coerceToMessage(givenMsg, desiredMsg proto.Message) error {
	if reflect.TypeOf(givenMsg) == reflect.TypeOf(desiredMsg) {
		// Below is a way of saying "*desiredMsg = *givenMsg" using go reflect
		reflect.Indirect(reflect.ValueOf(desiredMsg)).Set(reflect.Indirect(reflect.ValueOf(givenMsg)))
		return nil
	}

	givenName, desiredName := givenMsg.ProtoReflect().Descriptor().FullName(), desiredMsg.ProtoReflect().Descriptor().FullName()
	if givenName != desiredName {
		return fmt.Errorf("expected dynamicpb.Message with FullName %s, got %s", desiredName, givenName)
	}

	desiredFields := desiredMsg.ProtoReflect().Descriptor().Fields()
	for i := 0; i < desiredFields.Len(); i++ {
		fd := desiredFields.Get(i)
		desiredMsg.ProtoReflect().Set(fd, getValueFromFieldName(givenMsg, string(fd.Name())))
	}

	return nil
}
