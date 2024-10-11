package msgservice

import (
	"reflect"

	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc"

	"cosmossdk.io/core/registry"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx"
)

// RegisterMsgServiceDesc registers all type_urls from Msg services described
// in `sd` into the registry. The ServiceDesc must be a standard gRPC ServiceDesc
// from a generated file as this function will use reflection to extract the
// concrete types and expects the HandlerType to follow the normal
// generated type conventions.
func RegisterMsgServiceDesc(registrar registry.InterfaceRegistrar, sd *grpc.ServiceDesc) {
	handlerType := reflect.TypeOf(sd.HandlerType).Elem()
	msgType := reflect.TypeOf((*proto.Message)(nil)).Elem()
	numMethods := handlerType.NumMethod()
	for i := 0; i < numMethods; i++ {
		method := handlerType.Method(i)
		numIn := method.Type.NumIn()
		numOut := method.Type.NumOut()
		if numIn != 2 || numOut != 2 {
			continue
		}
		reqType := method.Type.In(1)
		resType := method.Type.Out(0)
		if reqType.AssignableTo(msgType) && resType.AssignableTo(msgType) {
			req := reflect.New(reqType.Elem()).Interface()
			registrar.RegisterImplementations((*sdk.Msg)(nil), req.(proto.Message))
			res := reflect.New(resType.Elem()).Interface()
			registrar.RegisterImplementations((*tx.MsgResponse)(nil), res.(proto.Message))
		}
	}
}
