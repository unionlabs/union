package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
)

func TestParseDenomTrace(t *testing.T) {
	testCases := []struct {
		name     string
		denom    string
		expTrace types.DenomTrace
	}{
		{"empty denom", "", types.DenomTrace{}},
		{"base denom", "uatom", types.DenomTrace{BaseDenom: "uatom"}},
		{"base denom ending with '/'", "uatom/", types.DenomTrace{BaseDenom: "uatom/"}},
		{"base denom with single '/'s", "gamm/pool/1", types.DenomTrace{BaseDenom: "gamm/pool/1"}},
		{"base denom with double '/'s", "gamm//pool//1", types.DenomTrace{BaseDenom: "gamm//pool//1"}},
		{"trace info", "transfer/channel-1/uatom", types.DenomTrace{BaseDenom: "uatom", Path: "transfer/channel-1"}},
		{"trace info with custom port", "customtransfer/channel-1/uatom", types.DenomTrace{BaseDenom: "uatom", Path: "customtransfer/channel-1"}},
		{"trace info with base denom ending in '/'", "transfer/channel-1/uatom/", types.DenomTrace{BaseDenom: "uatom/", Path: "transfer/channel-1"}},
		{"trace info with single '/' in base denom", "transfer/channel-1/erc20/0x85bcBCd7e79Ec36f4fBBDc54F90C643d921151AA", types.DenomTrace{BaseDenom: "erc20/0x85bcBCd7e79Ec36f4fBBDc54F90C643d921151AA", Path: "transfer/channel-1"}},
		{"trace info with multiple '/'s in base denom", "transfer/channel-1/gamm/pool/1", types.DenomTrace{BaseDenom: "gamm/pool/1", Path: "transfer/channel-1"}},
		{"trace info with multiple double '/'s in base denom", "transfer/channel-1/gamm//pool//1", types.DenomTrace{BaseDenom: "gamm//pool//1", Path: "transfer/channel-1"}},
		{"trace info with multiple port/channel pairs", "transfer/channel-1/transfer/channel-2/uatom", types.DenomTrace{BaseDenom: "uatom", Path: "transfer/channel-1/transfer/channel-2"}},
		{"trace info with multiple custom ports", "customtransfer/channel-1/alternativetransfer/channel-2/uatom", types.DenomTrace{BaseDenom: "uatom", Path: "customtransfer/channel-1/alternativetransfer/channel-2"}},
		{"incomplete path", "transfer/uatom", types.DenomTrace{BaseDenom: "transfer/uatom"}},
		{"invalid path (1)", "transfer//uatom", types.DenomTrace{BaseDenom: "transfer//uatom", Path: ""}},
		{"invalid path (2)", "channel-1/transfer/uatom", types.DenomTrace{BaseDenom: "channel-1/transfer/uatom"}},
		{"invalid path (3)", "uatom/transfer", types.DenomTrace{BaseDenom: "uatom/transfer"}},
		{"invalid path (4)", "transfer/channel-1", types.DenomTrace{BaseDenom: "transfer/channel-1"}},
		{"invalid path (5)", "transfer/channel-1/", types.DenomTrace{Path: "transfer/channel-1"}},
		{"invalid path (6)", "transfer/channel-1/transfer", types.DenomTrace{BaseDenom: "transfer", Path: "transfer/channel-1"}},
		{"invalid path (7)", "transfer/channel-1/transfer/channel-2", types.DenomTrace{Path: "transfer/channel-1/transfer/channel-2"}},
		{"invalid path (8)", "transfer/channelToA/uatom", types.DenomTrace{BaseDenom: "transfer/channelToA/uatom", Path: ""}},
	}

	for _, tc := range testCases {
		tc := tc

		trace := types.ParseDenomTrace(tc.denom)
		require.Equal(t, tc.expTrace, trace, tc.name)
	}
}

func TestDenomTrace_IBCDenom(t *testing.T) {
	testCases := []struct {
		name     string
		trace    types.DenomTrace
		expDenom string
	}{
		{"base denom", types.DenomTrace{BaseDenom: "uatom"}, "uatom"},
		{"trace info", types.DenomTrace{BaseDenom: "uatom", Path: "transfer/channel-1"}, "ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9"},
	}

	for _, tc := range testCases {
		tc := tc

		denom := tc.trace.IBCDenom()
		require.Equal(t, tc.expDenom, denom, tc.name)
	}
}

