package cloudwatch

import (
	"errors"
	"fmt"
	"strconv"
	"sync"
	"testing"

	"github.com/aws/aws-sdk-go/service/cloudwatch"
	"github.com/aws/aws-sdk-go/service/cloudwatch/cloudwatchiface"

	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/teststat"
	"github.com/go-kit/log"
)

const metricNameToGenerateError = "metric_name_used_to_throw_an_error"

var errTest = errors.New("test error")

type mockCloudWatch struct {
	cloudwatchiface.CloudWatchAPI
	mtx                sync.RWMutex
	valuesReceived     map[string][]float64
	dimensionsReceived map[string][]*cloudwatch.Dimension
}

func newMockCloudWatch() *mockCloudWatch {
	return &mockCloudWatch{
		valuesReceived:     map[string][]float64{},
		dimensionsReceived: map[string][]*cloudwatch.Dimension{},
	}
}

func (mcw *mockCloudWatch) PutMetricData(input *cloudwatch.PutMetricDataInput) (*cloudwatch.PutMetricDataOutput, error) {
	mcw.mtx.Lock()
	defer mcw.mtx.Unlock()
	for _, datum := range input.MetricData {
		if *datum.MetricName == metricNameToGenerateError {
			return nil, errTest
		}

		if len(datum.Values) > 0 {
			for _, v := range datum.Values {
				mcw.valuesReceived[*datum.MetricName] = append(mcw.valuesReceived[*datum.MetricName], *v)
			}
		} else {
			mcw.valuesReceived[*datum.MetricName] = append(mcw.valuesReceived[*datum.MetricName], *datum.Value)
		}
		mcw.dimensionsReceived[*datum.MetricName] = datum.Dimensions
	}
	return nil, nil
}

func (mcw *mockCloudWatch) testDimensions(name string, labelValues ...string) error {
	mcw.mtx.RLock()
	_, hasValue := mcw.valuesReceived[name]
	if !hasValue {
		return nil // nothing to check; 0 samples were received
	}
	dimensions, ok := mcw.dimensionsReceived[name]
	mcw.mtx.RUnlock()

	if !ok {
		if len(labelValues) > 0 {
			return errors.New("Expected dimensions to be available, but none were")
		}
	}
LabelValues:
	for i, j := 0, 0; i < len(labelValues); i, j = i+2, j+1 {
		name, value := labelValues[i], labelValues[i+1]
		for _, dimension := range dimensions {
			if *dimension.Name == name {
				if *dimension.Value == value {
					break LabelValues
				}
			}
		}
		return fmt.Errorf("could not find dimension with name %s and value %s", name, value)
	}

	return nil
}

func TestCounter(t *testing.T) {
	namespace, name := "abc", "def"
	label, value := "label", "value"
	svc := newMockCloudWatch()
	cw := New(namespace, svc, WithLogger(log.NewNopLogger()))
	counter := cw.NewCounter(name).With(label, value)
	valuef := func() float64 {
		if err := cw.Send(); err != nil {
			t.Fatal(err)
		}
		svc.mtx.RLock()
		defer svc.mtx.RUnlock()
		value := svc.valuesReceived[name][len(svc.valuesReceived[name])-1]
		delete(svc.valuesReceived, name)

		return value
	}
	if err := teststat.TestCounter(counter, valuef); err != nil {
		t.Fatal(err)
	}
	if err := teststat.TestCounter(counter, valuef); err != nil {
		t.Fatal("Fill and flush counter 2nd time: ", err)
	}
	if err := svc.testDimensions(name, label, value); err != nil {
		t.Fatal(err)
	}
}

func TestCounterLowSendConcurrency(t *testing.T) {
	namespace := "abc"
	var names, labels, values []string
	for i := 1; i <= 45; i++ {
		num := strconv.Itoa(i)
		names = append(names, "name"+num)
		labels = append(labels, "label"+num)
		values = append(values, "value"+num)
	}
	svc := newMockCloudWatch()
	cw := New(namespace, svc,
		WithLogger(log.NewNopLogger()),
		WithConcurrentRequests(2),
	)

	counters := make(map[string]metrics.Counter)
	var wants []float64
	for i, name := range names {
		counters[name] = cw.NewCounter(name).With(labels[i], values[i])
		wants = append(wants, teststat.FillCounter(counters[name]))
	}

	if err := cw.Send(); err != nil {
		t.Fatal(err)
	}

	for i, name := range names {
		if l := len(svc.valuesReceived[name]); l == 0 && wants[i] == 0 {
			continue
		} else if l != 1 {
			t.Fatalf("one value expected, got %d", l)
		}

		if svc.valuesReceived[name][0] != wants[i] {
			t.Fatalf("want %f, have %f", wants[i], svc.valuesReceived[name])
		}
		if err := svc.testDimensions(name, labels[i], values[i]); err != nil {
			t.Fatal(err)
		}
	}
}

