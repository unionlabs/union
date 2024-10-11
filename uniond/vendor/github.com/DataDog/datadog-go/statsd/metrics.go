package statsd

import (
	"math"
	"sync"
	"sync/atomic"
)

/*
Those are metrics type that can be aggregated on the client side:
  - Gauge
  - Count
  - Set
*/

type countMetric struct {
	value int64
	name  string
	tags  []string
}

func newCountMetric(name string, value int64, tags []string) *countMetric {
	return &countMetric{
		value: value,
		name:  name,
		tags:  tags,
	}
}

func (c *countMetric) sample(v int64) {
	atomic.AddInt64(&c.value, v)
}

func (c *countMetric) flushUnsafe() metric {
	return metric{
		metricType: count,
		name:       c.name,
		tags:       c.tags,
		rate:       1,
		ivalue:     c.value,
	}
}

// Gauge

type gaugeMetric struct {
	value uint64
	name  string
	tags  []string
}

func newGaugeMetric(name string, value float64, tags []string) *gaugeMetric {
	return &gaugeMetric{
		value: math.Float64bits(value),
		name:  name,
		tags:  tags,
	}
}

func (g *gaugeMetric) sample(v float64) {
	atomic.StoreUint64(&g.value, math.Float64bits(v))
}

func (g *gaugeMetric) flushUnsafe() metric {
	return metric{
		metricType: gauge,
		name:       g.name,
		tags:       g.tags,
		rate:       1,
		fvalue:     math.Float64frombits(g.value),
	}
}

// Set

type setMetric struct {
	data map[string]struct{}
	name string
	tags []string
	sync.Mutex
}

func newSetMetric(name string, value string, tags []string) *setMetric {
	set := &setMetric{
		data: map[string]struct{}{},
		name: name,
		tags: tags,
	}
	set.data[value] = struct{}{}
	return set
}

func (s *setMetric) sample(v string) {
	s.Lock()
	defer s.Unlock()
	s.data[v] = struct{}{}
}

// Sets are aggregated on the agent side too. We flush the keys so a set from
// multiple application can be correctly aggregated on the agent side.
func (s *setMetric) flushUnsafe() []metric {
	if len(s.data) == 0 {
		return nil
	}

	metrics := make([]metric, len(s.data))
	i := 0
	for value := range s.data {
		metrics[i] = metric{
			metricType: set,
			name:       s.name,
			tags:       s.tags,
			rate:       1,
			svalue:     value,
		}
		i++
	}
	return metrics
}

// Histograms, Distributions and Timings

type bufferedMetric struct {
	sync.Mutex

	data []float64
	name string
	// Histograms and Distributions store tags as one string since we need
	// to compute its size multiple time when serializing.
	tags  string
	mtype metricType
}

func (s *bufferedMetric) sample(v float64) {
	s.Lock()
	defer s.Unlock()
	s.data = append(s.data, v)
}

func (s *bufferedMetric) flushUnsafe() metric {
	return metric{
		metricType: s.mtype,
		name:       s.name,
		stags:      s.tags,
		rate:       1,
		fvalues:    s.data,
	}
}

type histogramMetric = bufferedMetric

func newHistogramMetric(name string, value float64, stringTags string) *histogramMetric {
	return &histogramMetric{
		data:  []float64{value},
		name:  name,
		tags:  stringTags,
		mtype: histogramAggregated,
	}
}

type distributionMetric = bufferedMetric

func newDistributionMetric(name string, value float64, stringTags string) *distributionMetric {
	return &distributionMetric{
		data:  []float64{value},
		name:  name,
		tags:  stringTags,
		mtype: distributionAggregated,
	}
}

type timingMetric = bufferedMetric

func newTimingMetric(name string, value float64, stringTags string) *timingMetric {
	return &timingMetric{
		data:  []float64{value},
		name:  name,
		tags:  stringTags,
		mtype: timingAggregated,
	}
}
