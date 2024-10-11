// +build !go1.13

package statsd_test

import "testing"

func reportMetric(*testing.B, float64, string) {}
