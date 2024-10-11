package telemetry

import (
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

var mu sync.Mutex

func initTelemetry(v bool) {
	globalTelemetryEnabled = v
}

// Reset the global state to a known disabled state before each test.
func setupTest(t *testing.T) {
	t.Helper()
	mu.Lock() // Ensure no other test can modify global state at the same time.
	defer mu.Unlock()
	initTelemetry(false)
}

// TestNow tests the Now function when telemetry is enabled and disabled.
func TestNow(t *testing.T) {
	setupTest(t) // Locks the mutex to avoid race condition.

	initTelemetry(true)
	telemetryTime := Now()
	assert.NotEqual(t, time.Time{}, telemetryTime, "Now() should not return zero time when telemetry is enabled")

	setupTest(t) // Reset the global state and lock the mutex again.

	initTelemetry(false)
	telemetryTime = Now()
	assert.Equal(t, time.Time{}, telemetryTime, "Now() should return zero time when telemetry is disabled")
}

// TestIsTelemetryEnabled tests the IsTelemetryEnabled function.
func TestIsTelemetryEnabled(t *testing.T) {
	setupTest(t) // Locks the mutex to avoid race condition.

	initTelemetry(true)
	assert.True(t, IsTelemetryEnabled(), "IsTelemetryEnabled() should return true when globalTelemetryEnabled is set to true")

	setupTest(t) // Reset the global state and lock the mutex again.

	initTelemetry(false)
	assert.False(t, IsTelemetryEnabled(), "IsTelemetryEnabled() should return false when globalTelemetryEnabled is set to false")
}
