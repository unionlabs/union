package circuitbreaker_test

import (
	"io/ioutil"
	stdlog "log"
	"testing"
	"time"

	"github.com/afex/hystrix-go/hystrix"

	"github.com/go-kit/kit/circuitbreaker"
)

func TestHystrix(t *testing.T) {
	stdlog.SetOutput(ioutil.Discard)

	const (
		commandName   = "my-endpoint"
		errorPercent  = 5
		maxConcurrent = 1000
	)
	hystrix.ConfigureCommand(commandName, hystrix.CommandConfig{
		ErrorPercentThreshold: errorPercent,
		MaxConcurrentRequests: maxConcurrent,
	})

	var (
		breaker          = circuitbreaker.Hystrix(commandName)
		primeWith        = hystrix.DefaultVolumeThreshold * 2
		shouldPass       = func(n int) bool { return (float64(n) / float64(primeWith+n)) <= (float64(errorPercent-1) / 100.0) }
		openCircuitError = hystrix.ErrCircuitOpen.Error()
	)

	// hystrix-go uses buffered channels to receive reports on request success/failure,
	// and so is basically impossible to test deterministically. We have to make sure
	// the report buffer is emptied, by injecting a sleep between each invocation.
	requestDelay := 5 * time.Millisecond

	testFailingEndpoint(t, breaker, primeWith, shouldPass, requestDelay, openCircuitError)
}
