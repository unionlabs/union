package y

import (
	"errors"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestCombineWithBothErrorsPresent(t *testing.T) {
	combinedError := CombineErrors(errors.New("one"), errors.New("two"))
	require.Equal(t, "one; two", combinedError.Error())
}

func TestCombineErrorsWithOneErrorPresent(t *testing.T) {
	combinedError := CombineErrors(errors.New("one"), nil)
	require.Equal(t, "one", combinedError.Error())
}

func TestCombineErrorsWithOtherErrorPresent(t *testing.T) {
	combinedError := CombineErrors(nil, errors.New("other"))
	require.Equal(t, "other", combinedError.Error())
}

func TestCombineErrorsWithBothErrorsAsNil(t *testing.T) {
	combinedError := CombineErrors(nil, nil)
	require.NoError(t, combinedError)
}
