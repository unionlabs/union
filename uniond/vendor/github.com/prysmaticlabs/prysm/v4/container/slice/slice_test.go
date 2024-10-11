package slice_test

import (
	"reflect"
	"sort"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/container/slice"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestSubsetUint64(t *testing.T) {
	testCases := []struct {
		setA []uint64
		setB []uint64
		out  bool
	}{
		{[]uint64{1}, []uint64{1, 2, 3, 4}, true},
		{[]uint64{1, 2, 3, 4}, []uint64{1, 2, 3, 4}, true},
		{[]uint64{1, 1}, []uint64{1, 2, 3, 4}, false},
		{[]uint64{}, []uint64{1}, true},
		{[]uint64{1}, []uint64{}, false},
		{[]uint64{1, 2, 3, 4, 5}, []uint64{1, 2, 3, 4}, false},
	}
	for _, tt := range testCases {
		result := slice.SubsetUint64(tt.setA, tt.setB)
		if result != tt.out {
			t.Errorf("%v, got %v, want %v", tt.setA, result, tt.out)
		}
	}
}

func TestIntersectionUint64(t *testing.T) {
	testCases := []struct {
		setA []uint64
		setB []uint64
		setC []uint64
		out  []uint64
	}{
		{[]uint64{2, 3, 5}, []uint64{3}, []uint64{3}, []uint64{3}},
		{[]uint64{2, 3, 5}, []uint64{3, 5}, []uint64{5}, []uint64{5}},
		{[]uint64{2, 3, 5}, []uint64{3, 5}, []uint64{3, 5}, []uint64{3, 5}},
		{[]uint64{2, 3, 5}, []uint64{5, 3, 2}, []uint64{3, 2, 5}, []uint64{2, 3, 5}},
		{[]uint64{3, 2, 5}, []uint64{5, 3, 2}, []uint64{3, 2, 5}, []uint64{2, 3, 5}},
		{[]uint64{3, 3, 5}, []uint64{5, 3, 2}, []uint64{3, 2, 5}, []uint64{3, 5}},
		{[]uint64{2, 3, 5}, []uint64{2, 3, 5}, []uint64{2, 3, 5}, []uint64{2, 3, 5}},
		{[]uint64{2, 3, 5}, []uint64{}, []uint64{}, []uint64{}},
		{[]uint64{2, 3, 5}, []uint64{2, 3, 5}, []uint64{}, []uint64{}},
		{[]uint64{2, 3}, []uint64{2, 3, 5}, []uint64{5}, []uint64{}},
		{[]uint64{2, 2, 2}, []uint64{2, 2, 2}, []uint64{}, []uint64{}},
		{[]uint64{}, []uint64{2, 3, 5}, []uint64{}, []uint64{}},
		{[]uint64{}, []uint64{}, []uint64{}, []uint64{}},
		{[]uint64{1}, []uint64{1}, []uint64{}, []uint64{}},
		{[]uint64{1, 1, 1}, []uint64{1, 1}, []uint64{1, 2, 3}, []uint64{1}},
	}
	for _, tt := range testCases {
		setA := append([]uint64{}, tt.setA...)
		setB := append([]uint64{}, tt.setB...)
		setC := append([]uint64{}, tt.setC...)
		result := slice.IntersectionUint64(setA, setB, setC)
		sort.Slice(result, func(i, j int) bool {
			return result[i] < result[j]
		})
		if !reflect.DeepEqual(result, tt.out) {
			t.Errorf("got %d, want %d", result, tt.out)
		}
		if !reflect.DeepEqual(setA, tt.setA) {
			t.Errorf("slice modified, got %v, want %v", setA, tt.setA)
		}
		if !reflect.DeepEqual(setB, tt.setB) {
			t.Errorf("slice modified, got %v, want %v", setB, tt.setB)
		}
		if !reflect.DeepEqual(setC, tt.setC) {
			t.Errorf("slice modified, got %v, want %v", setC, tt.setC)
		}
	}
}

