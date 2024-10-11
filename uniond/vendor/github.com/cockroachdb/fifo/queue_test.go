// Copyright 2024 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package fifo

import (
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"
)

var pool = MakeQueueBackingPool[int]()

func TestQueue(t *testing.T) {
	q := MakeQueue[int](&pool)
	require.Nil(t, q.PeekFront())
	require.Equal(t, 0, q.Len())
	q.PushBack(1)
	q.PushBack(2)
	q.PushBack(3)
	require.Equal(t, 3, q.Len())
	require.Equal(t, 1, *q.PeekFront())
	q.PopFront()
	require.Equal(t, 2, *q.PeekFront())
	q.PopFront()
	require.Equal(t, 3, *q.PeekFront())
	q.PopFront()
	require.Nil(t, q.PeekFront())

	for i := 1; i <= 1000; i++ {
		q.PushBack(i)
		require.Equal(t, i, q.Len())
	}
	for i := 1; i <= 1000; i++ {
		require.Equal(t, i, *q.PeekFront())
		q.PopFront()
		require.Equal(t, 1000-i, q.Len())
	}
}

func TestQueueRand(t *testing.T) {
	q := MakeQueue[int](&pool)
	l, r := 0, 0
	for iteration := 0; iteration < 100; iteration++ {
		for n := rand.Intn(100); n > 0; n-- {
			r++
			q.PushBack(r)
			require.Equal(t, r-l, q.Len())
		}
		for n := rand.Intn(q.Len() + 1); n > 0; n-- {
			l++
			require.Equal(t, l, *q.PeekFront())
			q.PopFront()
			require.Equal(t, r-l, q.Len())
		}
	}
}
