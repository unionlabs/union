// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package tlog

import (
	"fmt"
	"testing"
)

// FuzzParseTilePath tests that ParseTilePath never crashes
func FuzzParseTilePath(f *testing.F) {
	f.Add("tile/4/0/001")
	f.Add("tile/4/0/001.p/5")
	f.Add("tile/3/5/x123/x456/078")
	f.Add("tile/3/5/x123/x456/078.p/2")
	f.Add("tile/1/0/x003/x057/500")
	f.Add("tile/3/5/123/456/078")
	f.Add("tile/3/-1/123/456/078")
	f.Add("tile/1/data/x003/x057/500")
	f.Fuzz(func(t *testing.T, path string) {
		ParseTilePath(path)
	})
}

func TestNewTilesForSize(t *testing.T) {
	for _, tt := range []struct {
		old, new int64
		want     int
	}{
		{1, 1, 0},
		{100, 101, 1},
		{1023, 1025, 3},
		{1024, 1030, 1},
		{1030, 2000, 1},
		{1030, 10000, 10},
		{49516517, 49516586, 3},
	} {
		t.Run(fmt.Sprintf("%d-%d", tt.old, tt.new), func(t *testing.T) {
			tiles := NewTiles(10, tt.old, tt.new)
			if got := len(tiles); got != tt.want {
				t.Errorf("got %d, want %d", got, tt.want)
				for _, tile := range tiles {
					t.Logf("%+v", tile)
				}
			}
		})
	}
}
