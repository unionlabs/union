package list

import (
	"fmt"
	"reflect"
	"testing"
)

func TestListNew(t *testing.T) {
	t.Run("when items a slice nil", func(t *testing.T) {
		_, err := New([]int{1, 2, 3}, 3)
		if err != nil {
			t.Errorf("Expected no errors, error %v", err)
		}
	})

	t.Run("when items is nil", func(t *testing.T) {
		_, err := New(nil, 3)
		if err == nil {
			t.Errorf("Expected error got none")
		}
	})

	t.Run("when items is not a slice", func(t *testing.T) {
		_, err := New("1,2,3", 3)
		if err == nil {
			t.Errorf("Expected error got none")
		}
	})
}

func TestListMovement(t *testing.T) {
	letters := []rune{'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j'}

	l, err := New(letters, 4)
	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}

	tcs := []struct {
		expect   []rune
		move     string
		selected rune
	}{
		{move: "next", selected: 'b', expect: []rune{'a', 'b', 'c', 'd'}},
		{move: "prev", selected: 'a', expect: []rune{'a', 'b', 'c', 'd'}},
		{move: "prev", selected: 'a', expect: []rune{'a', 'b', 'c', 'd'}},
		{move: "next", selected: 'b', expect: []rune{'a', 'b', 'c', 'd'}},
		{move: "next", selected: 'c', expect: []rune{'a', 'b', 'c', 'd'}},
		{move: "next", selected: 'd', expect: []rune{'a', 'b', 'c', 'd'}},
		{move: "next", selected: 'e', expect: []rune{'b', 'c', 'd', 'e'}},
		{move: "prev", selected: 'd', expect: []rune{'b', 'c', 'd', 'e'}},
		{move: "up", selected: 'a', expect: []rune{'a', 'b', 'c', 'd'}},
		{move: "up", selected: 'a', expect: []rune{'a', 'b', 'c', 'd'}},
		{move: "down", selected: 'e', expect: []rune{'e', 'f', 'g', 'h'}},
		{move: "down", selected: 'g', expect: []rune{'g', 'h', 'i', 'j'}},
		{move: "down", selected: 'j', expect: []rune{'g', 'h', 'i', 'j'}},
	}

	for _, tc := range tcs {
		t.Run(fmt.Sprintf("list %s", tc.move), func(t *testing.T) {
			switch tc.move {
			case "next":
				l.Next()
			case "prev":
				l.Prev()
			case "up":
				l.PageUp()
			case "down":
				l.PageDown()
			default:
				t.Fatalf("unknown move %q", tc.move)
			}

			list, idx := l.Items()

			got := castList(list)

			if !reflect.DeepEqual(tc.expect, got) {
				t.Errorf("expected %q, got %q", tc.expect, got)
			}

			selected := list[idx]

			if tc.selected != selected {
				t.Errorf("expected selected to be %q, got %q", tc.selected, selected)
			}
		})
	}
}

func TestListPageDown(t *testing.T) {
	t.Run("when list has fewer items than page size", func(t *testing.T) {
		letters := []rune{'a', 'b'}
		l, err := New(letters, 4)
		if err != nil {
			t.Fatalf("Expected no error, got %v", err)
		}

		l.PageDown()
		list, idx := l.Items()

		expected := 'b'
		selected := list[idx]

		if selected != expected {
			t.Errorf("expected selected to be %q, got %q", expected, selected)
		}
	})
}

func TestListComparion(t *testing.T) {
	t.Run("when item supports comparison", func(t *testing.T) {
		type comparable struct {
			Number int
		}

		structs := []comparable{
			{Number: 1},
			{Number: 2},
		}

		l, err := New(structs, 4)
		if err != nil {
			t.Fatalf("Expected no error, got %v", err)
		}

		idx := l.Index()

		if idx != 0 {
			t.Errorf("expected index to be first, got %d", idx)
		}
	})

	t.Run("when item doesn't support comparison", func(t *testing.T) {
		type uncomparable struct {
			Numbers []int
		}

		structs := []uncomparable{
			{Numbers: []int{1}},
			{Numbers: []int{2}},
		}

		l, err := New(structs, 4)
		if err != nil {
			t.Fatalf("Expected no error, got %v", err)
		}

		idx := l.Index()

		if idx != 0 {
			t.Errorf("expected index to be first, got %d", idx)
		}
	})
}

func castList(list []interface{}) []rune {
	result := make([]rune, len(list))
	for i, l := range list {
		result[i] = l.(rune)
	}
	return result
}
