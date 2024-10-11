package statsd

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestShouldSample(t *testing.T) {
	rates := []float64{0.01, 0.05, 0.1, 0.25, 0.5, 0.75, 0.9, 0.99, 1.0}
	iterations := 50_000

	for _, rate := range rates {
		rate := rate // Capture range variable.
		t.Run(fmt.Sprintf("Rate %0.2f", rate), func(t *testing.T) {
			t.Parallel()

			worker := newWorker(newBufferPool(1, 1, 1), nil)
			count := 0
			for i := 0; i < iterations; i++ {
				if shouldSample(rate, worker.random, &worker.randomLock) {
					count++
				}
			}
			assert.InDelta(t, rate, float64(count)/float64(iterations), 0.01)
		})
	}
}

func BenchmarkShouldSample(b *testing.B) {
	b.RunParallel(func(pb *testing.PB) {
		worker := newWorker(newBufferPool(1, 1, 1), nil)
		for pb.Next() {
			shouldSample(0.1, worker.random, &worker.randomLock)
		}
	})
}

func initWorker(bufferSize int) (*bufferPool, *sender, *worker) {
	pool := newBufferPool(10, bufferSize, 5)
	// manually create the sender so the sender loop is not started. All we
	// need is the queue
	s := &sender{
		queue: make(chan *statsdBuffer, 10),
		pool:  pool,
	}

	w := newWorker(pool, s)
	return pool, s, w
}

func testWorker(t *testing.T, m metric, expectedBuffer string) {
	_, s, w := initWorker(100)

	err := w.processMetric(m)
	assert.Nil(t, err)

	w.flush()
	data := <-s.queue
	assert.Equal(t, expectedBuffer, string(data.buffer))

}

func TestWorkerGauge(t *testing.T) {
	testWorker(
		t,
		metric{
			metricType: gauge,
			namespace:  "namespace.",
			globalTags: []string{"globalTags", "globalTags2"},
			name:       "test_gauge",
			fvalue:     21,
			tags:       []string{"tag1", "tag2"},
			rate:       1,
		},
		"namespace.test_gauge:21|g|#globalTags,globalTags2,tag1,tag2\n",
	)
}

func TestWorkerCount(t *testing.T) {
	testWorker(
		t,
		metric{
			metricType: count,
			namespace:  "namespace.",
			globalTags: []string{"globalTags", "globalTags2"},
			name:       "test_count",
			ivalue:     21,
			tags:       []string{"tag1", "tag2"},
			rate:       1,
		},
		"namespace.test_count:21|c|#globalTags,globalTags2,tag1,tag2\n",
	)
}

func TestWorkerHistogram(t *testing.T) {
	testWorker(
		t,
		metric{
			metricType: histogram,
			namespace:  "namespace.",
			globalTags: []string{"globalTags", "globalTags2"},
			name:       "test_histogram",
			fvalue:     21,
			tags:       []string{"tag1", "tag2"},
			rate:       1,
		},
		"namespace.test_histogram:21|h|#globalTags,globalTags2,tag1,tag2\n",
	)
}

func TestWorkerDistribution(t *testing.T) {
	testWorker(
		t,
		metric{
			metricType: distribution,
			namespace:  "namespace.",
			globalTags: []string{"globalTags", "globalTags2"},
			name:       "test_distribution",
			fvalue:     21,
			tags:       []string{"tag1", "tag2"},
			rate:       1,
		},
		"namespace.test_distribution:21|d|#globalTags,globalTags2,tag1,tag2\n",
	)
}

func TestWorkerSet(t *testing.T) {
	testWorker(
		t,
		metric{
			metricType: set,
			namespace:  "namespace.",
			globalTags: []string{"globalTags", "globalTags2"},
			name:       "test_set",
			svalue:     "value:1",
			tags:       []string{"tag1", "tag2"},
			rate:       1,
		},
		"namespace.test_set:value:1|s|#globalTags,globalTags2,tag1,tag2\n",
	)
}

func TestWorkerTiming(t *testing.T) {
	testWorker(
		t,
		metric{
			metricType: timing,
			namespace:  "namespace.",
			globalTags: []string{"globalTags", "globalTags2"},
			name:       "test_timing",
			fvalue:     1.2,
			tags:       []string{"tag1", "tag2"},
			rate:       1,
		},
		"namespace.test_timing:1.200000|ms|#globalTags,globalTags2,tag1,tag2\n",
	)
}

func TestWorkerHistogramAggregated(t *testing.T) {
	testWorker(
		t,
		metric{
			metricType: histogramAggregated,
			namespace:  "namespace.",
			globalTags: []string{"globalTags", "globalTags2"},
			name:       "test_histogram",
			fvalues:    []float64{1.2},
			stags:      "tag1,tag2",
			rate:       1,
		},
		"namespace.test_histogram:1.2|h|#globalTags,globalTags2,tag1,tag2\n",
	)
}

