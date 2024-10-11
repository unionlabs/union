package statsd

import (
	"sort"
	"strings"
	"sync"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestAggregatorSample(t *testing.T) {
	a := newAggregator(nil)

	tags := []string{"tag1", "tag2"}

	for i := 0; i < 2; i++ {
		a.gauge("gaugeTest", 21, tags)
		assert.Len(t, a.gauges, 1)
		assert.Contains(t, a.gauges, "gaugeTest:tag1,tag2")

		a.count("countTest", 21, tags)
		assert.Len(t, a.counts, 1)
		assert.Contains(t, a.counts, "countTest:tag1,tag2")

		a.set("setTest", "value1", tags)
		assert.Len(t, a.sets, 1)
		assert.Contains(t, a.sets, "setTest:tag1,tag2")

		a.set("setTest", "value1", tags)
		assert.Len(t, a.sets, 1)
		assert.Contains(t, a.sets, "setTest:tag1,tag2")

		a.histogram("histogramTest", 21, tags, 1)
		assert.Len(t, a.histograms.values, 1)
		assert.Contains(t, a.histograms.values, "histogramTest:tag1,tag2")

		a.distribution("distributionTest", 21, tags, 1)
		assert.Len(t, a.distributions.values, 1)
		assert.Contains(t, a.distributions.values, "distributionTest:tag1,tag2")

		a.timing("timingTest", 21, tags, 1)
		assert.Len(t, a.timings.values, 1)
		assert.Contains(t, a.timings.values, "timingTest:tag1,tag2")
	}
}

func TestAggregatorFlush(t *testing.T) {
	a := newAggregator(nil)

	tags := []string{"tag1", "tag2"}

	a.gauge("gaugeTest1", 21, tags)
	a.gauge("gaugeTest1", 10, tags)
	a.gauge("gaugeTest2", 15, tags)

	a.count("countTest1", 21, tags)
	a.count("countTest1", 10, tags)
	a.count("countTest2", 1, tags)

	a.set("setTest1", "value1", tags)
	a.set("setTest1", "value1", tags)
	a.set("setTest1", "value2", tags)
	a.set("setTest2", "value1", tags)

	a.histogram("histogramTest1", 21, tags, 1)
	a.histogram("histogramTest1", 22, tags, 1)
	a.histogram("histogramTest2", 23, tags, 1)

	a.distribution("distributionTest1", 21, tags, 1)
	a.distribution("distributionTest1", 22, tags, 1)
	a.distribution("distributionTest2", 23, tags, 1)

	a.timing("timingTest1", 21, tags, 1)
	a.timing("timingTest1", 22, tags, 1)
	a.timing("timingTest2", 23, tags, 1)

	metrics := a.flushMetrics()

	assert.Len(t, a.gauges, 0)
	assert.Len(t, a.counts, 0)
	assert.Len(t, a.sets, 0)
	assert.Len(t, a.histograms.values, 0)
	assert.Len(t, a.distributions.values, 0)
	assert.Len(t, a.timings.values, 0)

	assert.Len(t, metrics, 13)

	sort.Slice(metrics, func(i, j int) bool {
		if metrics[i].metricType == metrics[j].metricType {
			res := strings.Compare(metrics[i].name, metrics[j].name)
			// this happens fo set
			if res == 0 {
				return strings.Compare(metrics[i].svalue, metrics[j].svalue) != 1
			}
			return res != 1
		}
		return metrics[i].metricType < metrics[j].metricType
	})

	assert.Equal(t, []metric{
		metric{
			metricType: gauge,
			name:       "gaugeTest1",
			tags:       tags,
			rate:       1,
			fvalue:     float64(10),
		},
		metric{
			metricType: gauge,
			name:       "gaugeTest2",
			tags:       tags,
			rate:       1,
			fvalue:     float64(15),
		},
		metric{
			metricType: count,
			name:       "countTest1",
			tags:       tags,
			rate:       1,
			ivalue:     int64(31),
		},
		metric{
			metricType: count,
			name:       "countTest2",
			tags:       tags,
			rate:       1,
			ivalue:     int64(1),
		},
		metric{
			metricType: histogramAggregated,
			name:       "histogramTest1",
			stags:      strings.Join(tags, tagSeparatorSymbol),
			rate:       1,
			fvalues:    []float64{21.0, 22.0},
		},
		metric{
			metricType: histogramAggregated,
			name:       "histogramTest2",
			stags:      strings.Join(tags, tagSeparatorSymbol),
			rate:       1,
			fvalues:    []float64{23.0},
		},
		metric{
			metricType: distributionAggregated,
			name:       "distributionTest1",
			stags:      strings.Join(tags, tagSeparatorSymbol),
			rate:       1,
			fvalues:    []float64{21.0, 22.0},
		},
		metric{
			metricType: distributionAggregated,
			name:       "distributionTest2",
			stags:      strings.Join(tags, tagSeparatorSymbol),
			rate:       1,
			fvalues:    []float64{23.0},
		},
		metric{
			metricType: set,
			name:       "setTest1",
			tags:       tags,
			rate:       1,
			svalue:     "value1",
		},
		metric{
			metricType: set,
			name:       "setTest1",
			tags:       tags,
			rate:       1,
			svalue:     "value2",
		},
		metric{
			metricType: set,
			name:       "setTest2",
			tags:       tags,
			rate:       1,
			svalue:     "value1",
		},
		metric{
			metricType: timingAggregated,
			name:       "timingTest1",
			stags:      strings.Join(tags, tagSeparatorSymbol),
			rate:       1,
			fvalues:    []float64{21.0, 22.0},
		},
		metric{
			metricType: timingAggregated,
			name:       "timingTest2",
			stags:      strings.Join(tags, tagSeparatorSymbol),
			rate:       1,
			fvalues:    []float64{23.0},
		},
	},
		metrics)

}

func TestAggregatorFlushConcurrency(t *testing.T) {
	a := newAggregator(nil)

	var wg sync.WaitGroup
	wg.Add(10)

	tags := []string{"tag1", "tag2"}

	for i := 0; i < 5; i++ {
		go func() {
			defer wg.Done()

			a.gauge("gaugeTest1", 21, tags)
			a.count("countTest1", 21, tags)
			a.set("setTest1", "value1", tags)
			a.histogram("histogramTest1", 21, tags, 1)
			a.distribution("distributionTest1", 21, tags, 1)
			a.timing("timingTest1", 21, tags, 1)
		}()
	}

	for i := 0; i < 5; i++ {
		go func() {
			defer wg.Done()

			a.flushMetrics()
		}()
	}

	wg.Wait()
}
