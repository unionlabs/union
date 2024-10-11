// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package race_test

import (
	"sync"
	"testing"

	"google.golang.org/protobuf/proto"

	epb "google.golang.org/protobuf/internal/testprotos/race/extender"
	mpb "google.golang.org/protobuf/internal/testprotos/race/message"
)

// There must be no other test in this package as we are testing global
// initialization which only happens once per binary.
// It tests that it is safe to initialize the descriptor of a message and
// the descriptor of an extendee of that message in parallel (i.e. no data race).
func TestConcurrentInitialization(t *testing.T) {
	var wg sync.WaitGroup
	wg.Add(2)
	go func() {
		defer wg.Done()
		m := &mpb.MyMessage{
			I32: proto.Int32(int32(42)),
		}
		// This initializes the descriptor.
		_, err := proto.Marshal(m)
		if err != nil {
			t.Errorf("proto.Marshal(): %v", err)
		}
	}()
	go func() {
		defer wg.Done()
		m := &epb.OtherMessage{
			I32: proto.Int32(int32(42)),
		}
		// This initializes the descriptor including the extension.
		_, err := proto.Marshal(m)
		if err != nil {
			t.Errorf("proto.Marshal(): %v", err)
		}
	}()
	wg.Wait()
}
