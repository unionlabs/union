// Copyright 2016 Michal Witkowski. All Rights Reserved.
// See LICENSE for licensing terms.

package grpc_retry_test

import (
	"context"
	"fmt"
	"io"
	"time"

	"github.com/grpc-ecosystem/go-grpc-middleware/retry"
	pb_testproto "github.com/grpc-ecosystem/go-grpc-middleware/testing/testproto"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
)

var cc *grpc.ClientConn

func newCtx(timeout time.Duration) context.Context {
	ctx, _ := context.WithTimeout(context.TODO(), timeout)
	return ctx
}

// Simple example of using the default interceptor configuration.
func Example_initialization() {
	grpc.Dial("myservice.example.com",
		grpc.WithStreamInterceptor(grpc_retry.StreamClientInterceptor()),
		grpc.WithUnaryInterceptor(grpc_retry.UnaryClientInterceptor()),
	)
}

// Complex example with a 100ms linear backoff interval, and retry only on NotFound and Unavailable.
func Example_initializationWithOptions() {
	opts := []grpc_retry.CallOption{
		grpc_retry.WithBackoff(grpc_retry.BackoffLinear(100 * time.Millisecond)),
		grpc_retry.WithCodes(codes.NotFound, codes.Aborted),
	}
	grpc.Dial("myservice.example.com",
		grpc.WithStreamInterceptor(grpc_retry.StreamClientInterceptor(opts...)),
		grpc.WithUnaryInterceptor(grpc_retry.UnaryClientInterceptor(opts...)),
	)
}

// Example with an exponential backoff starting with 100ms.
//
// Each next interval is the previous interval multiplied by 2.
func Example_initializationWithExponentialBackoff() {
	opts := []grpc_retry.CallOption{
		grpc_retry.WithBackoff(grpc_retry.BackoffExponential(100 * time.Millisecond)),
	}
	grpc.Dial("myservice.example.com",
		grpc.WithStreamInterceptor(grpc_retry.StreamClientInterceptor(opts...)),
		grpc.WithUnaryInterceptor(grpc_retry.UnaryClientInterceptor(opts...)),
	)
}

// Simple example of an idempotent `ServerStream` call, that will be retried automatically 3 times.
func Example_simpleCall() {
	client := pb_testproto.NewTestServiceClient(cc)
	stream, _ := client.PingList(newCtx(1*time.Second), &pb_testproto.PingRequest{}, grpc_retry.WithMax(3))

	for {
		pong, err := stream.Recv() // retries happen here
		if err == io.EOF {
			break
		} else if err != nil {
			return
		}
		fmt.Printf("got pong: %v", pong)
	}
}

// This is an example of an `Unary` call that will also retry on deadlines.
//
// Because the passed in context has a `5s` timeout, the whole `Ping` invocation should finish
// within that time. However, by default all retried calls will use the parent context for their
// deadlines. This means, that unless you shorten the deadline of each call of the retry, you won't
// be able to retry the first call at all.
//
// `WithPerRetryTimeout` allows you to shorten the deadline of each retry call, allowing you to fit
// multiple retries in the single parent deadline.
func ExampleWithPerRetryTimeout() {
	client := pb_testproto.NewTestServiceClient(cc)
	pong, _ := client.Ping(
		newCtx(5*time.Second),
		&pb_testproto.PingRequest{},
		grpc_retry.WithMax(3),
		grpc_retry.WithPerRetryTimeout(1*time.Second))

	fmt.Printf("got pong: %v", pong)
}
