// Copyright (c) 2023 Uber Technologies, Inc.
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

//go:build go1.20
// +build go1.20

package multierr

import (
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestErrorsOnErrorsJoin(t *testing.T) {
	err1 := errors.New("err1")
	err2 := errors.New("err2")
	err := errors.Join(err1, err2)

	errs := Errors(err)
	assert.Equal(t, 2, len(errs))
	assert.Equal(t, err1, errs[0])
	assert.Equal(t, err2, errs[1])
}

func TestEveryWithErrorsJoin(t *testing.T) {
	myError1 := errors.New("woeful misfortune")
	myError2 := errors.New("worrisome travesty")

	t.Run("all match", func(t *testing.T) {
		err := errors.Join(myError1, myError1, myError1)

		assert.True(t, errors.Is(err, myError1))
		assert.True(t, Every(err, myError1))
		assert.False(t, errors.Is(err, myError2))
		assert.False(t, Every(err, myError2))
	})

	t.Run("one matches", func(t *testing.T) {
		err := errors.Join(myError1, myError2)

		assert.True(t, errors.Is(err, myError1))
		assert.False(t, Every(err, myError1))
		assert.True(t, errors.Is(err, myError2))
		assert.False(t, Every(err, myError2))
	})
}
