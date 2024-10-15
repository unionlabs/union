package protocompat

import (
	"context"
	"fmt"
	"reflect"

	gogoproto "github.com/cosmos/gogoproto/proto"
	"github.com/golang/protobuf/proto" //nolint: staticcheck // needed because gogoproto.Merge does not work consistently. See NOTE: comments.
	"google.golang.org/grpc"
	proto2 "google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"
	"google.golang.org/protobuf/runtime/protoiface"

	"github.com/cosmos/cosmos-sdk/codec"
)

var (
	gogoType           = reflect.TypeOf((*gogoproto.Message)(nil)).Elem()
	protov2Type        = reflect.TypeOf((*proto2.Message)(nil)).Elem()
	protov2MarshalOpts = proto2.MarshalOptions{Deterministic: true}
)

type Handler = func(ctx context.Context, request, response protoiface.MessageV1) error

// MakeHybridHandler returns a handler that can handle both gogo and protov2 messages, no matter
// if the handler is a gogo or protov2 handler.
func MakeHybridHandler(cdc codec.BinaryCodec, sd *grpc.ServiceDesc, method grpc.MethodDesc, handler interface{}) (Handler, error) {
	methodFullName := protoreflect.FullName(fmt.Sprintf("%s.%s", sd.ServiceName, method.MethodName))
	desc, err := gogoproto.HybridResolver.FindDescriptorByName(methodFullName)
	if err != nil {
		return nil, err
	}
	methodDesc, ok := desc.(protoreflect.MethodDescriptor)
	if !ok {
		return nil, fmt.Errorf("invalid method descriptor %s", methodFullName)
	}

	isProtov2Handler, err := isProtov2(method)
	if err != nil {
		return nil, err
	}
	if isProtov2Handler {
		return makeProtoV2HybridHandler(methodDesc, cdc, method, handler)
	}
	return makeGogoHybridHandler(methodDesc, cdc, method, handler)
}

// makeProtoV2HybridHandler returns a handler that can handle both gogo and protov2 messages.
func makeProtoV2HybridHandler(prefMethod protoreflect.MethodDescriptor, cdc codec.BinaryCodec, method grpc.MethodDesc, handler any) (Handler, error) {
	// it's a protov2 handler, if a gogo counterparty is not found we cannot handle gogo messages.
	gogoExists := gogoproto.MessageType(string(prefMethod.Output().FullName())) != nil
	if !gogoExists {
		return func(ctx context.Context, inReq, outResp protoiface.MessageV1) error {
			protov2Request, ok := inReq.(proto2.Message)
			if !ok {
				return fmt.Errorf("invalid request type %T, method %s does not accept gogoproto messages", inReq, prefMethod.FullName())
			}
			resp, err := method.Handler(handler, ctx, func(msg any) error {
				proto2.Merge(msg.(proto2.Message), protov2Request)
				return nil
			}, nil)
			if err != nil {
				return err
			}
			// merge on the resp
			proto2.Merge(outResp.(proto2.Message), resp.(proto2.Message))
			return nil
		}, nil
	}
	return func(ctx context.Context, inReq, outResp protoiface.MessageV1) error {
		// we check if the request is a protov2 message.
		switch m := inReq.(type) {
		case proto2.Message:
			// we can just call the handler after making a copy of the message, for safety reasons.
			resp, err := method.Handler(handler, ctx, func(msg any) error {
				proto2.Merge(msg.(proto2.Message), m)
				return nil
			}, nil)
			if err != nil {
				return err
			}
			// merge on the resp
			proto2.Merge(outResp.(proto2.Message), resp.(proto2.Message))
			return nil
		case gogoproto.Message:
			// we need to marshal and unmarshal the request.
			requestBytes, err := cdc.Marshal(m)
			if err != nil {
				return err
			}
			resp, err := method.Handler(handler, ctx, func(msg any) error {
				// unmarshal request into the message.
				return proto2.Unmarshal(requestBytes, msg.(proto2.Message))
			}, nil)
			if err != nil {
				return err
			}
			// the response is a protov2 message, so we cannot just return it.
			// since the request came as gogoproto, we expect the response
			// to also be gogoproto.
			respBytes, err := protov2MarshalOpts.Marshal(resp.(proto2.Message))
			if err != nil {
				return err
			}

			// unmarshal response into a gogo message.
			return cdc.Unmarshal(respBytes, outResp.(gogoproto.Message))
		default:
			panic("unreachable")
		}
	}, nil
}

