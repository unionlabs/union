package internal

import (
	"bytes"
	"encoding/gob"
	"fmt"
	metrics2 "runtime/metrics"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/metrics"
)

func TestCollectRuntimeMetrics(t *testing.T) {
	t.Skip("Only used for generating testdata")
	serialize := func(path string, histogram *metrics2.Float64Histogram) {
		var f = new(bytes.Buffer)
		if err := gob.NewEncoder(f).Encode(histogram); err != nil {
			panic(err)
		}
		fmt.Printf("var %v = %q\n", path, f.Bytes())
	}
	time.Sleep(2 * time.Second)
	stats := metrics.ReadRuntimeStats()
	serialize("schedlatency", stats.SchedLatency)
	serialize("gcpauses", stats.GCPauses)
}
