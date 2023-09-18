package types_test

import (
	"testing"
	"time"

	"github.com/cometbft/cometbft/libs/log"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/upgrade/types"
)

func mustParseTime(s string) time.Time {
	t, err := time.Parse(time.RFC3339, s)
	if err != nil {
		panic(err)
	}
	return t
}

func TestPlanString(t *testing.T) {
	cases := map[string]struct {
		p      types.Plan
		expect string
	}{
		"with height": {
			p: types.Plan{
				Name:   "by height",
				Info:   "https://foo.bar/baz",
				Height: 7890,
			},
			expect: "Upgrade Plan\n  Name: by height\n  height: 7890\n  Info: https://foo.bar/baz.",
		},
		"neither": {
			p: types.Plan{
				Name: "almost-empty",
			},
			expect: "Upgrade Plan\n  Name: almost-empty\n  height: 0\n  Info: .",
		},
	}

	for name, tc := range cases {
		tc := tc // copy to local variable for scopelint
		t.Run(name, func(t *testing.T) {
			s := tc.p.String()
			require.Equal(t, tc.expect, s)
		})
	}
}

func TestPlanValid(t *testing.T) {
	cases := map[string]struct {
		p     types.Plan
		valid bool
	}{
		"proper by height": {
			p: types.Plan{
				Name:   "all-good",
				Height: 123450000,
			},
			valid: true,
		},
		"no name": {
			p: types.Plan{
				Height: 123450000,
			},
		},
		"time-base upgrade": {
			p: types.Plan{
				Time: time.Now(),
			},
		},
		"IBC upgrade": {
			p: types.Plan{
				Height:              123450000,
				UpgradedClientState: &codectypes.Any{},
			},
		},
		"no due at": {
			p: types.Plan{
				Name: "missing",
				Info: "important",
			},
		},
		"negative height": {
			p: types.Plan{
				Name:   "minus",
				Height: -12345,
			},
		},
	}

	for name, tc := range cases {
		tc := tc // copy to local variable for scopelint
		t.Run(name, func(t *testing.T) {
			err := tc.p.ValidateBasic()
			if tc.valid {
				assert.NoError(t, err)
			} else {
				assert.Error(t, err)
			}
		})
	}
}

func TestShouldExecute(t *testing.T) {
	cases := map[string]struct {
		p         types.Plan
		ctxTime   time.Time
		ctxHeight int64
		expected  bool
	}{
		"past height": {
			p: types.Plan{
				Name:   "do-good",
				Height: 1234,
			},
			ctxTime:   mustParseTime("2019-07-08T11:32:00Z"),
			ctxHeight: 1000,
			expected:  false,
		},
		"on height": {
			p: types.Plan{
				Name:   "do-good",
				Height: 1234,
			},
			ctxTime:   mustParseTime("2019-07-08T11:32:00Z"),
			ctxHeight: 1234,
			expected:  true,
		},
		"future height": {
			p: types.Plan{
				Name:   "do-good",
				Height: 1234,
			},
			ctxTime:   mustParseTime("2019-07-08T11:32:00Z"),
			ctxHeight: 1235,
			expected:  true,
		},
	}

	for name, tc := range cases {
		tc := tc // copy to local variable for scopelint
		t.Run(name, func(t *testing.T) {
			ctx := sdk.NewContext(nil, tmproto.Header{Height: tc.ctxHeight, Time: tc.ctxTime}, false, log.NewNopLogger())
			should := tc.p.ShouldExecute(ctx)
			assert.Equal(t, tc.expected, should)
		})
	}
}
