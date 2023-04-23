package grpcweb

import (
	"context"
	"time"

	backoff "github.com/cenkalti/backoff/v4"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	healthpb "google.golang.org/grpc/health/grpc_health_v1"
	"google.golang.org/grpc/status"
)

const healthCheckMethod = "/grpc.health.v1.Health/Watch"

// Client health check function is also part of the grpc/internal package
// The following code is a simplified version of client.go
// For more details see: https://pkg.go.dev/google.golang.org/grpc/health
func ClientHealthCheck(ctx context.Context, backendConn *grpc.ClientConn, service string, setServingStatus func(serving bool)) error {
	shouldBackoff := false // No need for backoff on the first connection attempt
	backoffSrc := backoff.NewExponentialBackOff()
	healthClient := healthpb.NewHealthClient(backendConn)

	for {
		// Backs off if the connection has failed in some way without receiving a message in the previous retry.
		if shouldBackoff {
			select {
			case <-time.After(backoffSrc.NextBackOff()):
			case <-ctx.Done():
				return nil
			}
		}
		shouldBackoff = true // we should backoff next time, since we attempt connecting below

		req := healthpb.HealthCheckRequest{Service: service}
		s, err := healthClient.Watch(ctx, &req)
		if err != nil {
			continue
		}

		resp := new(healthpb.HealthCheckResponse)
		for {
			err = s.RecvMsg(resp)
			if err != nil {
				setServingStatus(false)
				// The health check functionality should be disabled if health check service is not implemented on the backend
				if status.Code(err) == codes.Unimplemented {
					return err
				}
				// breaking out of the loop, since we got an error from Recv, triggering reconnect
				break
			}

			// As a message has been received, removes the need for backoff for the next retry.
			shouldBackoff = false
			backoffSrc.Reset()

			if resp.Status == healthpb.HealthCheckResponse_SERVING {
				setServingStatus(true)
			} else {
				setServingStatus(false)
			}
		}
	}
}