func TestDenomTrace_Validate(t *testing.T) {
	testCases := []struct {
		name     string
		trace    types.DenomTrace
		expError bool
	}{
		{"base denom only", types.DenomTrace{BaseDenom: "uatom"}, false},
		{"base denom only with single '/'", types.DenomTrace{BaseDenom: "erc20/0x85bcBCd7e79Ec36f4fBBDc54F90C643d921151AA"}, false},
		{"base denom only with multiple '/'s", types.DenomTrace{BaseDenom: "gamm/pool/1"}, false},
		{"empty DenomTrace", types.DenomTrace{}, true},
		{"valid single trace info", types.DenomTrace{BaseDenom: "uatom", Path: "transfer/channel-1"}, false},
		{"valid multiple trace info", types.DenomTrace{BaseDenom: "uatom", Path: "transfer/channel-1/transfer/channel-2"}, false},
		{"single trace identifier", types.DenomTrace{BaseDenom: "uatom", Path: "transfer"}, true},
		{"invalid port ID", types.DenomTrace{BaseDenom: "uatom", Path: "(transfer)/channel-1"}, true},
		{"invalid channel ID", types.DenomTrace{BaseDenom: "uatom", Path: "transfer/(channel-1)"}, true},
		{"empty base denom with trace", types.DenomTrace{BaseDenom: "", Path: "transfer/channel-1"}, true},
	}

	for _, tc := range testCases {
		tc := tc

		err := tc.trace.Validate()
		if tc.expError {
			require.Error(t, err, tc.name)
			continue
		}
		require.NoError(t, err, tc.name)
	}
}

func TestTraces_Validate(t *testing.T) {
	testCases := []struct {
		name     string
		traces   types.Traces
		expError bool
	}{
		{"empty Traces", types.Traces{}, false},
		{"valid multiple trace info", types.Traces{{BaseDenom: "uatom", Path: "transfer/channel-1/transfer/channel-2"}}, false},
		{
			"valid multiple trace info",
			types.Traces{
				{BaseDenom: "uatom", Path: "transfer/channel-1/transfer/channel-2"},
				{BaseDenom: "uatom", Path: "transfer/channel-1/transfer/channel-2"},
			},
			true,
		},
		{"empty base denom with trace", types.Traces{{BaseDenom: "", Path: "transfer/channel-1"}}, true},
	}

	for _, tc := range testCases {
		tc := tc

		err := tc.traces.Validate()
		if tc.expError {
			require.Error(t, err, tc.name)
			continue
		}
		require.NoError(t, err, tc.name)
	}
}

func TestValidatePrefixedDenom(t *testing.T) {
	testCases := []struct {
		name     string
		denom    string
		expError bool
	}{
		{"prefixed denom", "transfer/channel-1/uatom", false},
		{"prefixed denom with '/'", "transfer/channel-1/gamm/pool/1", false},
		{"empty prefix", "/uatom", false},
		{"empty identifiers", "//uatom", false},
		{"base denom", "uatom", false},
		{"base denom with single '/'", "erc20/0x85bcBCd7e79Ec36f4fBBDc54F90C643d921151AA", false},
		{"base denom with multiple '/'s", "gamm/pool/1", false},
		{"invalid port ID", "(transfer)/channel-1/uatom", true},
		{"empty denom", "", true},
		{"single trace identifier", "transfer/", true},
	}

	for _, tc := range testCases {
		tc := tc

		err := types.ValidatePrefixedDenom(tc.denom)
		if tc.expError {
			require.Error(t, err, tc.name)
		} else {
			require.NoError(t, err, tc.name)
		}
	}
}

func TestValidateIBCDenom(t *testing.T) {
	testCases := []struct {
		name     string
		denom    string
		expError bool
	}{
		{"denom with trace hash", "ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2", false},
		{"base denom", "uatom", false},
		{"base denom ending with '/'", "uatom/", false},
		{"base denom with single '/'s", "gamm/pool/1", false},
		{"base denom with double '/'s", "gamm//pool//1", false},
		{"non-ibc prefix with hash", "notibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2", false},
		{"empty denom", "", true},
		{"denom 'ibc'", "ibc", true},
		{"denom 'ibc/'", "ibc/", true},
		{"invald hash", "ibc/!@#$!@#", true},
	}

	for _, tc := range testCases {
		tc := tc

		err := types.ValidateIBCDenom(tc.denom)
		if tc.expError {
			require.Error(t, err, tc.name)
			continue
		}
		require.NoError(t, err, tc.name)
	}
}
