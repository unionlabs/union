// Copyright (c) 2017 Uber Technologies, Inc.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

package multierr

import (
	"errors"
	"fmt"
	"testing"
)

func BenchmarkAppend(b *testing.B) {
	errorTypes := []struct {
		name string
		err  error
	}{
		{
			name: "nil",
			err:  nil,
		},
		{
			name: "single error",
			err:  errors.New("test"),
		},
		{
			name: "multiple errors",
			err:  appendN(nil, errors.New("err"), 10),
		},
	}

	for _, initial := range errorTypes {
		for _, v := range errorTypes {
			msg := fmt.Sprintf("append %v to %v", v.name, initial.name)
			b.Run(msg, func(b *testing.B) {
				for _, appends := range []int{1, 2, 10} {
					b.Run(fmt.Sprint(appends), func(b *testing.B) {
						for i := 0; i < b.N; i++ {
							appendN(initial.err, v.err, appends)
						}
					})
				}
			})
		}
	}
}

func BenchmarkCombine(b *testing.B) {
	b.Run("inline 1", func(b *testing.B) {
		var x error
		for i := 0; i < b.N; i++ {
			Combine(x)
		}
	})

	b.Run("inline 2", func(b *testing.B) {
		var x, y error
		for i := 0; i < b.N; i++ {
			Combine(x, y)
		}
	})

	b.Run("inline 3 no error", func(b *testing.B) {
		var x, y, z error
		for i := 0; i < b.N; i++ {
			Combine(x, y, z)
		}
	})

	b.Run("inline 3 one error", func(b *testing.B) {
		var x, y, z error
		z = fmt.Errorf("failed")
		for i := 0; i < b.N; i++ {
			Combine(x, y, z)
		}
	})

	b.Run("inline 3 multiple errors", func(b *testing.B) {
		var x, y, z error
		z = fmt.Errorf("failed3")
		y = fmt.Errorf("failed2")
		x = fmt.Errorf("failed")
		for i := 0; i < b.N; i++ {
			Combine(x, y, z)
		}
	})

	b.Run("slice 100 no errors", func(b *testing.B) {
		errs := make([]error, 100)
		for i := 0; i < b.N; i++ {
			Combine(errs...)
		}
	})

	b.Run("slice 100 one error", func(b *testing.B) {
		errs := make([]error, 100)
		errs[len(errs)-1] = fmt.Errorf("failed")
		for i := 0; i < b.N; i++ {
			Combine(errs...)
		}
	})

	b.Run("slice 100 multi error", func(b *testing.B) {
		errs := make([]error, 100)
		errs[0] = fmt.Errorf("failed1")
		errs[len(errs)-1] = fmt.Errorf("failed2")
		for i := 0; i < b.N; i++ {
			Combine(errs...)
		}
	})
}
