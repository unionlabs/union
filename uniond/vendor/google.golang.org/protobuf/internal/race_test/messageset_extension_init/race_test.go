// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package race_test

import (
	"sync"
	"testing"

	epb "google.golang.org/protobuf/internal/testprotos/messageset/msetextpb"
)

// There must be no other test in this package as we are testing global
// initialization which only happens once per binary.
func TestConcurrentInitialization(t *testing.T) {
	var wg sync.WaitGroup
	wg.Add(2)
	go func() {
		defer wg.Done()
		epb.E_Ext1_MessageSetExtension.ValueOf(&epb.Ext1{})
	}()
	go func() {
		defer wg.Done()
		epb.E_Ext1_MessageSetExtension.TypeDescriptor().Message()
	}()
	wg.Wait()
}
