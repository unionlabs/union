package influx

import (
	"bytes"
	"fmt"
	"regexp"
	"strconv"
	"strings"
	"testing"

	influxdb "github.com/influxdata/influxdb1-client/v2"

	"github.com/go-kit/kit/metrics/teststat"
	"github.com/go-kit/log"
)

func TestCounter(t *testing.T) {
	in := New(map[string]string{"a": "b"}, influxdb.BatchPointsConfig{}, log.NewNopLogger())
	re := regexp.MustCompile(`influx_counter,a=b count=([0-9\.]+) [0-9]+`) // reverse-engineered :\
	counter := in.NewCounter("influx_counter")
	value := func() float64 {
		client := &bufWriter{}
		in.WriteTo(client)
		match := re.FindStringSubmatch(client.buf.String())
		f, _ := strconv.ParseFloat(match[1], 64)
		return f
	}
	if err := teststat.TestCounter(counter, value); err != nil {
		t.Fatal(err)
	}
}

func TestGauge(t *testing.T) {
	in := New(map[string]string{"foo": "alpha"}, influxdb.BatchPointsConfig{}, log.NewNopLogger())
	re := regexp.MustCompile(`influx_gauge,foo=alpha value=([0-9\.]+) [0-9]+`)
	gauge := in.NewGauge("influx_gauge")
	value := func() []float64 {
		client := &bufWriter{}
		in.WriteTo(client)
		match := re.FindStringSubmatch(client.buf.String())
		f, _ := strconv.ParseFloat(match[1], 64)
		return []float64{f}
	}
	if err := teststat.TestGauge(gauge, value); err != nil {
		t.Fatal(err)
	}
}

func TestHistogram(t *testing.T) {
	in := New(map[string]string{"foo": "alpha"}, influxdb.BatchPointsConfig{}, log.NewNopLogger())
	re := regexp.MustCompile(`influx_histogram,bar=beta,foo=alpha p50=([0-9\.]+),p90=([0-9\.]+),p95=([0-9\.]+),p99=([0-9\.]+) [0-9]+`)
	histogram := in.NewHistogram("influx_histogram").With("bar", "beta")
	quantiles := func() (float64, float64, float64, float64) {
		w := &bufWriter{}
		in.WriteTo(w)
		match := re.FindStringSubmatch(w.buf.String())
		if len(match) != 5 {
			t.Errorf("These are not the quantiles you're looking for: %v\n", match)
		}
		var result [4]float64
		for i, q := range match[1:] {
			result[i], _ = strconv.ParseFloat(q, 64)
		}
		return result[0], result[1], result[2], result[3]
	}
	if err := teststat.TestHistogram(histogram, quantiles, 0.01); err != nil {
		t.Fatal(err)
	}
}

func TestHistogramLabels(t *testing.T) {
	in := New(map[string]string{}, influxdb.BatchPointsConfig{}, log.NewNopLogger())
	h := in.NewHistogram("foo")
	h.Observe(123)
	h.With("abc", "xyz").Observe(456)
	w := &bufWriter{}
	if err := in.WriteTo(w); err != nil {
		t.Fatal(err)
	}
	if want, have := 2, len(strings.Split(strings.TrimSpace(w.buf.String()), "\n")); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}

func TestIssue404(t *testing.T) {
	in := New(map[string]string{}, influxdb.BatchPointsConfig{}, log.NewNopLogger())

	counterOne := in.NewCounter("influx_counter_one").With("a", "b")
	counterOne.Add(123)

	counterTwo := in.NewCounter("influx_counter_two").With("c", "d")
	counterTwo.Add(456)

	w := &bufWriter{}
	in.WriteTo(w)

	lines := strings.Split(strings.TrimSpace(w.buf.String()), "\n")
	if want, have := 2, len(lines); want != have {
		t.Fatalf("want %d, have %d", want, have)
	}
	for _, line := range lines {
		if strings.HasPrefix(line, "influx_counter_one") {
			if !strings.HasPrefix(line, "influx_counter_one,a=b count=123 ") {
				t.Errorf("invalid influx_counter_one: %s", line)
			}
		} else if strings.HasPrefix(line, "influx_counter_two") {
			if !strings.HasPrefix(line, "influx_counter_two,c=d count=456 ") {
				t.Errorf("invalid influx_counter_two: %s", line)
			}
		} else {
			t.Errorf("unexpected line: %s", line)
		}
	}
}

type bufWriter struct {
	buf bytes.Buffer
}

func (w *bufWriter) Write(bp influxdb.BatchPoints) error {
	for _, p := range bp.Points() {
		fmt.Fprintf(&w.buf, p.String()+"\n")
	}
	return nil
}
