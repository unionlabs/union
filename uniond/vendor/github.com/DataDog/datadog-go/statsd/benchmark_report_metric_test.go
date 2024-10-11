// +build go1.13

package statsd_test

import "testing"

func reportMetric(b *testing.B, value float64, unit string) {
	b.ReportMetric(value, unit)
}
