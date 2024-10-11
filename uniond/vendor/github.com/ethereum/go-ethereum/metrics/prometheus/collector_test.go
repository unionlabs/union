// Copyright 2023 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package prometheus

import (
	"fmt"
	"os"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/metrics"
	"github.com/ethereum/go-ethereum/metrics/internal"
)

func TestMain(m *testing.M) {
	metrics.Enabled = true
	os.Exit(m.Run())
}

func TestCollector(t *testing.T) {
	var (
		c    = newCollector()
		want string
	)
	internal.ExampleMetrics().Each(func(name string, i interface{}) {
		c.Add(name, i)
	})
	if wantB, err := os.ReadFile("./testdata/prometheus.want"); err != nil {
		t.Fatal(err)
	} else {
		want = string(wantB)
	}
	if have := c.buff.String(); have != want {
		t.Logf("have\n%v", have)
		t.Logf("have vs want:\n%v", findFirstDiffPos(have, want))
		t.Fatalf("unexpected collector output")
	}
}

func findFirstDiffPos(a, b string) string {
	yy := strings.Split(b, "\n")
	for i, x := range strings.Split(a, "\n") {
		if i >= len(yy) {
			return fmt.Sprintf("have:%d: %s\nwant:%d: <EOF>", i, x, i)
		}
		if y := yy[i]; x != y {
			return fmt.Sprintf("have:%d: %s\nwant:%d: %s", i, x, i, y)
		}
	}
	return ""
}