func TestIsSortedUint64(t *testing.T) {
	testCases := []struct {
		setA []uint64
		out  bool
	}{
		{[]uint64{1, 2, 3}, true},
		{[]uint64{3, 1, 3}, false},
		{[]uint64{1}, true},
		{[]uint64{}, true},
	}
	for _, tt := range testCases {
		result := slice.IsUint64Sorted(tt.setA)
		if result != tt.out {
			t.Errorf("got %v, want %v", result, tt.out)
		}
	}
}

func TestIntersectionInt64(t *testing.T) {
	testCases := []struct {
		setA []int64
		setB []int64
		setC []int64
		out  []int64
	}{
		{[]int64{2, 3, 5}, []int64{3}, []int64{3}, []int64{3}},
		{[]int64{2, 3, 5}, []int64{3, 5}, []int64{5}, []int64{5}},
		{[]int64{2, 3, 5}, []int64{3, 5}, []int64{3, 5}, []int64{3, 5}},
		{[]int64{2, 3, 5}, []int64{5, 3, 2}, []int64{3, 2, 5}, []int64{2, 3, 5}},
		{[]int64{3, 2, 5}, []int64{5, 3, 2}, []int64{3, 2, 5}, []int64{2, 3, 5}},
		{[]int64{3, 3, 5}, []int64{5, 3, 2}, []int64{3, 2, 5}, []int64{3, 5}},
		{[]int64{2, 3, 5}, []int64{2, 3, 5}, []int64{2, 3, 5}, []int64{2, 3, 5}},
		{[]int64{2, 3, 5}, []int64{}, []int64{}, []int64{}},
		{[]int64{2, 3, 5}, []int64{2, 3, 5}, []int64{}, []int64{}},
		{[]int64{2, 3}, []int64{2, 3, 5}, []int64{5}, []int64{}},
		{[]int64{2, 2, 2}, []int64{2, 2, 2}, []int64{}, []int64{}},
		{[]int64{}, []int64{2, 3, 5}, []int64{}, []int64{}},
		{[]int64{}, []int64{}, []int64{}, []int64{}},
		{[]int64{1}, []int64{1}, []int64{}, []int64{}},
		{[]int64{1, 1, 1}, []int64{1, 1}, []int64{1, 2, 3}, []int64{1}},
	}
	for _, tt := range testCases {
		setA := append([]int64{}, tt.setA...)
		setB := append([]int64{}, tt.setB...)
		setC := append([]int64{}, tt.setC...)
		result := slice.IntersectionInt64(setA, setB, setC)
		sort.Slice(result, func(i, j int) bool {
			return result[i] < result[j]
		})
		if !reflect.DeepEqual(result, tt.out) {
			t.Errorf("got %d, want %d", result, tt.out)
		}
		if !reflect.DeepEqual(setA, tt.setA) {
			t.Errorf("slice modified, got %v, want %v", setA, tt.setA)
		}
		if !reflect.DeepEqual(setB, tt.setB) {
			t.Errorf("slice modified, got %v, want %v", setB, tt.setB)
		}
		if !reflect.DeepEqual(setC, tt.setC) {
			t.Errorf("slice modified, got %v, want %v", setC, tt.setC)
		}
	}
}

func TestUnionUint64(t *testing.T) {
	testCases := []struct {
		setA []uint64
		setB []uint64
		out  []uint64
	}{
		{[]uint64{2, 3, 5}, []uint64{4, 6}, []uint64{2, 3, 5, 4, 6}},
		{[]uint64{2, 3, 5}, []uint64{3, 5}, []uint64{2, 3, 5}},
		{[]uint64{2, 3, 5}, []uint64{2, 3, 5}, []uint64{2, 3, 5}},
		{[]uint64{2, 3, 5}, []uint64{}, []uint64{2, 3, 5}},
		{[]uint64{}, []uint64{2, 3, 5}, []uint64{2, 3, 5}},
		{[]uint64{}, []uint64{}, []uint64{}},
		{[]uint64{1}, []uint64{1}, []uint64{1}},
	}
	for _, tt := range testCases {
		result := slice.UnionUint64(tt.setA, tt.setB)
		if !reflect.DeepEqual(result, tt.out) {
			t.Errorf("got %d, want %d", result, tt.out)
		}

	}
	items := [][]uint64{
		{3, 4, 5},
		{6, 7, 8},
		{9, 10, 11},
	}
	variadicResult := slice.UnionUint64(items...)
	want := []uint64{3, 4, 5, 6, 7, 8, 9, 10, 11}
	if !reflect.DeepEqual(want, variadicResult) {
		t.Errorf("Received %v, wanted %v", variadicResult, want)
	}
}

