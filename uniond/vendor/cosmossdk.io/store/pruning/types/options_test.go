package types

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestPruningOptions_Validate(t *testing.T) {
	testCases := []struct {
		opts      PruningOptions
		expectErr error
	}{
		{NewPruningOptions(PruningDefault), nil},
		{NewPruningOptions(PruningEverything), nil},
		{NewPruningOptions(PruningNothing), nil},
		{NewPruningOptions(PruningCustom), ErrPruningIntervalZero},
		{NewCustomPruningOptions(2, 10), nil},
		{NewCustomPruningOptions(100, 15), nil},
		{NewCustomPruningOptions(1, 10), ErrPruningKeepRecentTooSmall},
		{NewCustomPruningOptions(2, 9), ErrPruningIntervalTooSmall},
		{NewCustomPruningOptions(2, 0), ErrPruningIntervalZero},
		{NewCustomPruningOptions(2, 0), ErrPruningIntervalZero},
	}

	for _, tc := range testCases {
		err := tc.opts.Validate()
		require.Equal(t, tc.expectErr, err, "options: %v, err: %s", tc.opts, err)
	}
}

func TestPruningOptions_GetStrategy(t *testing.T) {
	testCases := []struct {
		opts             PruningOptions
		expectedStrategy PruningStrategy
	}{
		{NewPruningOptions(PruningDefault), PruningDefault},
		{NewPruningOptions(PruningEverything), PruningEverything},
		{NewPruningOptions(PruningNothing), PruningNothing},
		{NewPruningOptions(PruningCustom), PruningCustom},
		{NewCustomPruningOptions(2, 10), PruningCustom},
	}

	for _, tc := range testCases {
		actualStrategy := tc.opts.GetPruningStrategy()
		require.Equal(t, tc.expectedStrategy, actualStrategy)
	}
}

func TestNewPruningOptionsFromString(t *testing.T) {
	testCases := []struct {
		optString string
		expect    PruningOptions
	}{
		{PruningOptionDefault, NewPruningOptions(PruningDefault)},
		{PruningOptionEverything, NewPruningOptions(PruningEverything)},
		{PruningOptionNothing, NewPruningOptions(PruningNothing)},
		{"invalid", NewPruningOptions(PruningDefault)},
	}

	for _, tc := range testCases {
		actual := NewPruningOptionsFromString(tc.optString)
		require.Equal(t, tc.expect, actual)
	}
}
