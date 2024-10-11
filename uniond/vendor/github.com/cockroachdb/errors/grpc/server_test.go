package grpc

import (
	"context"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/grpc/status"

	"google.golang.org/grpc/codes"
)

var ErrCantEcho = errors.New("unable to echo")
var ErrTooLong = errors.New("text is too long")
var ErrInternal = errors.New("internal error!")

type EchoServer struct {
}

func (srv *EchoServer) Echo(ctx context.Context, req *EchoRequest) (*EchoReply, error) {
	msg := req.Text
	switch {
	case msg == "noecho":
		return nil, ErrCantEcho
	case len(msg) > 10:
		return nil, errors.WithMessage(ErrTooLong, msg+" is too long")
	case msg == "reverse":
		return nil, status.Error(codes.Unimplemented, "reverse is not implemented")
	case msg == "internal":
		return nil, status.WrapErr(codes.Internal, "there was a problem", ErrInternal)
	}
	return &EchoReply{
		Reply: "echoing: " + msg,
	}, nil
}