func TestUnionInt64(t *testing.T) {
	testCases := []struct {
		setA []int64
		setB []int64
		out  []int64
	}{
		{[]int64{2, 3, 5}, []int64{4, 6}, []int64{2, 3, 5, 4, 6}},
		{[]int64{2, 3, 5}, []int64{3, 5}, []int64{2, 3, 5}},
		{[]int64{2, 3, 5}, []int64{2, 3, 5}, []int64{2, 3, 5}},
		{[]int64{2, 3, 5}, []int64{}, []int64{2, 3, 5}},
		{[]int64{}, []int64{2, 3, 5}, []int64{2, 3, 5}},
		{[]int64{}, []int64{}, []int64{}},
		{[]int64{1}, []int64{1}, []int64{1}},
	}
	for _, tt := range testCases {
		result := slice.UnionInt64(tt.setA, tt.setB)
		if !reflect.DeepEqual(result, tt.out) {
			t.Errorf("got %d, want %d", result, tt.out)
		}
	}
	items := [][]int64{
		{3, 4, 5},
		{6, 7, 8},
		{9, 10, 11},
	}
	variadicResult := slice.UnionInt64(items...)
	want := []int64{3, 4, 5, 6, 7, 8, 9, 10, 11}
	if !reflect.DeepEqual(want, variadicResult) {
		t.Errorf("Received %v, wanted %v", variadicResult, want)
	}
}

func TestCleanUint64(t *testing.T) {
	testCases := []struct {
		in  []uint64
		out []uint64
	}{
		{[]uint64{2, 4, 4, 6, 6}, []uint64{2, 4, 6}},
		{[]uint64{3, 5, 5}, []uint64{3, 5}},
		{[]uint64{2, 2, 2}, []uint64{2}},
		{[]uint64{1, 4, 5, 9, 9}, []uint64{1, 4, 5, 9}},
		{[]uint64{}, []uint64{}},
		{[]uint64{1}, []uint64{1}},
	}
	for _, tt := range testCases {
		result := slice.SetUint64(tt.in)
		if !reflect.DeepEqual(result, tt.out) {
			t.Errorf("got %d, want %d", result, tt.out)
		}
	}
}

func TestNotUint64(t *testing.T) {
	testCases := []struct {
		setA []uint64
		setB []uint64
		out  []uint64
	}{
		{[]uint64{4, 6}, []uint64{2, 3, 5, 4, 6}, []uint64{2, 3, 5}},
		{[]uint64{3, 5}, []uint64{2, 3, 5}, []uint64{2}},
		{[]uint64{2, 3, 5}, []uint64{2, 3, 5}, []uint64{}},
		{[]uint64{2}, []uint64{2, 3, 5}, []uint64{3, 5}},
		{[]uint64{}, []uint64{2, 3, 5}, []uint64{2, 3, 5}},
		{[]uint64{}, []uint64{}, []uint64{}},
		{[]uint64{1}, []uint64{1}, []uint64{}},
	}
	for _, tt := range testCases {
		result := slice.NotUint64(tt.setA, tt.setB)
		if !reflect.DeepEqual(result, tt.out) {
			t.Errorf("got %d, want %d", result, tt.out)
		}
	}
}

func TestNotInt64(t *testing.T) {
	testCases := []struct {
		setA []int64
		setB []int64
		out  []int64
	}{
		{[]int64{4, 6}, []int64{2, 3, 5, 4, 6}, []int64{2, 3, 5}},
		{[]int64{3, 5}, []int64{2, 3, 5}, []int64{2}},
		{[]int64{2, 3, 5}, []int64{2, 3, 5}, []int64{}},
		{[]int64{2}, []int64{2, 3, 5}, []int64{3, 5}},
		{[]int64{}, []int64{2, 3, 5}, []int64{2, 3, 5}},
		{[]int64{}, []int64{}, []int64{}},
		{[]int64{1}, []int64{1}, []int64{}},
	}
	for _, tt := range testCases {
		result := slice.NotInt64(tt.setA, tt.setB)
		if !reflect.DeepEqual(result, tt.out) {
			t.Errorf("got %d, want %d", result, tt.out)
		}
	}
}

