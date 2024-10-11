// Package cloudwatch2 emits all data as a StatisticsSet (rather than
// a singular Value) to CloudWatch via the aws-sdk-go-v2 SDK.
package cloudwatch2

import (
	"context"
	"math"
	"sync"
	"time"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/service/cloudwatch"
	"github.com/aws/aws-sdk-go-v2/service/cloudwatch/types"
	"golang.org/x/sync/errgroup"

	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/internal/convert"
	"github.com/go-kit/kit/metrics/internal/lv"
	"github.com/go-kit/log"
)

const (
	maxConcurrentRequests = 20
)

// CloudWatchAPI is an interface that defines the set of Amazon CloudWatch API operations required by CloudWatch.
type CloudWatchAPI interface {
	PutMetricData(ctx context.Context, params *cloudwatch.PutMetricDataInput, optFns ...func(*cloudwatch.Options)) (*cloudwatch.PutMetricDataOutput, error)
}

// CloudWatch receives metrics observations and forwards them to CloudWatch.
// Create a CloudWatch object, use it to create metrics, and pass those metrics as
// dependencies to the components that will use them.
//
// To regularly report metrics to CloudWatch, use the WriteLoop helper method.
type CloudWatch struct {
	mtx                   sync.RWMutex
	sem                   chan struct{}
	namespace             string
	svc                   CloudWatchAPI
	counters              *lv.Space
	logger                log.Logger
	numConcurrentRequests int
}

// Option is a function adapter to change config of the CloudWatch struct
type Option func(*CloudWatch)

// WithLogger sets the Logger that will receive error messages generated
// during the WriteLoop. By default, no logger is used.
func WithLogger(logger log.Logger) Option {
	return func(cw *CloudWatch) {
		cw.logger = logger
	}
}

// WithConcurrentRequests sets the upper limit on how many
// cloudwatch.PutMetricDataRequest may be under way at any
// given time. If n is greater than 20, 20 is used. By default,
// the max is set at 10 concurrent requests.
func WithConcurrentRequests(n int) Option {
	return func(cw *CloudWatch) {
		if n > maxConcurrentRequests {
			n = maxConcurrentRequests
		}
		cw.numConcurrentRequests = n
	}
}

// New returns a CloudWatch object that may be used to create metrics.
// Namespace is applied to all created metrics and maps to the CloudWatch namespace.
// Callers must ensure that regular calls to Send are performed, either
// manually or with one of the helper methods.
func New(namespace string, svc CloudWatchAPI, options ...Option) *CloudWatch {
	cw := &CloudWatch{
		namespace:             namespace,
		svc:                   svc,
		counters:              lv.NewSpace(),
		numConcurrentRequests: 10,
		logger:                log.NewNopLogger(),
	}

	for _, optFunc := range options {
		optFunc(cw)
	}

	cw.sem = make(chan struct{}, cw.numConcurrentRequests)

	return cw
}

// NewCounter returns a counter. Observations are aggregated and emitted once
// per write invocation.
func (cw *CloudWatch) NewCounter(name string) metrics.Counter {
	return &Counter{
		name: name,
		obs:  cw.counters.Observe,
	}
}

// NewGauge returns an gauge. Under the covers, there is no distinctions
// in CloudWatch for how Counters/Histograms/Gauges are reported, so this
// just wraps a cloudwatch2.Counter.
func (cw *CloudWatch) NewGauge(name string) metrics.Gauge {
	return convert.NewCounterAsGauge(cw.NewCounter(name))
}

// NewHistogram returns a histogram. Under the covers, there is no distinctions
// in CloudWatch for how Counters/Histograms/Gauges are reported, so this
// just wraps a cloudwatch2.Counter.
func (cw *CloudWatch) NewHistogram(name string) metrics.Histogram {
	return convert.NewCounterAsHistogram(cw.NewCounter(name))
}

// WriteLoop is a helper method that invokes Send every time the passed
// channel fires. This method blocks until ctx is canceled, so clients
// probably want to run it in its own goroutine. For typical usage, create a
// time.Ticker and pass its C channel to this method.
func (cw *CloudWatch) WriteLoop(ctx context.Context, c <-chan time.Time) {
	for {
		select {
		case <-c:
			if err := cw.Send(); err != nil {
				cw.logger.Log("during", "Send", "err", err)
			}
		case <-ctx.Done():
			return
		}
	}
}

// Send will fire an API request to CloudWatch with the latest stats for
// all metrics. It is preferred that the WriteLoop method is used.
func (cw *CloudWatch) Send() error {
	cw.mtx.RLock()
	defer cw.mtx.RUnlock()
	now := time.Now()

	var datums []types.MetricDatum

	cw.counters.Reset().Walk(func(name string, lvs lv.LabelValues, values []float64) bool {
		datums = append(datums, types.MetricDatum{
			MetricName:      aws.String(name),
			Dimensions:      makeDimensions(lvs...),
			StatisticValues: stats(values),
			Timestamp:       aws.Time(now),
		})
		return true
	})

	var batches [][]types.MetricDatum
	for len(datums) > 0 {
		var batch []types.MetricDatum
		lim := len(datums)
		if lim > maxConcurrentRequests {
			lim = maxConcurrentRequests
		}
		batch, datums = datums[:lim], datums[lim:]
		batches = append(batches, batch)
	}

	var g errgroup.Group
	for _, batch := range batches {
		batch := batch
		g.Go(func() error {
			cw.sem <- struct{}{}
			defer func() {
				<-cw.sem
			}()
			_, err := cw.svc.PutMetricData(context.TODO(), &cloudwatch.PutMetricDataInput{
				Namespace:  aws.String(cw.namespace),
				MetricData: batch,
			})
			return err
		})
	}
	return g.Wait()
}

var zero = float64(0.0)

// Just build this once to reduce construction costs whenever
// someone does a Send with no aggregated values.
var zeros = types.StatisticSet{
	Maximum:     &zero,
	Minimum:     &zero,
	Sum:         &zero,
	SampleCount: &zero,
}

func stats(a []float64) *types.StatisticSet {
	count := float64(len(a))
	if count == 0 {
		return &zeros
	}

	var sum float64
	var min = math.MaxFloat64
	var max = math.MaxFloat64 * -1
	for _, f := range a {
		sum += f
		if f < min {
			min = f
		}
		if f > max {
			max = f
		}
	}

	return &types.StatisticSet{
		Maximum:     &max,
		Minimum:     &min,
		Sum:         &sum,
		SampleCount: &count,
	}
}

func makeDimensions(labelValues ...string) []types.Dimension {
	dimensions := make([]types.Dimension, len(labelValues)/2)
	for i, j := 0, 0; i < len(labelValues); i, j = i+2, j+1 {
		dimensions[j] = types.Dimension{
			Name:  aws.String(labelValues[i]),
			Value: aws.String(labelValues[i+1]),
		}
	}
	return dimensions
}

type observeFunc func(name string, lvs lv.LabelValues, value float64)

// Counter is a counter. Observations are forwarded to a node
// object, and aggregated per timeseries.
type Counter struct {
	name string
	lvs  lv.LabelValues
	obs  observeFunc
}

// With implements metrics.Counter.
func (c *Counter) With(labelValues ...string) metrics.Counter {
	return &Counter{
		name: c.name,
		lvs:  c.lvs.With(labelValues...),
		obs:  c.obs,
	}
}

// Add implements metrics.Counter.
func (c *Counter) Add(delta float64) {
	c.obs(c.name, c.lvs, delta)
}
