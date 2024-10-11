// Copyright 2019 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//go:build !windows
// +build !windows

package procfs

import (
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestZoneinfo(t *testing.T) {
	fs := getProcFixtures(t)

	protectionDMA := []*int64{newPInt64(0), newPInt64(2877), newPInt64(7826), newPInt64(7826), newPInt64(7826)}
	protectionDMA32 := []*int64{newPInt64(0), newPInt64(0), newPInt64(4949), newPInt64(4949), newPInt64(4949)}
	protectionNormal := []*int64{newPInt64(0), newPInt64(0), newPInt64(0), newPInt64(0), newPInt64(0)}
	protectionMovable := []*int64{newPInt64(0), newPInt64(0), newPInt64(0), newPInt64(0), newPInt64(0)}
	protectionDevice := []*int64{newPInt64(0), newPInt64(0), newPInt64(0), newPInt64(0), newPInt64(0)}

	refs := []Zoneinfo{
		{Node: "0", Zone: "DMA", NrFreePages: newPInt64(3952), Min: newPInt64(33), Low: newPInt64(41), High: newPInt64(49), Spanned: newPInt64(4095), Present: newPInt64(3975), Managed: newPInt64(3956), NrActiveAnon: newPInt64(547580), NrInactiveAnon: newPInt64(230981), NrIsolatedAnon: newPInt64(0), NrAnonPages: newPInt64(795576), NrAnonTransparentHugepages: newPInt64(0), NrActiveFile: newPInt64(346282), NrInactiveFile: newPInt64(316904), NrIsolatedFile: newPInt64(0), NrFilePages: newPInt64(761874), NrSlabReclaimable: newPInt64(131220), NrSlabUnreclaimable: newPInt64(47320), NrKernelStack: newPInt64(0), NrMapped: newPInt64(215483), NrDirty: newPInt64(908), NrWriteback: newPInt64(0), NrUnevictable: newPInt64(115467), NrShmem: newPInt64(224925), NrDirtied: newPInt64(8007423), NrWritten: newPInt64(7752121), NumaHit: newPInt64(1), NumaMiss: newPInt64(0), NumaForeign: newPInt64(0), NumaInterleave: newPInt64(0), NumaLocal: newPInt64(1), NumaOther: newPInt64(0), Protection: protectionDMA},
		{Node: "0", Zone: "DMA32", NrFreePages: newPInt64(204252), Min: newPInt64(19510), Low: newPInt64(21059), High: newPInt64(22608), Spanned: newPInt64(1044480), Present: newPInt64(759231), Managed: newPInt64(742806), NrKernelStack: newPInt64(2208), NumaHit: newPInt64(113952967), NumaMiss: newPInt64(0), NumaForeign: newPInt64(0), NumaInterleave: newPInt64(0), NumaLocal: newPInt64(113952967), NumaOther: newPInt64(0), Protection: protectionDMA32},
		{Node: "0", Zone: "Normal", NrFreePages: newPInt64(18553), Min: newPInt64(11176), Low: newPInt64(13842), High: newPInt64(16508), Spanned: newPInt64(1308160), Present: newPInt64(1308160), Managed: newPInt64(1268711), NrKernelStack: newPInt64(15136), NumaHit: newPInt64(162718019), NumaMiss: newPInt64(0), NumaForeign: newPInt64(0), NumaInterleave: newPInt64(26812), NumaLocal: newPInt64(162718019), NumaOther: newPInt64(0), Protection: protectionNormal},
		{Node: "0", Zone: "Movable", Min: newPInt64(0), Low: newPInt64(0), High: newPInt64(0), Spanned: newPInt64(0), Present: newPInt64(0), Managed: newPInt64(0), Protection: protectionMovable},
		{Node: "0", Zone: "Device", Min: newPInt64(0), Low: newPInt64(0), High: newPInt64(0), Spanned: newPInt64(0), Present: newPInt64(0), Managed: newPInt64(0), Protection: protectionDevice},
	}

	data, err := fs.Zoneinfo()
	if err != nil {
		t.Fatalf("failed to parse zoneinfo: %v", err)
	}
	if len(data) != 5 {
		t.Fatal("failed to get all all node info")
	}

	for index, ref := range refs {
		want, got := ref, data[index]
		if diff := cmp.Diff(want, got); diff != "" {
			t.Fatalf("unexpected crypto entry (-want +got):\n%s", diff)
		}

	}
}
