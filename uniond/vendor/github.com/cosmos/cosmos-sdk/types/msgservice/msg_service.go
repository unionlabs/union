package msgservice

import (
	"bytes"
	"compress/gzip"
	"fmt"
	"io"
	"reflect"

	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc"
	proto2 "google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protodesc"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/descriptorpb"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx"
)

// RegisterMsgServiceDesc registers all type_urls from Msg services described
// in `sd` into the registry.
func RegisterMsgServiceDesc(registry codectypes.InterfaceRegistry, sd *grpc.ServiceDesc) {
	fdBytesUnzipped := unzip(proto.FileDescriptor(sd.Metadata.(string)))
	if fdBytesUnzipped == nil {
		panic(fmt.Errorf("error unzipping file description for MsgService %s", sd.ServiceName))
	}

	fdRaw := &descriptorpb.FileDescriptorProto{}
	err := proto2.Unmarshal(fdBytesUnzipped, fdRaw)
	if err != nil {
		panic(err)
	}

	fd, err := protodesc.FileOptions{
		AllowUnresolvable: true,
	}.New(fdRaw, nil)
	if err != nil {
		panic(err)
	}

	prefSd := fd.Services().ByName(protoreflect.FullName(sd.ServiceName).Name())
	for i := 0; i < prefSd.Methods().Len(); i++ {
		md := prefSd.Methods().Get(i)
		requestDesc := md.Input()
		responseDesc := md.Output()

		reqTyp := proto.MessageType(string(requestDesc.FullName()))
		respTyp := proto.MessageType(string(responseDesc.FullName()))

		// Register sdk.Msg and sdk.MsgResponse to the registry.
		registry.RegisterImplementations((*sdk.Msg)(nil), reflect.New(reqTyp).Elem().Interface().(proto.Message))
		registry.RegisterImplementations((*tx.MsgResponse)(nil), reflect.New(respTyp).Elem().Interface().(proto.Message))
	}
}

func unzip(b []byte) []byte {
	if b == nil {
		return nil
	}
	r, err := gzip.NewReader(bytes.NewReader(b))
	if err != nil {
		panic(err)
	}

	unzipped, err := io.ReadAll(r)
	if err != nil {
		panic(err)
	}

	return unzipped
}