func TestIsInUint64(t *testing.T) {
	testCases := []struct {
		a      uint64
		b      []uint64
		result bool
	}{
		{0, []uint64{}, false},
		{0, []uint64{0}, true},
		{4, []uint64{2, 3, 5, 4, 6}, true},
		{100, []uint64{2, 3, 5, 4, 6}, false},
	}
	for _, tt := range testCases {
		result := slice.IsInUint64(tt.a, tt.b)
		if result != tt.result {
			t.Errorf("IsIn(%d, %v)=%v, wanted: %v",
				tt.a, tt.b, result, tt.result)
		}
	}
}

func TestIsInInt64(t *testing.T) {
	testCases := []struct {
		a      int64
		b      []int64
		result bool
	}{
		{0, []int64{}, false},
		{0, []int64{0}, true},
		{4, []int64{2, 3, 5, 4, 6}, true},
		{100, []int64{2, 3, 5, 4, 6}, false},
	}
	for _, tt := range testCases {
		result := slice.IsInInt64(tt.a, tt.b)
		if result != tt.result {
			t.Errorf("IsIn(%d, %v)=%v, wanted: %v",
				tt.a, tt.b, result, tt.result)
		}
	}
}

func TestUnionByteSlices(t *testing.T) {
	testCases := []struct {
		setA [][]byte
		setB [][]byte
		out  [][]byte
	}{
		{
			[][]byte{[]byte("hello"), []byte("world")},
			[][]byte{[]byte("world"), {}},
			[][]byte{[]byte("hello"), []byte("world"), {}},
		},
		{
			[][]byte{[]byte("hello")},
			[][]byte{[]byte("hello")},
			[][]byte{[]byte("hello")},
		},
	}
	for _, tt := range testCases {
		result := slice.UnionByteSlices(tt.setA, tt.setB)
		if !reflect.DeepEqual(result, tt.out) {
			t.Errorf("got %d, want %d", result, tt.out)
		}
	}
	items := [][][]byte{
		{
			{1, 2, 3},
		},
		{
			{4, 5, 6},
		},
		{
			{7, 8, 9},
		},
	}
	variadicResult := slice.UnionByteSlices(items...)
	want := [][]byte{
		{1, 2, 3},
		{4, 5, 6},
		{7, 8, 9},
	}
	if !reflect.DeepEqual(want, variadicResult) {
		t.Errorf("Received %v, wanted %v", variadicResult, want)
	}
}

func TestIntersectionByteSlices(t *testing.T) {
	testCases := []struct {
		name   string
		input  [][][]byte
		result [][]byte
	}{
		{
			name: "intersect with empty set",
			input: [][][]byte{
				{
					{1, 2, 3},
					{4, 5},
				},
				{
					{1, 2},
					{4, 5},
				},
				{},
			},
			result: [][]byte{},
		},
		{
			name: "ensure duplicate elements are removed in the resulting set",
			input: [][][]byte{
				{
					{1, 2, 3},
					{4, 5},
					{4, 5},
				},
				{
					{1, 2},
					{4, 5},
					{4, 5},
				},
				{
					{4, 5},
					{4, 5},
				},
			},
			result: [][]byte{{4, 5}},
		},
		{
			name: "ensure no intersection returns an empty set",
			input: [][][]byte{
				{
					{1, 2, 3},
					{4, 5},
				},
				{
					{1, 2},
				},
				{
					{1, 2},
				},
			},
			result: [][]byte{},
		},
		{
			name: "intersection between A and A should return A",
			input: [][][]byte{
				{
					{1, 2},
				},
				{
					{1, 2},
				},
				{
					{1, 2},
				},
			},
			result: [][]byte{{1, 2}},
		},
	}
	for _, tt := range testCases {
		t.Run(tt.name, func(t *testing.T) {
			result := slice.IntersectionByteSlices(tt.input...)
			if !reflect.DeepEqual(result, tt.result) {
				t.Errorf("IntersectionByteSlices(%v)=%v, wanted: %v",
					tt.input, result, tt.result)
			}
		})
	}
	t.Run("properly handle duplicates", func(t *testing.T) {
		input := [][][]byte{
			{{1, 2}, {1, 2}},
			{{1, 2}, {1, 2}},
			{},
		}
		result := slice.IntersectionByteSlices(input...)
		if !reflect.DeepEqual(result, [][]byte{}) {
			t.Errorf("IntersectionByteSlices(%v)=%v, wanted: %v", input, result, [][]byte{})
		}
	})
}

