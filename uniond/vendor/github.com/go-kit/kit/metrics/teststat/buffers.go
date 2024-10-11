package teststat

import (
	"bufio"
	"bytes"
	"io"
	"regexp"
	"strconv"

	"github.com/go-kit/kit/metrics/generic"
)

// SumLines expects a regex whose first capture group can be parsed as a
// float64. It will dump the WriterTo and parse each line, expecting to find a
// match. It returns the sum of all captured floats.
func SumLines(w io.WriterTo, regex string) func() float64 {
	return func() float64 {
		sum, _ := stats(w, regex, nil)
		return sum
	}
}

// LastLine expects a regex whose first capture group can be parsed as a
// float64. It will dump the WriterTo and parse each line, expecting to find a
// match. It returns the final captured float.
func LastLine(w io.WriterTo, regex string) func() []float64 {
	return func() []float64 {
		_, final := stats(w, regex, nil)
		return []float64{final}
	}
}

// Quantiles expects a regex whose first capture group can be parsed as a
// float64. It will dump the WriterTo and parse each line, expecting to find a
// match. It observes all captured floats into a generic.Histogram with the
// given number of buckets, and returns the 50th, 90th, 95th, and 99th quantiles
// from that histogram.
func Quantiles(w io.WriterTo, regex string, buckets int) func() (float64, float64, float64, float64) {
	return func() (float64, float64, float64, float64) {
		h := generic.NewHistogram("quantile-test", buckets)
		stats(w, regex, h)
		return h.Quantile(0.50), h.Quantile(0.90), h.Quantile(0.95), h.Quantile(0.99)
	}
}

func stats(w io.WriterTo, regex string, h *generic.Histogram) (sum, final float64) {
	re := regexp.MustCompile(regex)
	buf := &bytes.Buffer{}
	w.WriteTo(buf)
	s := bufio.NewScanner(buf)
	for s.Scan() {
		match := re.FindStringSubmatch(s.Text())
		f, err := strconv.ParseFloat(match[1], 64)
		if err != nil {
			panic(err)
		}
		sum += f
		final = f
		if h != nil {
			h.Observe(f)
		}
	}
	return sum, final
}
