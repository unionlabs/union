package kit_test

import (
	"time"

	"github.com/go-kit/log"
	grpc_middleware "github.com/grpc-ecosystem/go-grpc-middleware"
	"github.com/grpc-ecosystem/go-grpc-middleware/logging/kit"
	grpc_ctxtags "github.com/grpc-ecosystem/go-grpc-middleware/tags"

	"google.golang.org/grpc"
)

var (
	customFunc kit.CodeToLevel
)

// Initialization shows a relatively complex initialization sequence.
func Example_initialization() {
	// Logger is used, allowing pre-definition of certain fields by the user.
	logger := log.NewNopLogger()
	// Shared options for the logger, with a custom gRPC code to log level function.
	opts := []kit.Option{
		kit.WithLevels(customFunc),
	}
	// Create a server, make sure we put the grpc_ctxtags context before everything else.
	_ = grpc.NewServer(
		grpc_middleware.WithUnaryServerChain(
			grpc_ctxtags.UnaryServerInterceptor(grpc_ctxtags.WithFieldExtractor(grpc_ctxtags.CodeGenRequestFieldExtractor)),
			kit.UnaryServerInterceptor(logger, opts...),
		),
		grpc_middleware.WithStreamServerChain(
			grpc_ctxtags.StreamServerInterceptor(grpc_ctxtags.WithFieldExtractor(grpc_ctxtags.CodeGenRequestFieldExtractor)),
			kit.StreamServerInterceptor(logger, opts...),
		),
	)
}

func Example_initializationWithDurationFieldOverride() {
	// Logger is used, allowing pre-definition of certain fields by the user.
	logger := log.NewNopLogger()
	// Shared options for the logger, with a custom duration to log field function.
	opts := []kit.Option{
		kit.WithDurationField(func(duration time.Duration) []interface{} {
			return kit.DurationToTimeMillisField(duration)
		}),
	}
	_ = grpc.NewServer(
		grpc_middleware.WithUnaryServerChain(
			grpc_ctxtags.UnaryServerInterceptor(),
			kit.UnaryServerInterceptor(logger, opts...),
		),
		grpc_middleware.WithStreamServerChain(
			grpc_ctxtags.StreamServerInterceptor(),
			kit.StreamServerInterceptor(logger, opts...),
		),
	)
}

func ExampleWithDecider() {
	opts := []kit.Option{
		kit.WithDecider(func(methodFullName string, err error) bool {
			// will not log gRPC calls if it was a call to healthcheck and no error was raised
			if err == nil && methodFullName == "blah.foo.healthcheck" {
				return false
			}

			// by default you will log all calls
			return true
		}),
	}

	_ = []grpc.ServerOption{
		grpc_middleware.WithStreamServerChain(
			grpc_ctxtags.StreamServerInterceptor(),
			kit.StreamServerInterceptor(log.NewNopLogger(), opts...)),
		grpc_middleware.WithUnaryServerChain(
			grpc_ctxtags.UnaryServerInterceptor(),
			kit.UnaryServerInterceptor(log.NewNopLogger(), opts...)),
	}
}
