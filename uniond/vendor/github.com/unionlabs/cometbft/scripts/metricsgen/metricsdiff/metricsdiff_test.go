package main_test

import (
	"bytes"
	"io"
	"testing"

	"github.com/stretchr/testify/require"

	metricsdiff "github.com/cometbft/cometbft/scripts/metricsgen/metricsdiff" //nolint:revive // this only works with metricsdiff in front
)

func TestDiff(t *testing.T) {
	for _, tc := range []struct {
		name      string
		aContents string
		bContents string

		want string
	}{
		{
			name: "labels",
			aContents: `
			metric_one{label_one="content", label_two="content"} 0
			`,
			bContents: `
			metric_one{label_three="content", label_four="content"} 0
			`,
			want: `Label changes:
Metric: metric_one
+++ label_three
+++ label_four
--- label_one
--- label_two
`,
		},
		{
			name: "metrics",
			aContents: `
			metric_one{label_one="content"} 0
			`,
			bContents: `
			metric_two{label_two="content"} 0
			`,
			want: `Metric changes:
+++ metric_two
--- metric_one
`,
		},
	} {
		t.Run(tc.name, func(t *testing.T) {
			bufA := bytes.NewBuffer([]byte{})
			bufB := bytes.NewBuffer([]byte{})
			_, err := io.WriteString(bufA, tc.aContents)
			require.NoError(t, err)
			_, err = io.WriteString(bufB, tc.bContents)
			require.NoError(t, err)
			md, err := metricsdiff.DiffFromReaders(bufA, bufB)
			require.NoError(t, err)
			require.Equal(t, tc.want, md.String())
		})
	}
}
