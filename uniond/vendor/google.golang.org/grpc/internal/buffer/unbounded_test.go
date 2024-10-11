/*
 * Copyright 2019 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package buffer

import (
	"reflect"
	"sort"
	"sync"
	"testing"

	"google.golang.org/grpc/internal/grpctest"
)

const (
	numWriters = 10
	numWrites  = 10
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

// wantReads contains the set of values expected to be read by the reader
// goroutine in the tests.
var wantReads []int

func init() {
	for i := 0; i < numWriters; i++ {
		for j := 0; j < numWrites; j++ {
			wantReads = append(wantReads, i)
		}
	}
}

// TestSingleWriter starts one reader and one writer goroutine and makes sure
// that the reader gets all the values added to the buffer by the writer.
func (s) TestSingleWriter(t *testing.T) {
	ub := NewUnbounded()
	reads := []int{}

	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		ch := ub.Get()
		for i := 0; i < numWriters*numWrites; i++ {
			r := <-ch
			reads = append(reads, r.(int))
			ub.Load()
		}
	}()

	wg.Add(1)
	go func() {
		defer wg.Done()
		for i := 0; i < numWriters; i++ {
			for j := 0; j < numWrites; j++ {
				ub.Put(i)
			}
		}
	}()

	wg.Wait()
	if !reflect.DeepEqual(reads, wantReads) {
		t.Errorf("reads: %#v, wantReads: %#v", reads, wantReads)
	}
}

// TestMultipleWriters starts multiple writers and one reader goroutine and
// makes sure that the reader gets all the data written by all writers.
func (s) TestMultipleWriters(t *testing.T) {
	ub := NewUnbounded()
	reads := []int{}

	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		ch := ub.Get()
		for i := 0; i < numWriters*numWrites; i++ {
			r := <-ch
			reads = append(reads, r.(int))
			ub.Load()
		}
	}()

	wg.Add(numWriters)
	for i := 0; i < numWriters; i++ {
		go func(index int) {
			defer wg.Done()
			for j := 0; j < numWrites; j++ {
				ub.Put(index)
			}
		}(i)
	}

	wg.Wait()
	sort.Ints(reads)
	if !reflect.DeepEqual(reads, wantReads) {
		t.Errorf("reads: %#v, wantReads: %#v", reads, wantReads)
	}
}

// TestClose closes the buffer and makes sure that nothing is sent after the
// buffer is closed.
func (s) TestClose(t *testing.T) {
	ub := NewUnbounded()
	if err := ub.Put(1); err != nil {
		t.Fatalf("Unbounded.Put() = %v; want nil", err)
	}
	ub.Close()
	if err := ub.Put(1); err == nil {
		t.Fatalf("Unbounded.Put() = <nil>; want non-nil error")
	}
	if v, ok := <-ub.Get(); !ok {
		t.Errorf("Unbounded.Get() = %v, %v, want %v, %v", v, ok, 1, true)
	}
	if err := ub.Put(1); err == nil {
		t.Fatalf("Unbounded.Put() = <nil>; want non-nil error")
	}
	ub.Load()
	if v, ok := <-ub.Get(); ok {
		t.Errorf("Unbounded.Get() = %v, want closed channel", v)
	}
	if err := ub.Put(1); err == nil {
		t.Fatalf("Unbounded.Put() = <nil>; want non-nil error")
	}
	ub.Close() // ignored
}
