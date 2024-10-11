package execution

import (
	"context"
	"testing"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

// TestCleanup ensures that the cleanup function unregisters the prometheus.Collection
// also tests the interchangability of the explicit prometheus Register/Unregister
// and the implicit methods within the collector implementation
func TestCleanup(t *testing.T) {
	ctx := context.Background()
	pc, err := NewPowchainCollector(ctx)
	assert.NoError(t, err, "Unexpected error calling NewPowchainCollector")
	unregistered := pc.unregister()
	assert.Equal(t, true, unregistered, "PowchainCollector.unregister did not return true (via prometheus.DefaultRegistry)")
	// PowchainCollector is a prometheus.Collector, so we should be able to register it again
	err = prometheus.Register(pc)
	assert.NoError(t, err, "Got error from prometheus.Register after unregistering PowchainCollector")
	// even if it somehow gets registered somewhere else, unregister should work
	unregistered = pc.unregister()
	assert.Equal(t, true, unregistered, "PowchainCollector.unregister failed on the second attempt")
	// and so we should be able to register it again
	err = prometheus.Register(pc)
	assert.NoError(t, err, "Got error from prometheus.Register on the second attempt")
	// ok clean it up one last time for real :)
	unregistered = prometheus.Unregister(pc)
	assert.Equal(t, true, unregistered, "prometheus.Unregister failed to unregister PowchainCollector on final cleanup")
}

// TestCancelation tests that canceling the context passed into
// NewPowchainCollector cleans everything up as expected. This
// does come at the cost of an extra channel cluttering up
// PowchainCollector, just for this test.
func TestCancelation(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	pc, err := NewPowchainCollector(ctx)
	assert.NoError(t, err, "Unexpected error calling NewPowchainCollector")
	ticker := time.NewTicker(10 * time.Second)
	cancel()
	select {
	case <-ticker.C:
		t.Error("Hit timeout waiting for cancel() to cleanup PowchainCollector")
	case <-pc.finishChan:
		break
	}
	err = prometheus.Register(pc)
	assert.NoError(t, err, "Got error from prometheus.Register after unregistering PowchainCollector through canceled context")
}