func TestSplitCommaSeparated(t *testing.T) {
	tests := []struct {
		input  []string
		output []string
	}{
		{
			input:  []string{"a,b", "c,d"},
			output: []string{"a", "b", "c", "d"},
		},
		{
			input:  []string{"a", "b,c,d"},
			output: []string{"a", "b", "c", "d"},
		},
		{
			input:  []string{"a", "b", "c"},
			output: []string{"a", "b", "c"},
		},
	}

	for _, tt := range tests {
		if result := slice.SplitCommaSeparated(tt.input); !reflect.DeepEqual(result, tt.output) {
			t.Errorf("SplitCommaSeparated(%v) = %v; wanted %v", tt.input, result, tt.output)
		}
	}
}

func TestSplitOffset_OK(t *testing.T) {
	testCases := []struct {
		listSize uint64
		chunks   uint64
		index    uint64
		offset   uint64
	}{
		{30, 3, 2, 20},
		{1000, 10, 60, 6000},
		{2482, 10, 70, 17374},
		{323, 98, 56, 184},
		{273, 8, 6, 204},
		{3274, 98, 256, 8552},
		{23, 3, 2, 15},
		{23, 3, 9, 69},
	}
	for _, tt := range testCases {
		result := slice.SplitOffset(tt.listSize, tt.chunks, tt.index)
		if result != tt.offset {
			t.Errorf("got %d, want %d", result, tt.offset)
		}

	}
}

func TestIntersectionSlot(t *testing.T) {
	testCases := []struct {
		setA []primitives.Slot
		setB []primitives.Slot
		setC []primitives.Slot
		out  []primitives.Slot
	}{
		{[]primitives.Slot{2, 3, 5}, []primitives.Slot{3}, []primitives.Slot{3}, []primitives.Slot{3}},
		{[]primitives.Slot{2, 3, 5}, []primitives.Slot{3, 5}, []primitives.Slot{5}, []primitives.Slot{5}},
		{[]primitives.Slot{2, 3, 5}, []primitives.Slot{3, 5}, []primitives.Slot{3, 5}, []primitives.Slot{3, 5}},
		{[]primitives.Slot{2, 3, 5}, []primitives.Slot{5, 3, 2}, []primitives.Slot{3, 2, 5}, []primitives.Slot{2, 3, 5}},
		{[]primitives.Slot{3, 2, 5}, []primitives.Slot{5, 3, 2}, []primitives.Slot{3, 2, 5}, []primitives.Slot{2, 3, 5}},
		{[]primitives.Slot{3, 3, 5}, []primitives.Slot{5, 3, 2}, []primitives.Slot{3, 2, 5}, []primitives.Slot{3, 5}},
		{[]primitives.Slot{2, 3, 5}, []primitives.Slot{2, 3, 5}, []primitives.Slot{2, 3, 5}, []primitives.Slot{2, 3, 5}},
		{[]primitives.Slot{2, 3, 5}, []primitives.Slot{}, []primitives.Slot{}, []primitives.Slot{}},
		{[]primitives.Slot{2, 3, 5}, []primitives.Slot{2, 3, 5}, []primitives.Slot{}, []primitives.Slot{}},
		{[]primitives.Slot{2, 3}, []primitives.Slot{2, 3, 5}, []primitives.Slot{5}, []primitives.Slot{}},
		{[]primitives.Slot{2, 2, 2}, []primitives.Slot{2, 2, 2}, []primitives.Slot{}, []primitives.Slot{}},
		{[]primitives.Slot{}, []primitives.Slot{2, 3, 5}, []primitives.Slot{}, []primitives.Slot{}},
		{[]primitives.Slot{}, []primitives.Slot{}, []primitives.Slot{}, []primitives.Slot{}},
		{[]primitives.Slot{1}, []primitives.Slot{1}, []primitives.Slot{}, []primitives.Slot{}},
		{[]primitives.Slot{1, 1, 1}, []primitives.Slot{1, 1}, []primitives.Slot{1, 2, 3}, []primitives.Slot{1}},
	}
	for _, tt := range testCases {
		setA := append([]primitives.Slot{}, tt.setA...)
		setB := append([]primitives.Slot{}, tt.setB...)
		setC := append([]primitives.Slot{}, tt.setC...)
		result := slice.IntersectionSlot(setA, setB, setC)
		sort.Slice(result, func(i, j int) bool {
			return result[i] < result[j]
		})
		if !reflect.DeepEqual(result, tt.out) {
			t.Errorf("got %d, want %d", result, tt.out)
		}
		if !reflect.DeepEqual(setA, tt.setA) {
			t.Errorf("slice modified, got %v, want %v", setA, tt.setA)
		}
		if !reflect.DeepEqual(setB, tt.setB) {
			t.Errorf("slice modified, got %v, want %v", setB, tt.setB)
		}
		if !reflect.DeepEqual(setC, tt.setC) {
			t.Errorf("slice modified, got %v, want %v", setC, tt.setC)
		}
	}
}