func TestWorkerHistogramAggregatedMultiple(t *testing.T) {
	_, s, w := initWorker(100)

	m := metric{
		metricType: histogramAggregated,
		namespace:  "namespace.",
		globalTags: []string{"globalTags", "globalTags2"},
		name:       "test_histogram",
		fvalues:    []float64{1.1, 2.2, 3.3, 4.4},
		stags:      "tag1,tag2",
		rate:       1,
	}
	err := w.processMetric(m)
	assert.Nil(t, err)

	w.flush()
	data := <-s.queue
	assert.Equal(t, "namespace.test_histogram:1.1:2.2:3.3:4.4|h|#globalTags,globalTags2,tag1,tag2\n", string(data.buffer))

	// reducing buffer size so not all values fit in one packet
	_, s, w = initWorker(70)

	err = w.processMetric(m)
	assert.Nil(t, err)

	w.flush()
	data = <-s.queue
	assert.Equal(t, "namespace.test_histogram:1.1:2.2|h|#globalTags,globalTags2,tag1,tag2\n", string(data.buffer))
	data = <-s.queue
	assert.Equal(t, "namespace.test_histogram:3.3:4.4|h|#globalTags,globalTags2,tag1,tag2\n", string(data.buffer))
}

func TestWorkerDistributionAggregated(t *testing.T) {
	testWorker(
		t,
		metric{
			metricType: distributionAggregated,
			namespace:  "namespace.",
			globalTags: []string{"globalTags", "globalTags2"},
			name:       "test_distribution",
			fvalues:    []float64{1.2},
			stags:      "tag1,tag2",
			rate:       1,
		},
		"namespace.test_distribution:1.2|d|#globalTags,globalTags2,tag1,tag2\n",
	)
}

func TestWorkerDistributionAggregatedMultiple(t *testing.T) {
	_, s, w := initWorker(100)

	m := metric{
		metricType: distributionAggregated,
		namespace:  "namespace.",
		globalTags: []string{"globalTags", "globalTags2"},
		name:       "test_distribution",
		fvalues:    []float64{1.1, 2.2, 3.3, 4.4},
		stags:      "tag1,tag2",
		rate:       1,
	}
	err := w.processMetric(m)
	assert.Nil(t, err)

	w.flush()
	data := <-s.queue
	assert.Equal(t, "namespace.test_distribution:1.1:2.2:3.3:4.4|d|#globalTags,globalTags2,tag1,tag2\n", string(data.buffer))

	// reducing buffer size so not all values fit in one packet
	_, s, w = initWorker(72)

	err = w.processMetric(m)
	assert.Nil(t, err)

	w.flush()
	data = <-s.queue
	assert.Equal(t, "namespace.test_distribution:1.1:2.2|d|#globalTags,globalTags2,tag1,tag2\n", string(data.buffer))
	data = <-s.queue
	assert.Equal(t, "namespace.test_distribution:3.3:4.4|d|#globalTags,globalTags2,tag1,tag2\n", string(data.buffer))
}

func TestWorkerMultipleDifferentDistributionAggregated(t *testing.T) {
	// first metric will fit but not the second one
	_, s, w := initWorker(160)

	m := metric{
		metricType: distributionAggregated,
		namespace:  "namespace.",
		globalTags: []string{"globalTags", "globalTags2"},
		name:       "test_distribution",
		fvalues:    []float64{1.1, 2.2, 3.3, 4.4},
		stags:      "tag1,tag2",
		rate:       1,
	}
	err := w.processMetric(m)
	assert.Nil(t, err)
	m = metric{
		metricType: distributionAggregated,
		namespace:  "namespace.",
		globalTags: []string{"globalTags", "globalTags2"},
		name:       "test_distribution_2",
		fvalues:    []float64{1.1, 2.2, 3.3, 4.4},
		stags:      "tag1,tag2",
		rate:       1,
	}
	err = w.processMetric(m)
	assert.Nil(t, err)

	w.flush()
	data := <-s.queue
	assert.Equal(t, "namespace.test_distribution:1.1:2.2:3.3:4.4|d|#globalTags,globalTags2,tag1,tag2\nnamespace.test_distribution_2:1.1:2.2:3.3|d|#globalTags,globalTags2,tag1,tag2\n", string(data.buffer))
	data = <-s.queue
	assert.Equal(t, "namespace.test_distribution_2:4.4|d|#globalTags,globalTags2,tag1,tag2\n", string(data.buffer))
}
