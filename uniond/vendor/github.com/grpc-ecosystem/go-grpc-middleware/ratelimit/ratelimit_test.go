package ratelimit

import (
	"context"
	"errors"
	"testing"

	"google.golang.org/grpc"

	"github.com/stretchr/testify/assert"
)

const errMsgFake = "fake error"

type mockPassLimiter struct{}

func (*mockPassLimiter) Limit() bool {
	return false
}

func TestUnaryServerInterceptor_RateLimitPass(t *testing.T) {
	interceptor := UnaryServerInterceptor(&mockPassLimiter{})
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return nil, errors.New(errMsgFake)
	}
	info := &grpc.UnaryServerInfo{
		FullMethod: "FakeMethod",
	}
	resp, err := interceptor(nil, nil, info, handler)
	assert.Nil(t, resp)
	assert.EqualError(t, err, errMsgFake)
}

type mockFailLimiter struct{}

func (*mockFailLimiter) Limit() bool {
	return true
}

func TestUnaryServerInterceptor_RateLimitFail(t *testing.T) {
	interceptor := UnaryServerInterceptor(&mockFailLimiter{})
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return nil, errors.New(errMsgFake)
	}
	info := &grpc.UnaryServerInfo{
		FullMethod: "FakeMethod",
	}
	resp, err := interceptor(nil, nil, info, handler)
	assert.Nil(t, resp)
	assert.EqualError(t, err, "rpc error: code = ResourceExhausted desc = FakeMethod is rejected by grpc_ratelimit middleware, please retry later.")
}

func TestStreamServerInterceptor_RateLimitPass(t *testing.T) {
	interceptor := StreamServerInterceptor(&mockPassLimiter{})
	handler := func(srv interface{}, stream grpc.ServerStream) error {
		return errors.New(errMsgFake)
	}
	info := &grpc.StreamServerInfo{
		FullMethod: "FakeMethod",
	}
	err := interceptor(nil, nil, info, handler)
	assert.EqualError(t, err, errMsgFake)
}

func TestStreamServerInterceptor_RateLimitFail(t *testing.T) {
	interceptor := StreamServerInterceptor(&mockFailLimiter{})
	handler := func(srv interface{}, stream grpc.ServerStream) error {
		return errors.New(errMsgFake)
	}
	info := &grpc.StreamServerInfo{
		FullMethod: "FakeMethod",
	}
	err := interceptor(nil, nil, info, handler)
	assert.EqualError(t, err, "rpc error: code = ResourceExhausted desc = FakeMethod is rejected by grpc_ratelimit middleware, please retry later.")
}
