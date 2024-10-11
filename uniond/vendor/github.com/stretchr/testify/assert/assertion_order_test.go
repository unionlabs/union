package assert

import (
	"bytes"
	"testing"
)

func TestIsIncreasing(t *testing.T) {
	mockT := new(testing.T)

	if !IsIncreasing(mockT, []int{1, 2}) {
		t.Error("IsIncreasing should return true")
	}

	if !IsIncreasing(mockT, []int{1, 2, 3, 4, 5}) {
		t.Error("IsIncreasing should return true")
	}

	if IsIncreasing(mockT, []int{1, 1}) {
		t.Error("IsIncreasing should return false")
	}

	if IsIncreasing(mockT, []int{2, 1}) {
		t.Error("IsIncreasing should return false")
	}

	// Check error report
	for _, currCase := range []struct {
		collection interface{}
		msg        string
	}{
		{collection: []string{"b", "a"}, msg: `"b" is not less than "a"`},
		{collection: []int{2, 1}, msg: `"2" is not less than "1"`},
		{collection: []int{2, 1, 3, 4, 5, 6, 7}, msg: `"2" is not less than "1"`},
		{collection: []int{-1, 0, 2, 1}, msg: `"2" is not less than "1"`},
		{collection: []int8{2, 1}, msg: `"2" is not less than "1"`},
		{collection: []int16{2, 1}, msg: `"2" is not less than "1"`},
		{collection: []int32{2, 1}, msg: `"2" is not less than "1"`},
		{collection: []int64{2, 1}, msg: `"2" is not less than "1"`},
		{collection: []uint8{2, 1}, msg: `"2" is not less than "1"`},
		{collection: []uint16{2, 1}, msg: `"2" is not less than "1"`},
		{collection: []uint32{2, 1}, msg: `"2" is not less than "1"`},
		{collection: []uint64{2, 1}, msg: `"2" is not less than "1"`},
		{collection: []float32{2.34, 1.23}, msg: `"2.34" is not less than "1.23"`},
		{collection: []float64{2.34, 1.23}, msg: `"2.34" is not less than "1.23"`},
	} {
		out := &outputT{buf: bytes.NewBuffer(nil)}
		False(t, IsIncreasing(out, currCase.collection))
		Contains(t, out.buf.String(), currCase.msg)
	}
}

func TestIsNonIncreasing(t *testing.T) {
	mockT := new(testing.T)

	if !IsNonIncreasing(mockT, []int{2, 1}) {
		t.Error("IsNonIncreasing should return true")
	}

	if !IsNonIncreasing(mockT, []int{5, 4, 4, 3, 2, 1}) {
		t.Error("IsNonIncreasing should return true")
	}

	if !IsNonIncreasing(mockT, []int{1, 1}) {
		t.Error("IsNonIncreasing should return true")
	}

	if IsNonIncreasing(mockT, []int{1, 2}) {
		t.Error("IsNonIncreasing should return false")
	}

	// Check error report
	for _, currCase := range []struct {
		collection interface{}
		msg        string
	}{
		{collection: []string{"a", "b"}, msg: `"a" is not greater than or equal to "b"`},
		{collection: []int{1, 2}, msg: `"1" is not greater than or equal to "2"`},
		{collection: []int{1, 2, 7, 6, 5, 4, 3}, msg: `"1" is not greater than or equal to "2"`},
		{collection: []int{5, 4, 3, 1, 2}, msg: `"1" is not greater than or equal to "2"`},
		{collection: []int8{1, 2}, msg: `"1" is not greater than or equal to "2"`},
		{collection: []int16{1, 2}, msg: `"1" is not greater than or equal to "2"`},
		{collection: []int32{1, 2}, msg: `"1" is not greater than or equal to "2"`},
		{collection: []int64{1, 2}, msg: `"1" is not greater than or equal to "2"`},
		{collection: []uint8{1, 2}, msg: `"1" is not greater than or equal to "2"`},
		{collection: []uint16{1, 2}, msg: `"1" is not greater than or equal to "2"`},
		{collection: []uint32{1, 2}, msg: `"1" is not greater than or equal to "2"`},
		{collection: []uint64{1, 2}, msg: `"1" is not greater than or equal to "2"`},
		{collection: []float32{1.23, 2.34}, msg: `"1.23" is not greater than or equal to "2.34"`},
		{collection: []float64{1.23, 2.34}, msg: `"1.23" is not greater than or equal to "2.34"`},
	} {
		out := &outputT{buf: bytes.NewBuffer(nil)}
		False(t, IsNonIncreasing(out, currCase.collection))
		Contains(t, out.buf.String(), currCase.msg)
	}
}

