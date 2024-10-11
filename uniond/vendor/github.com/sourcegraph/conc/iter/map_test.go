package iter

import (
	"errors"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
)

func ExampleMapper() {
	input := []int{1, 2, 3, 4}
	mapper := Mapper[int, bool]{
		MaxGoroutines: len(input) / 2,
	}

	results := mapper.Map(input, func(v *int) bool { return *v%2 == 0 })
	fmt.Println(results)
	// Output:
	// [false true false true]
}

func TestMap(t *testing.T) {
	t.Parallel()

	t.Run("empty", func(t *testing.T) {
		t.Parallel()
		f := func() {
			ints := []int{}
			Map(ints, func(val *int) int {
				panic("this should never be called")
			})
		}
		require.NotPanics(t, f)
	})

	t.Run("panic is propagated", func(t *testing.T) {
		t.Parallel()
		f := func() {
			ints := []int{1}
			Map(ints, func(val *int) int {
				panic("super bad thing happened")
			})
		}
		require.Panics(t, f)
	})

	t.Run("mutating inputs is fine, though not recommended", func(t *testing.T) {
		t.Parallel()
		ints := []int{1, 2, 3, 4, 5}
		Map(ints, func(val *int) int {
			*val += 1
			return 0
		})
		require.Equal(t, []int{2, 3, 4, 5, 6}, ints)
	})

	t.Run("basic increment", func(t *testing.T) {
		t.Parallel()
		ints := []int{1, 2, 3, 4, 5}
		res := Map(ints, func(val *int) int {
			return *val + 1
		})
		require.Equal(t, []int{2, 3, 4, 5, 6}, res)
		require.Equal(t, []int{1, 2, 3, 4, 5}, ints)
	})

	t.Run("huge inputs", func(t *testing.T) {
		t.Parallel()
		ints := make([]int, 10000)
		res := Map(ints, func(val *int) int {
			return 1
		})
		expected := make([]int, 10000)
		for i := 0; i < 10000; i++ {
			expected[i] = 1
		}
		require.Equal(t, expected, res)
	})
}

func TestMapErr(t *testing.T) {
	t.Parallel()

	t.Run("empty", func(t *testing.T) {
		t.Parallel()
		f := func() {
			ints := []int{}
			res, err := MapErr(ints, func(val *int) (int, error) {
				panic("this should never be called")
			})
			require.NoError(t, err)
			require.Equal(t, ints, res)
		}
		require.NotPanics(t, f)
	})

	t.Run("panic is propagated", func(t *testing.T) {
		t.Parallel()
		f := func() {
			ints := []int{1}
			_, _ = MapErr(ints, func(val *int) (int, error) {
				panic("super bad thing happened")
			})
		}
		require.Panics(t, f)
	})

	t.Run("mutating inputs is fine, though not recommended", func(t *testing.T) {
		t.Parallel()
		ints := []int{1, 2, 3, 4, 5}
		res, err := MapErr(ints, func(val *int) (int, error) {
			*val += 1
			return 0, nil
		})
		require.NoError(t, err)
		require.Equal(t, []int{2, 3, 4, 5, 6}, ints)
		require.Equal(t, []int{0, 0, 0, 0, 0}, res)
	})

	t.Run("basic increment", func(t *testing.T) {
		t.Parallel()
		ints := []int{1, 2, 3, 4, 5}
		res, err := MapErr(ints, func(val *int) (int, error) {
			return *val + 1, nil
		})
		require.NoError(t, err)
		require.Equal(t, []int{2, 3, 4, 5, 6}, res)
		require.Equal(t, []int{1, 2, 3, 4, 5}, ints)
	})

	err1 := errors.New("error1")
	err2 := errors.New("error1")

	t.Run("error is propagated", func(t *testing.T) {
		t.Parallel()
		ints := []int{1, 2, 3, 4, 5}
		res, err := MapErr(ints, func(val *int) (int, error) {
			if *val == 3 {
				return 0, err1
			}
			return *val + 1, nil
		})
		require.ErrorIs(t, err, err1)
		require.Equal(t, []int{2, 3, 0, 5, 6}, res)
		require.Equal(t, []int{1, 2, 3, 4, 5}, ints)
	})

	t.Run("multiple errors are propagated", func(t *testing.T) {
		t.Parallel()
		ints := []int{1, 2, 3, 4, 5}
		res, err := MapErr(ints, func(val *int) (int, error) {
			if *val == 3 {
				return 0, err1
			}
			if *val == 4 {
				return 0, err2
			}
			return *val + 1, nil
		})
		require.ErrorIs(t, err, err1)
		require.ErrorIs(t, err, err2)
		require.Equal(t, []int{2, 3, 0, 0, 6}, res)
		require.Equal(t, []int{1, 2, 3, 4, 5}, ints)
	})

	t.Run("huge inputs", func(t *testing.T) {
		t.Parallel()
		ints := make([]int, 10000)
		res := Map(ints, func(val *int) int {
			return 1
		})
		expected := make([]int, 10000)
		for i := 0; i < 10000; i++ {
			expected[i] = 1
		}
		require.Equal(t, expected, res)
	})
}
