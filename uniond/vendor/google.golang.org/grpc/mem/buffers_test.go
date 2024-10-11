/*
 *
 * Copyright 2024 gRPC authors.
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

package mem_test

import (
	"bytes"
	"testing"

	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/mem"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	internal.SetBufferPoolingThresholdForTesting.(func(int))(0)

	grpctest.RunSubTests(t, s{})
}

// Tests that a buffer created with NewBuffer, which when later freed, invokes
// the free function with the correct data.
func (s) TestBuffer_NewBufferAndFree(t *testing.T) {
	data := "abcd"
	freed := false
	freeF := poolFunc(func(got *[]byte) {
		if !bytes.Equal(*got, []byte(data)) {
			t.Fatalf("Free function called with bytes %s, want %s", string(*got), data)
		}
		freed = true
	})

	buf := newBuffer([]byte(data), freeF)
	if got := buf.ReadOnlyData(); !bytes.Equal(got, []byte(data)) {
		t.Fatalf("Buffer contains data %s, want %s", string(got), string(data))
	}

	// Verify that the free function is invoked when all references are freed.
	buf.Free()
	if !freed {
		t.Fatalf("Buffer not freed")
	}
}

// Tests that a buffer created with NewBuffer, on which an additional reference
// is acquired, which when later freed, invokes the free function with the
// correct data, but only after all references are released.
func (s) TestBuffer_NewBufferRefAndFree(t *testing.T) {
	data := "abcd"
	freed := false
	freeF := poolFunc(func(got *[]byte) {
		if !bytes.Equal(*got, []byte(data)) {
			t.Fatalf("Free function called with bytes %s, want %s", string(*got), string(data))
		}
		freed = true
	})

	buf := newBuffer([]byte(data), freeF)
	if got := buf.ReadOnlyData(); !bytes.Equal(got, []byte(data)) {
		t.Fatalf("Buffer contains data %s, want %s", string(got), string(data))
	}

	buf.Ref()
	if got := buf.ReadOnlyData(); !bytes.Equal(got, []byte(data)) {
		t.Fatalf("New reference to the Buffer contains data %s, want %s", string(got), string(data))
	}

	// Verify that the free function is not invoked when all references are yet
	// to be freed.
	buf.Free()
	if freed {
		t.Fatalf("Free function called before all references freed")
	}

	// Verify that the free function is invoked when all references are freed.
	buf.Free()
	if !freed {
		t.Fatalf("Buffer not freed")
	}
}

func (s) TestBuffer_FreeAfterFree(t *testing.T) {
	buf := newBuffer([]byte("abcd"), mem.NopBufferPool{})
	if buf.Len() != 4 {
		t.Fatalf("Buffer length is %d, want 4", buf.Len())
	}

	// Ensure that a double free does panic.
	buf.Free()
	defer checkForPanic(t, "Cannot free freed buffer")
	buf.Free()
}

type singleBufferPool struct {
	t    *testing.T
	data *[]byte
}

func (s *singleBufferPool) Get(length int) *[]byte {
	if len(*s.data) != length {
		s.t.Fatalf("Invalid requested length, got %d want %d", length, len(*s.data))
	}
	return s.data
}

func (s *singleBufferPool) Put(b *[]byte) {
	if s.data != b {
		s.t.Fatalf("Wrong buffer returned to pool, got %p want %p", b, s.data)
	}
	s.data = nil
}

// Tests that a buffer created with Copy, which when later freed, returns the underlying
// byte slice to the buffer pool.
func (s) TestBuffer_CopyAndFree(t *testing.T) {
	data := []byte("abcd")
	testPool := &singleBufferPool{
		t:    t,
		data: &data,
	}

	buf := mem.Copy(data, testPool)
	if got := buf.ReadOnlyData(); !bytes.Equal(got, data) {
		t.Fatalf("Buffer contains data %s, want %s", string(got), string(data))
	}

	// Verify that the free function is invoked when all references are freed.
	buf.Free()
	if testPool.data != nil {
		t.Fatalf("Buffer not freed")
	}
}

// Tests that a buffer created with Copy, on which an additional reference is
// acquired, which when later freed, returns the underlying byte slice to the
// buffer pool.
func (s) TestBuffer_CopyRefAndFree(t *testing.T) {
	data := []byte("abcd")
	testPool := &singleBufferPool{
		t:    t,
		data: &data,
	}

	buf := mem.Copy(data, testPool)
	if got := buf.ReadOnlyData(); !bytes.Equal(got, data) {
		t.Fatalf("Buffer contains data %s, want %s", string(got), string(data))
	}

	buf.Ref()
	if got := buf.ReadOnlyData(); !bytes.Equal(got, []byte(data)) {
		t.Fatalf("New reference to the Buffer contains data %s, want %s", string(got), string(data))
	}

	// Verify that the free function is not invoked when all references are yet
	// to be freed.
	buf.Free()
	if testPool.data == nil {
		t.Fatalf("Free function called before all references freed")
	}

	// Verify that the free function is invoked when all references are freed.
	buf.Free()
	if testPool.data != nil {
		t.Fatalf("Free never called")
	}
}

func (s) TestBuffer_ReadOnlyDataAfterFree(t *testing.T) {
	// Verify that reading before freeing does not panic.
	buf := newBuffer([]byte("abcd"), mem.NopBufferPool{})
	buf.ReadOnlyData()

	buf.Free()
	defer checkForPanic(t, "Cannot read freed buffer")
	buf.ReadOnlyData()
}

func (s) TestBuffer_RefAfterFree(t *testing.T) {
	// Verify that acquiring a ref before freeing does not panic.
	buf := newBuffer([]byte("abcd"), mem.NopBufferPool{})
	buf.Ref()

	// This first call should not panc and bring the ref counter down to 1
	buf.Free()
	// This second call actually frees the buffer
	buf.Free()
	defer checkForPanic(t, "Cannot ref freed buffer")
	buf.Ref()
}

func (s) TestBuffer_SplitAfterFree(t *testing.T) {
	// Verify that splitting before freeing does not panic.
	buf := newBuffer([]byte("abcd"), mem.NopBufferPool{})
	buf, bufSplit := mem.SplitUnsafe(buf, 2)

	bufSplit.Free()
	buf.Free()
	defer checkForPanic(t, "Cannot split freed buffer")
	mem.SplitUnsafe(buf, 2)
}

type poolFunc func(*[]byte)

func (p poolFunc) Get(length int) *[]byte {
	panic("Get should never be called")
}

func (p poolFunc) Put(i *[]byte) {
	p(i)
}

func (s) TestBuffer_Split(t *testing.T) {
	ready := false
	freed := false
	data := []byte{1, 2, 3, 4}
	buf := mem.NewBuffer(&data, poolFunc(func(bytes *[]byte) {
		if !ready {
			t.Fatalf("Freed too early")
		}
		freed = true
	}))
	checkBufData := func(b mem.Buffer, expected []byte) {
		t.Helper()
		if !bytes.Equal(b.ReadOnlyData(), expected) {
			t.Fatalf("Buffer did not contain expected data %v, got %v", expected, b.ReadOnlyData())
		}
	}

	buf, split1 := mem.SplitUnsafe(buf, 2)
	checkBufData(buf, data[:2])
	checkBufData(split1, data[2:])

	// Check that splitting the buffer more than once works as intended.
	split1, split2 := mem.SplitUnsafe(split1, 1)
	checkBufData(split1, data[2:3])
	checkBufData(split2, data[3:])

	// If any of the following frees actually free the buffer, the test will fail.
	buf.Free()
	split2.Free()

	ready = true
	split1.Free()

	if !freed {
		t.Fatalf("Buffer never freed")
	}
}

func checkForPanic(t *testing.T, wantErr string) {
	t.Helper()
	r := recover()
	if r == nil {
		t.Fatalf("Use after free did not panic")
	}
	if msg, ok := r.(string); !ok || msg != wantErr {
		t.Fatalf("panic called with %v, want %s", r, wantErr)
	}
}
