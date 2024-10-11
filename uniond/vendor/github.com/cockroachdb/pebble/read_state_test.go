// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"fmt"
	"testing"
	"time"

	"github.com/cockroachdb/pebble/vfs"
	"golang.org/x/exp/rand"
)

func BenchmarkReadState(b *testing.B) {
	d, err := Open("", &Options{
		FS: vfs.NewMem(),
	})
	if err != nil {
		b.Fatal(err)
	}

	for _, updateFrac := range []float32{0, 0.1, 0.5} {
		b.Run(fmt.Sprintf("updates=%.0f", updateFrac*100), func(b *testing.B) {
			b.RunParallel(func(pb *testing.PB) {
				rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))

				for pb.Next() {
					if rng.Float32() < updateFrac {
						d.mu.Lock()
						d.updateReadStateLocked(nil)
						d.mu.Unlock()
					} else {
						s := d.loadReadState()
						s.unref()
					}
				}
			})
		})
	}

	if err := d.Close(); err != nil {
		b.Fatal(err)
	}
}
