package statsd

import (
	"math/rand"
	"sync"
	"sync/atomic"
	"time"
)

// bufferedMetricContexts represent the contexts for Histograms, Distributions
// and Timing. Since those 3 metric types behave the same way and are sampled
// with the same type they're represented by the same class.
type bufferedMetricContexts struct {
	nbContext int32
	mutex     sync.RWMutex
	values    bufferedMetricMap
	newMetric func(string, float64, string) *bufferedMetric

	// Each bufferedMetricContexts uses its own random source and random
	// lock to prevent goroutines from contending for the lock on the
	// "math/rand" package-global random source (e.g. calls like
	// "rand.Float64()" must acquire a shared lock to get the next
	// pseudorandom number).
	random     *rand.Rand
	randomLock sync.Mutex
}

func newBufferedContexts(newMetric func(string, float64, string) *bufferedMetric) bufferedMetricContexts {
	return bufferedMetricContexts{
		values:    bufferedMetricMap{},
		newMetric: newMetric,
		// Note that calling "time.Now().UnixNano()" repeatedly quickly may return
		// very similar values. That's fine for seeding the worker-specific random
		// source because we just need an evenly distributed stream of float values.
		// Do not use this random source for cryptographic randomness.
		random: rand.New(rand.NewSource(time.Now().UnixNano())),
	}
}

func (bc *bufferedMetricContexts) flush(metrics []metric) []metric {
	bc.mutex.Lock()
	values := bc.values
	bc.values = bufferedMetricMap{}
	bc.mutex.Unlock()

	for _, d := range values {
		metrics = append(metrics, d.flushUnsafe())
	}
	atomic.AddInt32(&bc.nbContext, int32(len(values)))
	return metrics
}

func (bc *bufferedMetricContexts) sample(name string, value float64, tags []string, rate float64) error {
	if !shouldSample(rate, bc.random, &bc.randomLock) {
		return nil
	}

	context, stringTags := getContextAndTags(name, tags)

	bc.mutex.RLock()
	if v, found := bc.values[context]; found {
		v.sample(value)
		bc.mutex.RUnlock()
		return nil
	}
	bc.mutex.RUnlock()

	bc.mutex.Lock()
	// Check if another goroutines hasn't created the value betwen the 'RUnlock' and 'Lock'
	if v, found := bc.values[context]; found {
		v.sample(value)
		bc.mutex.Unlock()
		return nil
	}
	bc.values[context] = bc.newMetric(name, value, stringTags)
	bc.mutex.Unlock()
	return nil
}

func (bc *bufferedMetricContexts) resetAndGetNbContext() int32 {
	return atomic.SwapInt32(&bc.nbContext, 0)
}