func makeGogoHybridHandler(prefMethod protoreflect.MethodDescriptor, cdc codec.BinaryCodec, method grpc.MethodDesc, handler any) (Handler, error) {
	// it's a gogo handler, we check if the existing protov2 counterparty exists.
	_, err := protoregistry.GlobalTypes.FindMessageByName(prefMethod.Output().FullName())
	if err != nil {
		// this can only be a gogo message.
		return func(ctx context.Context, inReq, outResp protoiface.MessageV1) error {
			_, ok := inReq.(proto2.Message)
			if ok {
				return fmt.Errorf("invalid request type %T, method %s does not accept protov2 messages", inReq, prefMethod.FullName())
			}
			resp, err := method.Handler(handler, ctx, func(msg any) error {
				// merge! ref: https://github.com/cosmos/cosmos-sdk/issues/18003
				// NOTE: using gogoproto.Merge will fail for some reason unknown to me, but
				// using proto.Merge with gogo messages seems to work fine.
				proto.Merge(msg.(gogoproto.Message), inReq)
				return nil
			}, nil)
			if err != nil {
				return err
			}
			// merge resp, ref: https://github.com/cosmos/cosmos-sdk/issues/18003
			// NOTE: using gogoproto.Merge will fail for some reason unknown to me, but
			// using proto.Merge with gogo messages seems to work fine.
			proto.Merge(outResp.(gogoproto.Message), resp.(gogoproto.Message))
			return nil
		}, nil
	}
	// this is a gogo handler, and we have a protov2 counterparty.
	return func(ctx context.Context, inReq, outResp protoiface.MessageV1) error {
		switch m := inReq.(type) {
		case proto2.Message:
			// we need to marshal and unmarshal the request.
			requestBytes, err := protov2MarshalOpts.Marshal(m)
			if err != nil {
				return err
			}
			resp, err := method.Handler(handler, ctx, func(msg any) error {
				// unmarshal request into the message.
				return cdc.Unmarshal(requestBytes, msg.(gogoproto.Message))
			}, nil)
			if err != nil {
				return err
			}
			// the response is a gogo message, so we cannot just return it.
			// since the request came as protov2, we expect the response
			// to also be protov2.
			respBytes, err := cdc.Marshal(resp.(gogoproto.Message))
			if err != nil {
				return err
			}
			// now we unmarshal back into a protov2 message.
			return proto2.Unmarshal(respBytes, outResp.(proto2.Message))
		case gogoproto.Message:
			// we can just call the handler after making a copy of the message, for safety reasons.
			resp, err := method.Handler(handler, ctx, func(msg any) error {
				// ref: https://github.com/cosmos/cosmos-sdk/issues/18003
				asGogoProto := msg.(gogoproto.Message)
				// NOTE: using gogoproto.Merge will fail for some reason unknown to me, but
				// using proto.Merge with gogo messages seems to work fine.
				proto.Merge(asGogoProto, m)
				return nil
			}, nil)
			if err != nil {
				return err
			}
			// merge on the resp, ref: https://github.com/cosmos/cosmos-sdk/issues/18003
			// NOTE: using gogoproto.Merge will fail for some reason unknown to me, but
			// using proto.Merge with gogo messages seems to work fine.
			proto.Merge(outResp.(gogoproto.Message), resp.(gogoproto.Message))
			return nil
		default:
			panic("unreachable")
		}
	}, nil
}

// isProtov2 returns true if the given method accepts protov2 messages.
// Returns false if it does not.
// It uses the decoder function passed to the method handler to determine
// the type. Since the decoder function is passed in by the concrete implementer the expected
// message where bytes are unmarshaled to, we can use that to determine the type.
func isProtov2(md grpc.MethodDesc) (isV2Type bool, err error) {
	pullRequestType := func(msg interface{}) error {
		typ := reflect.TypeOf(msg)
		switch {
		case typ.Implements(protov2Type):
			isV2Type = true
			return nil
		case typ.Implements(gogoType):
			isV2Type = false
			return nil
		default:
			err = fmt.Errorf("invalid request type %T, expected protov2 or gogo message", msg)
			return nil
		}
	}
	// doNotExecute is a dummy handler that stops the request execution.
	doNotExecute := func(_ context.Context, _ any, _ *grpc.UnaryServerInfo, _ grpc.UnaryHandler) (any, error) {
		return nil, nil
	}
	// we are allowed to pass in a nil context and nil request, since we are not actually executing the request.
	// this is made possible by the doNotExecute function which immediately returns without calling other handlers.
	_, _ = md.Handler(nil, nil, pullRequestType, doNotExecute)
	return
}

// RequestFullNameFromMethodDesc returns the fully-qualified name of the request message of the provided service's method.
func RequestFullNameFromMethodDesc(sd *grpc.ServiceDesc, method grpc.MethodDesc) (protoreflect.FullName, error) {
	methodFullName := protoreflect.FullName(fmt.Sprintf("%s.%s", sd.ServiceName, method.MethodName))
	desc, err := gogoproto.HybridResolver.FindDescriptorByName(methodFullName)
	if err != nil {
		return "", fmt.Errorf("cannot find method descriptor %s", methodFullName)
	}
	methodDesc, ok := desc.(protoreflect.MethodDescriptor)
	if !ok {
		return "", fmt.Errorf("invalid method descriptor %s", methodFullName)
	}
	return methodDesc.Input().FullName(), nil
}

// ResponseFullNameFromMethodDesc returns the fully-qualified name of the response message of the provided service's method.
func ResponseFullNameFromMethodDesc(sd *grpc.ServiceDesc, method grpc.MethodDesc) (protoreflect.FullName, error) {
	methodFullName := protoreflect.FullName(fmt.Sprintf("%s.%s", sd.ServiceName, method.MethodName))
	desc, err := gogoproto.HybridResolver.FindDescriptorByName(methodFullName)
	if err != nil {
		return "", fmt.Errorf("cannot find method descriptor %s", methodFullName)
	}
	methodDesc, ok := desc.(protoreflect.MethodDescriptor)
	if !ok {
		return "", fmt.Errorf("invalid method descriptor %s", methodFullName)
	}
	return methodDesc.Output().FullName(), nil
}
