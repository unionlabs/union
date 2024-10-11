package grpc

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/sirupsen/logrus"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
)

// LogRequests logs the gRPC backend as well as request duration when the log level is set to debug
// or higher.
func LogRequests(
	ctx context.Context,
	method string, req,
	reply interface{},
	cc *grpc.ClientConn,
	invoker grpc.UnaryInvoker,
	opts ...grpc.CallOption,
) error {
	// Shortcut when debug logging is not enabled.
	if logrus.GetLevel() < logrus.DebugLevel {
		return invoker(ctx, method, req, reply, cc, opts...)
	}

	var header metadata.MD
	opts = append(
		opts,
		grpc.Header(&header),
	)
	start := time.Now()
	err := invoker(ctx, method, req, reply, cc, opts...)
	logrus.WithField("backend", header["x-backend"]).
		WithField("method", method).WithField("duration", time.Since(start)).
		Debug("gRPC request finished.")
	return err
}

// LogStream prints the method at DEBUG level at the start of the stream.
func LogStream(
	ctx context.Context,
	sd *grpc.StreamDesc,
	conn *grpc.ClientConn,
	method string,
	streamer grpc.Streamer,
	opts ...grpc.CallOption,
) (grpc.ClientStream, error) {
	// Shortcut when debug logging is not enabled.
	if logrus.GetLevel() < logrus.DebugLevel {
		return streamer(ctx, sd, conn, method, opts...)
	}

	var header metadata.MD
	opts = append(
		opts,
		grpc.Header(&header),
	)
	strm, err := streamer(ctx, sd, conn, method, opts...)
	logrus.WithField("backend", header["x-backend"]).
		WithField("method", method).
		Debug("gRPC stream started.")
	return strm, err
}

// AppendHeaders parses the provided GRPC headers
// and attaches them to the provided context.
func AppendHeaders(parent context.Context, headers []string) context.Context {
	for _, h := range headers {
		if h != "" {
			keyValue := strings.Split(h, "=")
			if len(keyValue) < 2 {
				logrus.Warnf("Incorrect gRPC header flag format. Skipping %v", keyValue[0])
				continue
			}
			parent = metadata.AppendToOutgoingContext(parent, keyValue[0], strings.Join(keyValue[1:], "="))
		}
	}
	return parent
}

// AppendCustomErrorHeader sets a CustomErrorMetadataKey gRPC header on the passed in context,
// using the passed in error data as the header's value. The data is serialized as JSON.
func AppendCustomErrorHeader(ctx context.Context, errorData interface{}) error {
	j, err := json.Marshal(errorData)
	if err != nil {
		return fmt.Errorf("could not marshal error data into JSON: %w", err)
	}
	if err := grpc.SetHeader(ctx, metadata.Pairs(CustomErrorMetadataKey, string(j))); err != nil {
		return fmt.Errorf("could not set custom error header: %w", err)
	}
	return nil
}
