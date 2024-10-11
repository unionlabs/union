package status

import (
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/extgrpc"

	"google.golang.org/grpc/codes"
)

func Error(c codes.Code, msg string) error {
	return extgrpc.WrapWithGrpcCode(errors.New(msg), c)
}

func Errorf(c codes.Code, format string, args ...interface{}) error {
	return extgrpc.WrapWithGrpcCode(errors.Newf(format, args...), c)
}

func WrapErr(c codes.Code, msg string, err error) error {
	return extgrpc.WrapWithGrpcCode(errors.WrapWithDepth(1, err, msg), c)
}

func WrapErrf(c codes.Code, err error, format string, args ...interface{}) error {
	return extgrpc.WrapWithGrpcCode(errors.WrapWithDepthf(1, err, format, args...), c)
}

func Code(err error) codes.Code {
	return extgrpc.GetGrpcCode(err)
}
