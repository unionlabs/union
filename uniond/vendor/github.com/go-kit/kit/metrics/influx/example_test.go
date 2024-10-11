package influx

import (
	"fmt"
	"regexp"

	influxdb "github.com/influxdata/influxdb1-client/v2"

	"github.com/go-kit/log"
)

func ExampleCounter() {
	in := New(map[string]string{"a": "b"}, influxdb.BatchPointsConfig{}, log.NewNopLogger())
	counter := in.NewCounter("influx_counter")
	counter.Add(10)
	counter.With("error", "true").Add(1)
	counter.With("error", "false").Add(2)
	counter.Add(50)

	client := &bufWriter{}
	in.WriteTo(client)

	expectedLines := []string{
		`(influx_counter,a=b count=60) [0-9]{19}`,
		`(influx_counter,a=b,error=true count=1) [0-9]{19}`,
		`(influx_counter,a=b,error=false count=2) [0-9]{19}`,
	}

	if err := extractAndPrintMessage(expectedLines, client.buf.String()); err != nil {
		fmt.Println(err.Error())
	}

	// Output:
	// influx_counter,a=b count=60
	// influx_counter,a=b,error=true count=1
	// influx_counter,a=b,error=false count=2
}

func ExampleGauge() {
	in := New(map[string]string{"a": "b"}, influxdb.BatchPointsConfig{}, log.NewNopLogger())
	gauge := in.NewGauge("influx_gauge")
	gauge.Set(10)
	gauge.With("error", "true").Set(2)
	gauge.With("error", "true").Set(1)
	gauge.With("error", "false").Set(2)
	gauge.Set(50)
	gauge.With("test", "true").Set(1)
	gauge.With("test", "true").Add(1)

	client := &bufWriter{}
	in.WriteTo(client)

	expectedLines := []string{
		`(influx_gauge,a=b,test=true value=2) [0-9]{19}`,
		`(influx_gauge,a=b value=50) [0-9]{19}`,
		`(influx_gauge,a=b,error=true value=1) [0-9]{19}`,
		`(influx_gauge,a=b,error=false value=2) [0-9]{19}`,
	}

	if err := extractAndPrintMessage(expectedLines, client.buf.String()); err != nil {
		fmt.Println(err.Error())
	}

	// Output:
	// influx_gauge,a=b,test=true value=2
	// influx_gauge,a=b value=50
	// influx_gauge,a=b,error=true value=1
	// influx_gauge,a=b,error=false value=2
}

func ExampleHistogram() {
	in := New(map[string]string{"foo": "alpha"}, influxdb.BatchPointsConfig{}, log.NewNopLogger())
	histogram := in.NewHistogram("influx_histogram")
	histogram.Observe(float64(10))
	histogram.With("error", "true").Observe(float64(1))
	histogram.With("error", "false").Observe(float64(2))
	histogram.Observe(float64(50))

	client := &bufWriter{}
	in.WriteTo(client)

	expectedLines := []string{
		`(influx_histogram,foo=alpha p50=10,p90=50,p95=50,p99=50) [0-9]{19}`,
		`(influx_histogram,error=true,foo=alpha p50=1,p90=1,p95=1,p99=1) [0-9]{19}`,
		`(influx_histogram,error=false,foo=alpha p50=2,p90=2,p95=2,p99=2) [0-9]{19}`,
	}

	if err := extractAndPrintMessage(expectedLines, client.buf.String()); err != nil {
		fmt.Println(err.Error())
	}

	// Output:
	// influx_histogram,foo=alpha p50=10,p90=50,p95=50,p99=50
	// influx_histogram,error=true,foo=alpha p50=1,p90=1,p95=1,p99=1
	// influx_histogram,error=false,foo=alpha p50=2,p90=2,p95=2,p99=2
}

func extractAndPrintMessage(expected []string, msg string) error {
	for _, pattern := range expected {
		re := regexp.MustCompile(pattern)
		match := re.FindStringSubmatch(msg)
		if len(match) != 2 {
			return fmt.Errorf("pattern not found! {%s} [%s]: %v\n", pattern, msg, match)
		}
		fmt.Println(match[1])
	}
	return nil
}
