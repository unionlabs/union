//go:build integration
// +build integration

package eureka

import (
	"os"
	"testing"
	"time"

	"github.com/hudl/fargo"

	"github.com/go-kit/log"
)

// Package sd/eureka provides a wrapper around the Netflix Eureka service
// registry by way of the Fargo library. This test assumes the user has an
// instance of Eureka available at the address in the environment variable.
// Example `${EUREKA_ADDR}` format: http://localhost:8761/eureka
//
// NOTE: when starting a Eureka server for integration testing, ensure
// the response cache interval is reduced to one second. This can be
// achieved with the following Java argument:
// `-Deureka.server.responseCacheUpdateIntervalMs=1000`
func TestIntegration(t *testing.T) {
	eurekaAddr := os.Getenv("EUREKA_ADDR")
	if eurekaAddr == "" {
		t.Skip("EUREKA_ADDR is not set")
	}

	logger := log.NewLogfmtLogger(os.Stderr)
	logger = log.With(logger, "ts", log.DefaultTimestamp)

	var fargoConfig fargo.Config
	// Target Eureka server(s).
	fargoConfig.Eureka.ServiceUrls = []string{eurekaAddr}
	// How often the subscriber should poll for updates.
	fargoConfig.Eureka.PollIntervalSeconds = 1

	// Create a Fargo connection and a Eureka registrar.
	fargoConnection := fargo.NewConnFromConfig(fargoConfig)
	registrar1 := NewRegistrar(&fargoConnection, instanceTest1, log.With(logger, "component", "registrar1"))

	// Register one instance.
	registrar1.Register()
	defer registrar1.Deregister()

	// Build a Eureka instancer.
	instancer := NewInstancer(
		&fargoConnection,
		appNameTest,
		log.With(logger, "component", "instancer"),
	)
	defer instancer.Stop()

	// checks every 100ms (fr up to 10s) for the expected number of instances to be reported
	waitForInstances := func(count int) {
		for t := 0; t < 100; t++ {
			state := instancer.state()
			if len(state.Instances) == count {
				return
			}
			time.Sleep(100 * time.Millisecond)
		}
		state := instancer.state()
		if state.Err != nil {
			t.Error(state.Err)
		}
		if want, have := 1, len(state.Instances); want != have {
			t.Errorf("want %d, have %d", want, have)
		}
	}

	// We should have one instance immediately after subscriber instantiation.
	waitForInstances(1)

	// Register a second instance
	registrar2 := NewRegistrar(&fargoConnection, instanceTest2, log.With(logger, "component", "registrar2"))
	registrar2.Register()
	defer registrar2.Deregister() // In case of exceptional circumstances.

	// This should be enough time for a scheduled update assuming Eureka is
	// configured with the properties mentioned in the function comments.
	waitForInstances(2)

	// Deregister the second instance.
	registrar2.Deregister()

	// Wait for another scheduled update.
	// And then there was one.
	waitForInstances(1)
}
