package sentry

import (
	"reflect"
	"testing"
)

var input = [][]byte{
	[]byte("line 1"),
	[]byte("line 2"),
	[]byte("line 3"),
	[]byte("line 4"),
	[]byte("line 5"),
}

func assertContextLines(t *testing.T, gotLines, wantLines [][]byte, gotReadLines, wantReadLines int) {
	t.Helper()

	if !reflect.DeepEqual(gotLines, wantLines) {
		t.Errorf("incorrect context lines returned. got %s, want %s", gotLines, wantLines)
	}

	if gotReadLines != wantReadLines {
		t.Errorf("incorrect context lines count returned. got %d, want %d", gotReadLines, wantReadLines)
	}
}

func TestCalculateContextLinesSingleLine(t *testing.T) {
	sr := newSourceReader()
	gotLines, gotReadLines := sr.calculateContextLines(input, 2, 0)
	wantLines, wantReadLines := [][]byte{
		[]byte("line 2"),
	}, 0

	assertContextLines(t, gotLines, wantLines, gotReadLines, wantReadLines)
}

func TestCalculateContextLinesNegativeLine(t *testing.T) {
	sr := newSourceReader()
	gotLines, gotReadLines := sr.calculateContextLines(input, -2, 0)
	var wantLines [][]byte
	var wantReadLines int

	assertContextLines(t, gotLines, wantLines, gotReadLines, wantReadLines)
}

func TestCalculateContextLinesNegativeContext(t *testing.T) {
	sr := newSourceReader()
	gotLines, gotReadLines := sr.calculateContextLines(input, 2, -2)
	wantLines, wantReadLines := [][]byte{
		[]byte("line 2"),
	}, 0

	assertContextLines(t, gotLines, wantLines, gotReadLines, wantReadLines)
}
func TestCalculateContextLinesOverflowLine(t *testing.T) {
	sr := newSourceReader()
	gotLines, gotReadLines := sr.calculateContextLines(input, 10, 0)
	var wantLines [][]byte
	var wantReadLines int

	assertContextLines(t, gotLines, wantLines, gotReadLines, wantReadLines)
}

func TestCalculateContextLinesWholeFile(t *testing.T) {
	sr := newSourceReader()
	gotLines, gotReadLines := sr.calculateContextLines(input, 3, 2)
	wantLines, wantReadLines := [][]byte{
		[]byte("line 1"),
		[]byte("line 2"),
		[]byte("line 3"),
		[]byte("line 4"),
		[]byte("line 5"),
	}, 2

	assertContextLines(t, gotLines, wantLines, gotReadLines, wantReadLines)
}

func TestCalculateContextLinesOverflowContextAtTheTop(t *testing.T) {
	sr := newSourceReader()
	gotLines, gotReadLines := sr.calculateContextLines(input, 2, 3)
	wantLines, wantReadLines := [][]byte{
		[]byte("line 1"),
		[]byte("line 2"),
		[]byte("line 3"),
		[]byte("line 4"),
		[]byte("line 5"),
	}, 1

	assertContextLines(t, gotLines, wantLines, gotReadLines, wantReadLines)
}

func TestCalculateContextLinesOverflowContextAtTheBottom(t *testing.T) {
	sr := newSourceReader()
	gotLines, gotReadLines := sr.calculateContextLines(input, 5, 3)
	wantLines, wantReadLines := [][]byte{
		[]byte("line 2"),
		[]byte("line 3"),
		[]byte("line 4"),
		[]byte("line 5"),
	}, 3

	assertContextLines(t, gotLines, wantLines, gotReadLines, wantReadLines)
}

func TestCalculateContextLinesOverflowContextBothSides(t *testing.T) {
	sr := newSourceReader()
	gotLines, gotReadLines := sr.calculateContextLines(input, 2, 10)
	wantLines, wantReadLines := [][]byte{
		[]byte("line 1"),
		[]byte("line 2"),
		[]byte("line 3"),
		[]byte("line 4"),
		[]byte("line 5"),
	}, 1

	assertContextLines(t, gotLines, wantLines, gotReadLines, wantReadLines)
}

func TestReadContextLinesNonExistingInput(t *testing.T) {
	sr := newSourceReader()
	gotLines, gotReadLines := sr.readContextLines("non_existing.go", 2, 10)
	var wantLines [][]byte
	var wantReadLines int

	assertContextLines(t, gotLines, wantLines, gotReadLines, wantReadLines)
	assertEqual(t, sr.cache["non_existing.go"], wantLines)
}
