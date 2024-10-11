// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package main

import (
	"bytes"
	"fmt"
	"reflect"
	"testing"
)

func TestRun(t *testing.T) {
	out := &bytes.Buffer{}
	in := bytes.NewBuffer([]byte(`
test.com/A@v1.0.0 test.com/B@v1.2.3
test.com/B@v1.0.0 test.com/C@v4.5.6
`))
	if err := modgraphviz(in, out); err != nil {
		t.Fatal(err)
	}

	gotGraph := string(out.Bytes())
	wantGraph := `digraph gomodgraph {
	node [ shape=rectangle fontsize=12 ]
	"test.com/A@v1.0.0" -> "test.com/B@v1.2.3"
	"test.com/B@v1.0.0" -> "test.com/C@v4.5.6"
	"test.com/A@v1.0.0" [style = filled, fillcolor = green]
	"test.com/B@v1.2.3" [style = filled, fillcolor = green]
	"test.com/C@v4.5.6" [style = filled, fillcolor = green]
	"test.com/B@v1.0.0" [style = filled, fillcolor = gray]
}
`
	if gotGraph != wantGraph {
		t.Fatalf("\ngot: %s\nwant: %s", gotGraph, wantGraph)
	}
}

func TestMVSPicking(t *testing.T) {
	for _, tc := range []struct {
		name         string
		in           []string
		wantPicked   []string
		wantUnpicked []string
	}{
		{
			name:         "single node",
			in:           []string{"foo@v0.0.1"},
			wantPicked:   []string{"foo@v0.0.1"},
			wantUnpicked: nil,
		},
		{
			name:         "duplicate same node",
			in:           []string{"foo@v0.0.1", "foo@v0.0.1"},
			wantPicked:   []string{"foo@v0.0.1"},
			wantUnpicked: nil,
		},
		{
			name:         "multiple semver - same major",
			in:           []string{"foo@v1.0.0", "foo@v1.3.7", "foo@v1.2.0", "foo@v1.0.1"},
			wantPicked:   []string{"foo@v1.3.7"},
			wantUnpicked: []string{"foo@v1.0.0", "foo@v1.2.0", "foo@v1.0.1"},
		},
		{
			name:         "multiple semver - multiple major",
			in:           []string{"foo@v1.0.0", "foo@v1.3.7", "foo/v2@v2.2.0", "foo/v2@v2.0.1", "foo@v1.1.1"},
			wantPicked:   []string{"foo/v2@v2.2.0", "foo@v1.3.7"},
			wantUnpicked: []string{"foo@v1.0.0", "foo/v2@v2.0.1", "foo@v1.1.1"},
		},
		{
			name:         "semver and pseudo version",
			in:           []string{"foo@v1.0.0", "foo@v1.3.7", "foo/v2@v2.2.0", "foo/v2@v2.0.1", "foo@v1.1.1", "foo@v0.0.0-20190311183353-d8887717615a"},
			wantPicked:   []string{"foo/v2@v2.2.0", "foo@v1.3.7"},
			wantUnpicked: []string{"foo@v1.0.0", "foo/v2@v2.0.1", "foo@v1.1.1", "foo@v0.0.0-20190311183353-d8887717615a"},
		},
		{
			name: "multiple pseudo version",
			in: []string{
				"foo@v0.0.0-20190311183353-d8887717615a",
				"foo@v0.0.0-20190227222117-0694c2d4d067",
				"foo@v0.0.0-20190312151545-0bb0c0a6e846",
			},
			wantPicked: []string{"foo@v0.0.0-20190312151545-0bb0c0a6e846"},
			wantUnpicked: []string{
				"foo@v0.0.0-20190227222117-0694c2d4d067",
				"foo@v0.0.0-20190311183353-d8887717615a",
			},
		},
		{
			name:         "semver and suffix",
			in:           []string{"foo@v1.0.0", "foo@v1.3.8-rc1", "foo@v1.3.7"},
			wantPicked:   []string{"foo@v1.3.8-rc1"},
			wantUnpicked: []string{"foo@v1.0.0", "foo@v1.3.7"},
		},
	} {
		t.Run(tc.name, func(t *testing.T) {
			buf := bytes.Buffer{}
			for _, node := range tc.in {
				fmt.Fprintf(&buf, "A %s\n", node)
			}

			g, err := convert(&buf)
			if err != nil {
				t.Fatal(err)
			}

			if !reflect.DeepEqual(g.mvsPicked, tc.wantPicked) {
				t.Fatalf("picked: got %v, want %v", g.mvsPicked, tc.wantPicked)
			}
			if !reflect.DeepEqual(g.mvsUnpicked, tc.wantUnpicked) {
				t.Fatalf("unpicked: got %v, want %v", g.mvsUnpicked, tc.wantUnpicked)
			}
		})
	}
}
