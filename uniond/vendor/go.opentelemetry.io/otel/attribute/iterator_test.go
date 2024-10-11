// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package attribute_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/otel/attribute"
)

func TestIterator(t *testing.T) {
	one := attribute.String("one", "1")
	two := attribute.Int("two", 2)
	lbl := attribute.NewSet(one, two)
	iter := lbl.Iter()
	require.Equal(t, 2, iter.Len())

	require.True(t, iter.Next())
	require.Equal(t, one, iter.Attribute())
	idx, attr := iter.IndexedAttribute()
	require.Equal(t, 0, idx)
	require.Equal(t, one, attr)
	require.Equal(t, 2, iter.Len())

	require.True(t, iter.Next())
	require.Equal(t, two, iter.Attribute())
	idx, attr = iter.IndexedAttribute()
	require.Equal(t, 1, idx)
	require.Equal(t, two, attr)
	require.Equal(t, 2, iter.Len())

	require.False(t, iter.Next())
	require.Equal(t, 2, iter.Len())
}

func TestEmptyIterator(t *testing.T) {
	lbl := attribute.NewSet()
	iter := lbl.Iter()
	require.Equal(t, 0, iter.Len())
	require.False(t, iter.Next())
}

func TestMergedIterator(t *testing.T) {
	type inputs struct {
		name   string
		keys1  []string
		keys2  []string
		expect []string
	}

	makeAttributes := func(keys []string, num int) (result []attribute.KeyValue) {
		for _, k := range keys {
			result = append(result, attribute.Int(k, num))
		}
		return
	}

	for _, input := range []inputs{
		{
			name:   "one overlap",
			keys1:  []string{"A", "B"},
			keys2:  []string{"B", "C"},
			expect: []string{"A/1", "B/1", "C/2"},
		},
		{
			name:   "reversed one overlap",
			keys1:  []string{"B", "A"},
			keys2:  []string{"C", "B"},
			expect: []string{"A/1", "B/1", "C/2"},
		},
		{
			name:   "one empty",
			keys1:  nil,
			keys2:  []string{"C", "B"},
			expect: []string{"B/2", "C/2"},
		},
		{
			name:   "two empty",
			keys1:  []string{"C", "B"},
			keys2:  nil,
			expect: []string{"B/1", "C/1"},
		},
		{
			name:   "no overlap both",
			keys1:  []string{"C"},
			keys2:  []string{"B"},
			expect: []string{"B/2", "C/1"},
		},
		{
			name:   "one empty single two",
			keys1:  nil,
			keys2:  []string{"B"},
			expect: []string{"B/2"},
		},
		{
			name:   "two empty single one",
			keys1:  []string{"A"},
			keys2:  nil,
			expect: []string{"A/1"},
		},
		{
			name:   "all empty",
			keys1:  nil,
			keys2:  nil,
			expect: nil,
		},
		{
			name:   "full overlap",
			keys1:  []string{"A", "B", "C", "D"},
			keys2:  []string{"A", "B", "C", "D"},
			expect: []string{"A/1", "B/1", "C/1", "D/1"},
		},
	} {
		t.Run(input.name, func(t *testing.T) {
			attr1 := makeAttributes(input.keys1, 1)
			attr2 := makeAttributes(input.keys2, 2)

			set1 := attribute.NewSet(attr1...)
			set2 := attribute.NewSet(attr2...)

			merge := attribute.NewMergeIterator(&set1, &set2)

			var result []string

			for merge.Next() {
				attr := merge.Attribute()
				result = append(result, fmt.Sprint(attr.Key, "/", attr.Value.Emit()))
			}

			require.Equal(t, input.expect, result)
		})
	}
}
