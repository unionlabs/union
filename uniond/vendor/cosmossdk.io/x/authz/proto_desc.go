package authz

import "google.golang.org/grpc"

// MsgServiceDesc return ServiceDesc for Msg server
func MsgServiceDesc() *grpc.ServiceDesc {
	return &_Msg_serviceDesc
}
