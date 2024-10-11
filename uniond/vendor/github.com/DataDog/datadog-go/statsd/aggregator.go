package statsd

import (
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

type (
	countsMap         map[string]*countMetric
	gaugesMap         map[string]*gaugeMetric
	setsMap           map[string]*setMetric
	bufferedMetricMap map[string]*bufferedMetric
)

type aggregator struct {
	nbContextGauge int32
	nbContextCount int32
	nbContextSet   int32

	countsM sync.RWMutex
	gaugesM sync.RWMutex
	setsM   sync.RWMutex

	gauges        gaugesMap
	counts        countsMap
	sets          setsMap
	histograms    bufferedMetricContexts
	distributions bufferedMetricContexts
	timings       bufferedMetricContexts

	closed chan struct{}

	client *Client

	// aggregator implements ChannelMode mechanism to receive histograms,
	// distributions and timings. Since they need sampling they need to
	// lock for random. When using both ChannelMode and ExtendedAggregation
	// we don't want goroutine to fight over the lock.
	inputMetrics    chan metric
	stopChannelMode chan struct{}
	wg              sync.WaitGroup
}

type aggregatorMetrics struct {
	nbContext             int32
	nbContextGauge        int32
	nbContextCount        int32
	nbContextSet          int32
	nbContextHistogram    int32
	nbContextDistribution int32
	nbContextTiming       int32
}

func newAggregator(c *Client) *aggregator {
	return &aggregator{
		client:          c,
		counts:          countsMap{},
		gauges:          gaugesMap{},
		sets:            setsMap{},
		histograms:      newBufferedContexts(newHistogramMetric),
		distributions:   newBufferedContexts(newDistributionMetric),
		timings:         newBufferedContexts(newTimingMetric),
		closed:          make(chan struct{}),
		stopChannelMode: make(chan struct{}),
	}
}

func (a *aggregator) start(flushInterval time.Duration) {
	ticker := time.NewTicker(flushInterval)

	go func() {
		for {
			select {
			case <-ticker.C:
				a.flush()
			case <-a.closed:
				return
			}
		}
	}()
}

func (a *aggregator) startReceivingMetric(bufferSize int, nbWorkers int) {
	a.inputMetrics = make(chan metric, bufferSize)
	for i := 0; i < nbWorkers; i++ {
		a.wg.Add(1)
		go a.pullMetric()
	}
}

func (a *aggregator) stopReceivingMetric() {
	close(a.stopChannelMode)
	a.wg.Wait()
}

func (a *aggregator) stop() {
	a.closed <- struct{}{}
}

func (a *aggregator) pullMetric() {
	for {
		select {
		case m := <-a.inputMetrics:
			switch m.metricType {
			case histogram:
				a.histogram(m.name, m.fvalue, m.tags, m.rate)
			case distribution:
				a.distribution(m.name, m.fvalue, m.tags, m.rate)
			case timing:
				a.timing(m.name, m.fvalue, m.tags, m.rate)
			}
		case <-a.stopChannelMode:
			a.wg.Done()
			return
		}
	}
}

func (a *aggregator) flush() {
	for _, m := range a.flushMetrics() {
		a.client.sendBlocking(m)
	}
}

func (a *aggregator) flushTelemetryMetrics() *aggregatorMetrics {
	if a == nil {
		return nil
	}

	am := &aggregatorMetrics{
		nbContextGauge:        atomic.SwapInt32(&a.nbContextGauge, 0),
		nbContextCount:        atomic.SwapInt32(&a.nbContextCount, 0),
		nbContextSet:          atomic.SwapInt32(&a.nbContextSet, 0),
		nbContextHistogram:    a.histograms.resetAndGetNbContext(),
		nbContextDistribution: a.distributions.resetAndGetNbContext(),
		nbContextTiming:       a.timings.resetAndGetNbContext(),
	}

	am.nbContext = am.nbContextGauge + am.nbContextCount + am.nbContextSet + am.nbContextHistogram + am.nbContextDistribution + am.nbContextTiming
	return am
}

func (a *aggregator) flushMetrics() []metric {
	metrics := []metric{}

	// We reset the values to avoid sending 'zero' values for metrics not
	// sampled during this flush interval

	a.setsM.Lock()
	sets := a.sets
	a.sets = setsMap{}
	a.setsM.Unlock()

	for _, s := range sets {
		metrics = append(metrics, s.flushUnsafe()...)
	}

	a.gaugesM.Lock()
	gauges := a.gauges
	a.gauges = gaugesMap{}
	a.gaugesM.Unlock()

	for _, g := range gauges {
		metrics = append(metrics, g.flushUnsafe())
	}

	a.countsM.Lock()
	counts := a.counts
	a.counts = countsMap{}
	a.countsM.Unlock()

	for _, c := range counts {
		metrics = append(metrics, c.flushUnsafe())
	}

	metrics = a.histograms.flush(metrics)
	metrics = a.distributions.flush(metrics)
	metrics = a.timings.flush(metrics)

	atomic.AddInt32(&a.nbContextCount, int32(len(counts)))
	atomic.AddInt32(&a.nbContextGauge, int32(len(gauges)))
	atomic.AddInt32(&a.nbContextSet, int32(len(sets)))
	return metrics
}

func getContext(name string, tags []string) string {
	return name + ":" + strings.Join(tags, tagSeparatorSymbol)
}

func getContextAndTags(name string, tags []string) (string, string) {
	stringTags := strings.Join(tags, tagSeparatorSymbol)
	return name + ":" + stringTags, stringTags
}

func (a *aggregator) count(name string, value int64, tags []string) error {
	context := getContext(name, tags)
	a.countsM.RLock()
	if count, found := a.counts[context]; found {
		count.sample(value)
		a.countsM.RUnlock()
		return nil
	}
	a.countsM.RUnlock()

	a.countsM.Lock()
	// Check if another goroutines hasn't created the value betwen the RUnlock and 'Lock'
	if count, found := a.counts[context]; found {
		count.sample(value)
		a.countsM.Unlock()
		return nil
	}

	a.counts[context] = newCountMetric(name, value, tags)
	a.countsM.Unlock()
	return nil
}

func (a *aggregator) gauge(name string, value float64, tags []string) error {
	context := getContext(name, tags)
	a.gaugesM.RLock()
	if gauge, found := a.gauges[context]; found {
		gauge.sample(value)
		a.gaugesM.RUnlock()
		return nil
	}
	a.gaugesM.RUnlock()

	gauge := newGaugeMetric(name, value, tags)

	a.gaugesM.Lock()
	// Check if another goroutines hasn't created the value betwen the 'RUnlock' and 'Lock'
	if gauge, found := a.gauges[context]; found {
		gauge.sample(value)
		a.gaugesM.Unlock()
		return nil
	}
	a.gauges[context] = gauge
	a.gaugesM.Unlock()
	return nil
}

func (a *aggregator) set(name string, value string, tags []string) error {
	context := getContext(name, tags)
	a.setsM.RLock()
	if set, found := a.sets[context]; found {
		set.sample(value)
		a.setsM.RUnlock()
		return nil
	}
	a.setsM.RUnlock()

	a.setsM.Lock()
	// Check if another goroutines hasn't created the value betwen the 'RUnlock' and 'Lock'
	if set, found := a.sets[context]; found {
		set.sample(value)
		a.setsM.Unlock()
		return nil
	}
	a.sets[context] = newSetMetric(name, value, tags)
	a.setsM.Unlock()
	return nil
}

// Only histograms, distributions and timings are sampled with a rate since we
// only pack them in on message instead of aggregating them. Discarding the
// sample rate will have impacts on the CPU and memory usage of the Agent.

// type alias for Client.sendToAggregator
type bufferedMetricSampleFunc func(name string, value float64, tags []string, rate float64) error

func (a *aggregator) histogram(name string, value float64, tags []string, rate float64) error {
	return a.histograms.sample(name, value, tags, rate)
}

func (a *aggregator) distribution(name string, value float64, tags []string, rate float64) error {
	return a.distributions.sample(name, value, tags, rate)
}

func (a *aggregator) timing(name string, value float64, tags []string, rate float64) error {
	return a.timings.sample(name, value, tags, rate)
}