func TestIsDecreasing(t *testing.T) {
	mockT := new(testing.T)

	if !IsDecreasing(mockT, []int{2, 1}) {
		t.Error("IsDecreasing should return true")
	}

	if !IsDecreasing(mockT, []int{5, 4, 3, 2, 1}) {
		t.Error("IsDecreasing should return true")
	}

	if IsDecreasing(mockT, []int{1, 1}) {
		t.Error("IsDecreasing should return false")
	}

	if IsDecreasing(mockT, []int{1, 2}) {
		t.Error("IsDecreasing should return false")
	}

	// Check error report
	for _, currCase := range []struct {
		collection interface{}
		msg        string
	}{
		{collection: []string{"a", "b"}, msg: `"a" is not greater than "b"`},
		{collection: []int{1, 2}, msg: `"1" is not greater than "2"`},
		{collection: []int{1, 2, 7, 6, 5, 4, 3}, msg: `"1" is not greater than "2"`},
		{collection: []int{5, 4, 3, 1, 2}, msg: `"1" is not greater than "2"`},
		{collection: []int8{1, 2}, msg: `"1" is not greater than "2"`},
		{collection: []int16{1, 2}, msg: `"1" is not greater than "2"`},
		{collection: []int32{1, 2}, msg: `"1" is not greater than "2"`},
		{collection: []int64{1, 2}, msg: `"1" is not greater than "2"`},
		{collection: []uint8{1, 2}, msg: `"1" is not greater than "2"`},
		{collection: []uint16{1, 2}, msg: `"1" is not greater than "2"`},
		{collection: []uint32{1, 2}, msg: `"1" is not greater than "2"`},
		{collection: []uint64{1, 2}, msg: `"1" is not greater than "2"`},
		{collection: []float32{1.23, 2.34}, msg: `"1.23" is not greater than "2.34"`},
		{collection: []float64{1.23, 2.34}, msg: `"1.23" is not greater than "2.34"`},
	} {
		out := &outputT{buf: bytes.NewBuffer(nil)}
		False(t, IsDecreasing(out, currCase.collection))
		Contains(t, out.buf.String(), currCase.msg)
	}
}

func TestIsNonDecreasing(t *testing.T) {
	mockT := new(testing.T)

	if !IsNonDecreasing(mockT, []int{1, 2}) {
		t.Error("IsNonDecreasing should return true")
	}

	if !IsNonDecreasing(mockT, []int{1, 1, 2, 3, 4, 5}) {
		t.Error("IsNonDecreasing should return true")
	}

	if !IsNonDecreasing(mockT, []int{1, 1}) {
		t.Error("IsNonDecreasing should return false")
	}

	if IsNonDecreasing(mockT, []int{2, 1}) {
		t.Error("IsNonDecreasing should return false")
	}

	// Check error report
	for _, currCase := range []struct {
		collection interface{}
		msg        string
	}{
		{collection: []string{"b", "a"}, msg: `"b" is not less than or equal to "a"`},
		{collection: []int{2, 1}, msg: `"2" is not less than or equal to "1"`},
		{collection: []int{2, 1, 3, 4, 5, 6, 7}, msg: `"2" is not less than or equal to "1"`},
		{collection: []int{-1, 0, 2, 1}, msg: `"2" is not less than or equal to "1"`},
		{collection: []int8{2, 1}, msg: `"2" is not less than or equal to "1"`},
		{collection: []int16{2, 1}, msg: `"2" is not less than or equal to "1"`},
		{collection: []int32{2, 1}, msg: `"2" is not less than or equal to "1"`},
		{collection: []int64{2, 1}, msg: `"2" is not less than or equal to "1"`},
		{collection: []uint8{2, 1}, msg: `"2" is not less than or equal to "1"`},
		{collection: []uint16{2, 1}, msg: `"2" is not less than or equal to "1"`},
		{collection: []uint32{2, 1}, msg: `"2" is not less than or equal to "1"`},
		{collection: []uint64{2, 1}, msg: `"2" is not less than or equal to "1"`},
		{collection: []float32{2.34, 1.23}, msg: `"2.34" is not less than or equal to "1.23"`},
		{collection: []float64{2.34, 1.23}, msg: `"2.34" is not less than or equal to "1.23"`},
	} {
		out := &outputT{buf: bytes.NewBuffer(nil)}
		False(t, IsNonDecreasing(out, currCase.collection))
		Contains(t, out.buf.String(), currCase.msg)
	}
}

func TestOrderingMsgAndArgsForwarding(t *testing.T) {
	msgAndArgs := []interface{}{"format %s %x", "this", 0xc001}
	expectedOutput := "format this c001\n"
	collection := []int{1, 2, 1}
	funcs := []func(t TestingT){
		func(t TestingT) { IsIncreasing(t, collection, msgAndArgs...) },
		func(t TestingT) { IsNonIncreasing(t, collection, msgAndArgs...) },
		func(t TestingT) { IsDecreasing(t, collection, msgAndArgs...) },
		func(t TestingT) { IsNonDecreasing(t, collection, msgAndArgs...) },
	}
	for _, f := range funcs {
		out := &outputT{buf: bytes.NewBuffer(nil)}
		f(out)
		Contains(t, out.buf.String(), expectedOutput)
	}
}
