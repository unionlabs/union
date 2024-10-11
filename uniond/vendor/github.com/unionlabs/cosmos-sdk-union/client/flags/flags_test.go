package flags_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client/flags"
)

func TestParseGasSetting(t *testing.T) {
	testCases := []struct {
		name      string
		input     string
		expected  flags.GasSetting
		expectErr bool
	}{
		{"empty input", "", flags.GasSetting{false, flags.DefaultGasLimit}, false},
		{"auto", flags.GasFlagAuto, flags.GasSetting{true, 0}, false},
		{"valid custom gas", "73800", flags.GasSetting{false, 73800}, false},
		{"invalid custom gas", "-73800", flags.GasSetting{false, 0}, true},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			gs, err := flags.ParseGasSetting(tc.input)

			if tc.expectErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				require.Equal(t, tc.expected, gs)
			}
		})
	}
}