func TestNotSlot(t *testing.T) {
	testCases := []struct {
		setA []primitives.Slot
		setB []primitives.Slot
		out  []primitives.Slot
	}{
		{[]primitives.Slot{4, 6}, []primitives.Slot{2, 3, 5, 4, 6}, []primitives.Slot{2, 3, 5}},
		{[]primitives.Slot{3, 5}, []primitives.Slot{2, 3, 5}, []primitives.Slot{2}},
		{[]primitives.Slot{2, 3, 5}, []primitives.Slot{2, 3, 5}, []primitives.Slot{}},
		{[]primitives.Slot{2}, []primitives.Slot{2, 3, 5}, []primitives.Slot{3, 5}},
		{[]primitives.Slot{}, []primitives.Slot{2, 3, 5}, []primitives.Slot{2, 3, 5}},
		{[]primitives.Slot{}, []primitives.Slot{}, []primitives.Slot{}},
		{[]primitives.Slot{1}, []primitives.Slot{1}, []primitives.Slot{}},
	}
	for _, tt := range testCases {
		result := slice.NotSlot(tt.setA, tt.setB)
		if !reflect.DeepEqual(result, tt.out) {
			t.Errorf("got %d, want %d", result, tt.out)
		}
	}
}

func TestIsInSlots(t *testing.T) {
	testCases := []struct {
		a      primitives.Slot
		b      []primitives.Slot
		result bool
	}{
		{0, []primitives.Slot{}, false},
		{0, []primitives.Slot{0}, true},
		{4, []primitives.Slot{2, 3, 5, 4, 6}, true},
		{100, []primitives.Slot{2, 3, 5, 4, 6}, false},
	}
	for _, tt := range testCases {
		result := slice.IsInSlots(tt.a, tt.b)
		if result != tt.result {
			t.Errorf("IsIn(%d, %v)=%v, wanted: %v",
				tt.a, tt.b, result, tt.result)
		}
	}
}

func TestUnique(t *testing.T) {
	t.Run("string", func(t *testing.T) {
		result := slice.Unique[string]([]string{"a", "b", "a"})
		require.DeepEqual(t, []string{"a", "b"}, result)
	})
	t.Run("uint64", func(t *testing.T) {
		result := slice.Unique[uint64]([]uint64{1, 2, 1})
		require.DeepEqual(t, []uint64{1, 2}, result)
	})
}

func TestReverse(t *testing.T) {
	tests := []struct {
		value [][32]byte
		want  [][32]byte
	}{
		{[][32]byte{}, [][32]byte{}},
		{[][32]byte{{'A'}, {'B'}, {'C'}, {'D'}},
			[][32]byte{{'D'}, {'C'}, {'B'}, {'A'}}},
		{[][32]byte{{1}, {2}, {3}, {4}},
			[][32]byte{{4}, {3}, {2}, {1}}},
	}
	for _, tt := range tests {
		b := slice.Reverse(tt.value)
		require.DeepEqual(t, tt.want, b)
	}
}
