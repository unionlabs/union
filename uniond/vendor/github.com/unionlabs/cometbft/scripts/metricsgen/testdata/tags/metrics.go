package tags

import "github.com/go-kit/kit/metrics"

//go:generate go run ../../../../scripts/metricsgen -struct=Metrics

type Metrics struct {
	WithLabels     metrics.Counter   `metrics_labels:"step,time"`
	WithExpBuckets metrics.Histogram `metrics_buckettype:"exp" metrics_bucketsizes:".1,100,8"`
	WithBuckets    metrics.Histogram `metrics_bucketsizes:"1, 2, 3, 4, 5"`
	Named          metrics.Counter   `metrics_name:"metric_with_name"`
}
