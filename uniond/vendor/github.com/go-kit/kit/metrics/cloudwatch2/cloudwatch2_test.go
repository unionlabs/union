package cloudwatch2

import (
	"context"
	"strings"
	"testing"

	"github.com/aws/aws-sdk-go-v2/service/cloudwatch"
	"github.com/aws/aws-sdk-go-v2/service/cloudwatch/types"
)

func TestStats(t *testing.T) {
	testCases := []struct {
		name string
		vals []float64
		xMin float64
		xMax float64
		xSum float64
		xCt  float64
	}{
		{
			"empty",
			[]float64{},
			0.0,
			0.0,
			0.0,
			0.0,
		},
		{
			"single",
			[]float64{3.1416},
			3.1416,
			3.1416,
			3.1416,
			1.0,
		},
		{
			"double",
			[]float64{1.0, 9.0},
			1.0,
			9.0,
			10.0,
			2.0,
		},
		{
			"multiple",
			[]float64{5.0, 1.0, 9.0, 5.0},
			1.0,
			9.0,
			20.0,
			4.0,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			s := stats(tc.vals)
			if tc.xMin != *s.Minimum {
				t.Errorf("expected [%f]: %f\n", tc.xMin, *s.Minimum)
			}
			if tc.xMax != *s.Maximum {
				t.Errorf("expected [%f]: %f\n", tc.xMax, *s.Maximum)
			}
			if tc.xSum != *s.Sum {
				t.Errorf("expected [%f]: %f\n", tc.xSum, *s.Sum)
			}
			if tc.xCt != *s.SampleCount {
				t.Errorf("expected [%f]: %f\n", tc.xCt, *s.SampleCount)
			}
		})
	}
}

type mockCloudWatch struct {
	CloudWatchAPI
	latestName string
	latestData []types.MetricDatum
}

func (mcw *mockCloudWatch) PutMetricData(ctx context.Context, params *cloudwatch.PutMetricDataInput, optFns ...func(*cloudwatch.Options)) (*cloudwatch.PutMetricDataOutput, error) {
	mcw.latestName = *params.Namespace
	mcw.latestData = params.MetricData

	return nil, nil
}

func TestSend(t *testing.T) {
	ns := "example-namespace"
	svc := &mockCloudWatch{}
	cw := New(ns, svc)

	c := cw.NewCounter("c").With("charlie", "cat")
	h := cw.NewHistogram("h").With("hotel", "horse")
	g := cw.NewGauge("g").With("golf", "giraffe")

	c.Add(4.0)
	c.Add(5.0)
	c.Add(6.0)
	h.Observe(3.0)
	h.Observe(5.0)
	h.Observe(7.0)
	g.Set(2.0)
	g.Set(5.0)
	g.Set(8.0)

	err := cw.Send()
	if err != nil {
		t.Fatalf("unexpected: %v\n", err)
	}

	if ns != svc.latestName {
		t.Errorf("expected namespace %q; not %q\n", ns, svc.latestName)
	}

	if len(svc.latestData) != 3 {
		t.Errorf("expected 3 datums: %v\n", svc.latestData)
	}
	for _, datum := range svc.latestData {
		initial := *datum.MetricName
		if len(datum.Dimensions) != 1 {
			t.Errorf("expected 1 dimension: %v\n", datum)
		}
		if !strings.HasPrefix(*datum.Dimensions[0].Name, initial) {
			t.Errorf("expected %q in Name of %v\n", initial, datum.Dimensions)
		}
		if !strings.HasPrefix(*datum.Dimensions[0].Value, initial) {
			t.Errorf("expected %q in Value of %v\n", initial, datum.Dimensions)
		}
		if datum.StatisticValues == nil {
			t.Errorf("expected StatisticValues in %v\n", datum)
		}
		if *datum.StatisticValues.Sum != 15.0 {
			t.Errorf("expected 15.0 for Sum in %v\n", datum)
		}
		if *datum.StatisticValues.SampleCount != 3.0 {
			t.Errorf("expected 3.0 for SampleCount in %v\n", datum)
		}
	}
}
