// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package replay

import (
	"bytes"
	"fmt"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/stretchr/testify/require"
)

func TestSampledMetric(t *testing.T) {
	var m SampledMetric
	var buf bytes.Buffer
	datadriven.RunTest(t, "testdata/sampled_metric", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "init":
			m = SampledMetric{samples: m.samples[:0]}
			var cumDur time.Duration
			for _, line := range strings.Split(td.Input, "\n") {
				fields := strings.Fields(line)
				v, err := strconv.ParseInt(fields[0], 10, 64)
				require.NoError(t, err)
				dur, err := time.ParseDuration(fields[1])
				require.NoError(t, err)
				cumDur += dur
				m.samples = append(m.samples, sample{
					value: v,
					since: cumDur,
				})
			}
			return ""
		case "values":
			buf.Reset()
			var width int
			td.ScanArgs(t, "width", &width)
			for i, v := range m.Values(width) {
				if i > 0 {
					fmt.Fprint(&buf, " ")
				}
				fmt.Fprintf(&buf, "%.1f", v)
			}
			return buf.String()
		case "plot":
			var width, height int
			var scaleStr string
			td.ScanArgs(t, "width", &width)
			td.ScanArgs(t, "height", &height)
			td.ScanArgs(t, "scale", &scaleStr)
			var scale float64
			_, err := fmt.Sscanf(scaleStr, "%f", &scale)
			require.NoError(t, err)
			return m.Plot(width, height, scale)
		case "plot-increasing-per-sec":
			var width, height int
			var scaleStr string
			td.ScanArgs(t, "width", &width)
			td.ScanArgs(t, "height", &height)
			td.ScanArgs(t, "scale", &scaleStr)
			var scale float64
			_, err := fmt.Sscanf(scaleStr, "%f", &scale)
			require.NoError(t, err)
			return m.PlotIncreasingPerSec(width, height, scale)
		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}