func TestGauge(t *testing.T) {
	namespace, name := "abc", "def"
	label, value := "label", "value"
	svc := newMockCloudWatch()
	cw := New(namespace, svc, WithLogger(log.NewNopLogger()))
	gauge := cw.NewGauge(name).With(label, value)
	valuef := func() []float64 {
		if err := cw.Send(); err != nil {
			t.Fatal(err)
		}
		svc.mtx.RLock()
		defer svc.mtx.RUnlock()
		res := svc.valuesReceived[name]
		delete(svc.valuesReceived, name)
		return res
	}

	if err := teststat.TestGauge(gauge, valuef); err != nil {
		t.Fatal(err)
	}
	if err := svc.testDimensions(name, label, value); err != nil {
		t.Fatal(err)
	}
}

func TestHistogram(t *testing.T) {
	namespace, name := "abc", "def"
	label, value := "label", "value"
	svc := newMockCloudWatch()
	cw := New(namespace, svc, WithLogger(log.NewNopLogger()))
	histogram := cw.NewHistogram(name).With(label, value)
	n50 := fmt.Sprintf("%s_50", name)
	n90 := fmt.Sprintf("%s_90", name)
	n95 := fmt.Sprintf("%s_95", name)
	n99 := fmt.Sprintf("%s_99", name)
	quantiles := func() (p50, p90, p95, p99 float64) {
		err := cw.Send()
		if err != nil {
			t.Fatal(err)
		}

		svc.mtx.RLock()
		defer svc.mtx.RUnlock()
		if len(svc.valuesReceived[n50]) > 0 {
			p50 = svc.valuesReceived[n50][0]
			delete(svc.valuesReceived, n50)
		}

		if len(svc.valuesReceived[n90]) > 0 {
			p90 = svc.valuesReceived[n90][0]
			delete(svc.valuesReceived, n90)
		}

		if len(svc.valuesReceived[n95]) > 0 {
			p95 = svc.valuesReceived[n95][0]
			delete(svc.valuesReceived, n95)
		}

		if len(svc.valuesReceived[n99]) > 0 {
			p99 = svc.valuesReceived[n99][0]
			delete(svc.valuesReceived, n99)
		}
		return
	}
	if err := teststat.TestHistogram(histogram, quantiles, 0.01); err != nil {
		t.Fatal(err)
	}
	if err := svc.testDimensions(n50, label, value); err != nil {
		t.Fatal(err)
	}
	if err := svc.testDimensions(n90, label, value); err != nil {
		t.Fatal(err)
	}
	if err := svc.testDimensions(n95, label, value); err != nil {
		t.Fatal(err)
	}
	if err := svc.testDimensions(n99, label, value); err != nil {
		t.Fatal(err)
	}

	// now test with only 2 custom percentiles
	//
	svc = newMockCloudWatch()
	cw = New(namespace, svc, WithLogger(log.NewNopLogger()), WithPercentiles(0.50, 0.90))
	histogram = cw.NewHistogram(name).With(label, value)

	customQuantiles := func() (p50, p90, p95, p99 float64) {
		err := cw.Send()
		if err != nil {
			t.Fatal(err)
		}
		svc.mtx.RLock()
		defer svc.mtx.RUnlock()
		if len(svc.valuesReceived[n50]) > 0 {
			p50 = svc.valuesReceived[n50][0]
			delete(svc.valuesReceived, n50)
		}
		if len(svc.valuesReceived[n90]) > 0 {
			p90 = svc.valuesReceived[n90][0]
			delete(svc.valuesReceived, n90)
		}

		// our teststat.TestHistogram wants us to give p95 and p99,
		// but with custom percentiles we don't have those.
		// So fake them. Maybe we should make teststat.nvq() public and use that?
		p95 = 541.121341
		p99 = 558.158697

		// but fail if they are actually set (because that would mean the
		// WithPercentiles() is not respected)
		if _, isSet := svc.valuesReceived[n95]; isSet {
			t.Fatal("p95 should not be set")
		}
		if _, isSet := svc.valuesReceived[n99]; isSet {
			t.Fatal("p99 should not be set")
		}
		return
	}
	if err := teststat.TestHistogram(histogram, customQuantiles, 0.01); err != nil {
		t.Fatal(err)
	}
	if err := svc.testDimensions(n50, label, value); err != nil {
		t.Fatal(err)
	}
	if err := svc.testDimensions(n90, label, value); err != nil {
		t.Fatal(err)
	}
	if err := svc.testDimensions(n95, label, value); err != nil {
		t.Fatal(err)
	}
	if err := svc.testDimensions(n99, label, value); err != nil {
		t.Fatal(err)
	}
}

func TestErrorLog(t *testing.T) {
	namespace := "abc"
	svc := newMockCloudWatch()
	cw := New(namespace, svc, WithLogger(log.NewNopLogger()))
	cw.NewGauge(metricNameToGenerateError).Set(123)
	if err := cw.Send(); err != errTest {
		t.Fatal("Expected error, but didn't get one")
	}
}
